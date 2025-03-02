use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::conn::Conn;
use crate::models::repositories::Repository;
use crate::models::tasks::{NewTask, Task, UpdateTask};
use crate::schema::tasks::dsl::*;
use crate::types::{TaskFailureReason, TaskStatus};

impl Conn<'_> {
    pub async fn get_tasks_for_user(&mut self, user_id: &Uuid) -> Vec<(Task, Repository)> {
        tasks
            .filter(created_by_id.eq(user_id))
            .inner_join(crate::schema::repositories::table)
            .order_by(created_at.desc())
            .load(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn user_id_that_created_task(&mut self, task_id: &Uuid) -> Uuid {
        tasks.select(created_by_id).filter(id.eq(task_id)).get_result(&mut self.conn).await.unwrap()
    }

    pub async fn add_task(&mut self, new_task: NewTask) -> Task {
        diesel::insert_into(tasks).values(new_task).get_result(&mut self.conn).await.unwrap()
    }

    pub async fn update_task(&mut self, update: UpdateTask) -> Task {
        diesel::update(&update).set(&update).get_result(&mut self.conn).await.unwrap()
    }

    pub async fn get_task_status(&mut self, task_id: &Uuid) -> TaskStatus {
        tasks.select(status).filter(id.eq(task_id)).get_result(&mut self.conn).await.unwrap()
    }

    pub async fn get_task(&mut self, task_id: &Uuid) -> Task {
        tasks.find(task_id).get_result(&mut self.conn).await.unwrap()
    }

    pub async fn get_task_and_repository(&mut self, task_id: &Uuid) -> (Task, Repository) {
        tasks
            .filter(id.eq(task_id))
            .inner_join(crate::schema::repositories::table)
            .first(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn receive_task(&mut self) -> Option<Task> {
        let tasks1 = diesel::alias!(crate::schema::tasks as tasks1);

        let select = tasks1
            .for_update()
            .skip_locked()
            .filter(tasks1.field(status).eq(TaskStatus::Queued))
            .order_by(tasks1.field(created_at))
            .limit(1)
            .select(tasks1.field(id));

        diesel::update(tasks)
            .filter(id.eq_any(select))
            .set(status.eq(TaskStatus::Running))
            .get_result(&mut self.conn)
            .await
            .ok()
    }

    pub async fn complete_task(&mut self, task_id: &Uuid, description: &str) -> Task {
        diesel::update(tasks)
            .filter(id.eq(task_id))
            .set((status.eq(TaskStatus::Completed), completion_description.eq(Some(description))))
            .get_result(&mut self.conn)
            .await
            .unwrap()
    }

    pub async fn fail_task(
        &mut self,
        task_id: &Uuid,
        reason: Option<TaskFailureReason>,
        description: &str,
    ) -> Task {
        diesel::update(tasks)
            .filter(id.eq(task_id))
            .set((
                status.eq(TaskStatus::Failed),
                failure_reason.eq(reason),
                failure_description.eq(Some(description)),
            ))
            .get_result(&mut self.conn)
            .await
            .unwrap()
    }
}
