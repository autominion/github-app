use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;

#[derive(Debug, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::TaskStatus)]
pub enum TaskStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

impl ToSql<crate::schema::sql_types::TaskStatus, Pg> for TaskStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        use TaskStatus::*;
        match *self {
            Queued => out.write_all(b"queued")?,
            Running => out.write_all(b"running")?,
            Completed => out.write_all(b"completed")?,
            Failed => out.write_all(b"failed")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::TaskStatus, Pg> for TaskStatus {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        use TaskStatus::*;
        match bytes.as_bytes() {
            b"queued" => Ok(Queued),
            b"running" => Ok(Running),
            b"completed" => Ok(Completed),
            b"failed" => Ok(Failed),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<agent_api::types::task::TaskStatus> for TaskStatus {
    fn into(self) -> agent_api::types::task::TaskStatus {
        use TaskStatus::*;
        match self {
            Queued => agent_api::types::task::TaskStatus::Queued,
            Running => agent_api::types::task::TaskStatus::Running,
            Completed => agent_api::types::task::TaskStatus::Completed,
            Failed => agent_api::types::task::TaskStatus::Failed,
        }
    }
}
