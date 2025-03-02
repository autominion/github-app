use chrono::{DateTime, Utc};
use diesel::{Identifiable, Insertable, Queryable};
use uuid::Uuid;

use crate::schema::task_compute_usage;

#[derive(Debug, Queryable, Identifiable)]
#[diesel(table_name = task_compute_usage)]
pub struct TaskComputeUsage {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub task_id: Uuid,
    pub compute_usage_start_timestamp: DateTime<Utc>,
    pub compute_usage_end_timestamp: Option<DateTime<Utc>>,
}

#[derive(Insertable)]
#[diesel(table_name = task_compute_usage)]
pub struct NewTaskComputeUsage {
    pub task_id: Uuid,
    pub compute_usage_start_timestamp: DateTime<Utc>,
    pub compute_usage_end_timestamp: Option<DateTime<Utc>>,
}
