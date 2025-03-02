use chrono::{DateTime, Utc};
use diesel::result::Error;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::task_compute_usage::{NewTaskComputeUsage, TaskComputeUsage};
use crate::schema::task_compute_usage::dsl as usage_dsl;
use crate::Conn;

impl Conn<'_> {
    /// Start a compute usage record for a task
    pub async fn start_compute_usage(
        &mut self,
        task_id: Uuid,
        usage_start: DateTime<Utc>,
    ) -> Result<TaskComputeUsage, Error> {
        let new_usage = NewTaskComputeUsage {
            task_id,
            compute_usage_start_timestamp: usage_start,
            compute_usage_end_timestamp: None,
        };

        diesel::insert_into(usage_dsl::task_compute_usage)
            .values(&new_usage)
            .get_result::<TaskComputeUsage>(&mut self.conn)
            .await
    }

    /// End a compute usage record for a task
    pub async fn end_compute_usage(
        &mut self,
        usage_id: Uuid,
        usage_end: DateTime<Utc>,
    ) -> Result<TaskComputeUsage, Error> {
        diesel::update(usage_dsl::task_compute_usage.find(usage_id))
            .set(usage_dsl::compute_usage_end_timestamp.eq(Some(usage_end)))
            .get_result::<TaskComputeUsage>(&mut self.conn)
            .await
    }
}
