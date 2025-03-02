use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;

#[derive(Clone, Copy, Debug, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::UserRole)]
pub enum UserRole {
    Admin,
    Member,
}

impl ToSql<crate::schema::sql_types::UserRole, Pg> for UserRole {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        match self {
            UserRole::Admin => out.write_all(b"admin")?,
            UserRole::Member => out.write_all(b"member")?,
        }
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::UserRole, Pg> for UserRole {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"admin" => Ok(UserRole::Admin),
            b"member" => Ok(UserRole::Member),
            _ => Err("Unrecognized enum variant for UserRole".into()),
        }
    }
}
