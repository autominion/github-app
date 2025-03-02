use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};
use std::io::Write;

#[derive(Debug, Clone, Copy, AsExpression, FromSqlRow, PartialEq, Eq)]
#[diesel(sql_type = crate::schema::sql_types::TaskFailureReason)]
pub enum TaskFailureReason {
    TechnicalIssues,
    TaskIssues,
    ProblemSolving,
}

impl ToSql<crate::schema::sql_types::TaskFailureReason, Pg> for TaskFailureReason {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let value = match *self {
            TaskFailureReason::TechnicalIssues => "technical_issues",
            TaskFailureReason::TaskIssues => "task_issues",
            TaskFailureReason::ProblemSolving => "problem_solving",
        };
        out.write_all(value.as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<crate::schema::sql_types::TaskFailureReason, Pg> for TaskFailureReason {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"technical_issues" => Ok(TaskFailureReason::TechnicalIssues),
            b"task_issues" => Ok(TaskFailureReason::TaskIssues),
            b"problem_solving" => Ok(TaskFailureReason::ProblemSolving),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl From<agent_api::types::task::TaskFailureReason> for TaskFailureReason {
    fn from(value: agent_api::types::task::TaskFailureReason) -> Self {
        match value {
            agent_api::types::task::TaskFailureReason::TechnicalIssues => {
                TaskFailureReason::TechnicalIssues
            }
            agent_api::types::task::TaskFailureReason::TaskIssues => TaskFailureReason::TaskIssues,
            agent_api::types::task::TaskFailureReason::ProblemSolving => {
                TaskFailureReason::ProblemSolving
            }
        }
    }
}
