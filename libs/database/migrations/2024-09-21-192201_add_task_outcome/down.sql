alter table tasks
drop column completion_description,
drop column failure_description,
drop column failure_reason;

drop type task_failure_reason;
