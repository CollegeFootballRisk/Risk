-- This file should undo anything in `up.sql`
create schema tempest;
alter table if exists public.__diesel_schema_migrations set schema tempest;
drop schema public cascade;
create schema public;
alter table if exists tempest.__diesel_schema_migrations set schema public;
drop schema tempest cascade;
