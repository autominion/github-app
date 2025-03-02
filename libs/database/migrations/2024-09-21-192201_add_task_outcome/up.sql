create type task_failure_reason as enum (
    'technical_issues',
    'task_issues',
    'problem_solving'
);

alter table tasks
add column completion_description text,
add column failure_description text,
add column failure_reason task_failure_reason;
