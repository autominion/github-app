alter table repositories drop column default_agent_config_id;
alter table tasks drop column agent_config_id;
drop table agent_configs;
