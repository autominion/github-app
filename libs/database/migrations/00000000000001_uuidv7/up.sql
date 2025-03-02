/*  Generate a UUIDv7 with sub-milliseconds precision
    We use this SQL function until builtin support for UUIDv7 generation
    lands in Postgres 18.
    Source: https://postgresql.verite.pro/blog/2024/07/15/uuid-v7-pure-sql.html
 */
create function uuidv7() returns uuid
as $$
 select encode(
   substring(int8send(floor(t_ms)::int8) from 3) ||
   int2send((7<<12)::int2 | ((t_ms-floor(t_ms))*4096)::int2) ||
   substring(uuid_send(gen_random_uuid()) from 9 for 8)
  , 'hex')::uuid
  from (select extract(epoch from clock_timestamp())*1000 as t_ms) s
$$ language sql volatile;
