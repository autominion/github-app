use actix_web::Scope;

pub mod task;

use task::*;

pub fn scope() -> Scope {
    Scope::new("/agent").service(task_info).service(task_complete).service(task_fail)
}
