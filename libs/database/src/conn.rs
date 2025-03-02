use diesel::connection::Connection;
use diesel::result::Error as DieselError;
use diesel::PgConnection;
use diesel_async::pooled_connection::bb8::{Pool, PooledConnection};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_async::SimpleAsyncConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use url::Url;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Clone)]
pub struct Database {
    pool: Pool<AsyncPgConnection>,
}

impl Database {
    /// Connect to the database
    ///
    /// This function only establishes a connection to the database and doesn't run any migrations.
    /// It requires the database to already exist and be up-to-date.
    pub async fn connect(url: &str) -> Self {
        let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        let pool = Pool::builder().build(mgr).await.unwrap();
        Self { pool }
    }

    /// Connect and initialize the database
    ///
    /// This function creates the database if it doesn't exist and runs all pending migrations.
    pub async fn connect_and_init(url: &str) -> Self {
        log::info!("Connecting to database at {}", url);
        let db_name = extract_db_name(url);
        let postgres_connection_string = replace_db_name(url, "postgres");

        let mgr =
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(&postgres_connection_string);
        let temp_pool = Pool::builder().build(mgr).await.unwrap();
        let mut conn = temp_pool.get().await.unwrap();

        create_database(&mut conn, &db_name).await.unwrap();

        drop(conn);
        drop(temp_pool);

        run_migrations(url).await.unwrap();

        let mgr = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        let pool = Pool::builder().build(mgr).await.unwrap();

        log::info!("Connected to database {}", db_name);

        Self { pool }
    }

    pub async fn conn(&self) -> Conn<'_> {
        Conn { conn: self.pool.get().await.unwrap() }
    }
}

pub struct Conn<'a> {
    pub(crate) conn: PooledConnection<'a, AsyncPgConnection>,
}

/// Extract the database name from the connection string
fn extract_db_name(connection_string: &str) -> String {
    let url = Url::parse(connection_string).unwrap();
    url.path().trim_start_matches('/').to_string()
}

/// Replace the database name in the connection string
fn replace_db_name(connection_string: &str, new_db_name: &str) -> String {
    let mut url = Url::parse(connection_string).unwrap();
    url.set_path(&format!("/{}", new_db_name));
    url.to_string()
}

/// Create the database if it doesn't exist
async fn create_database(
    conn: &mut AsyncPgConnection,
    db_name: &str,
) -> Result<(), diesel::result::Error> {
    let sql = format!("create database {}", db_name);
    match conn.batch_execute(&sql).await {
        Ok(()) => {
            log::info!("Database {} created", db_name);
            Ok(())
        }
        Err(e) => {
            if let DieselError::DatabaseError(_, ref info) = e {
                // Check if the error message indicates that the database already exists
                if info.message().contains("already exists") {
                    // Database already exists, proceed
                    log::info!("Database {} already exists", db_name);
                    Ok(())
                } else {
                    Err(e)
                }
            } else {
                Err(e)
            }
        }
    }
}

/// Run all pending migrations
///
/// As the diesel-migrations crate is synchronous, we run it in a blocking task and establish a
/// separate connection to the database for this purpose.
async fn run_migrations(
    connection_string: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::info!("Running migrations (if any)");
    let conn_str = connection_string.to_string();
    tokio::task::spawn_blocking(move || {
        let mut conn = PgConnection::establish(&conn_str)?;
        conn.run_pending_migrations(MIGRATIONS)?;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await?
}
