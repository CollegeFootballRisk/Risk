-- This file should undo anything in `up.sql`
delete from user_role where role = (select id from role where name = 'Moderator'));
