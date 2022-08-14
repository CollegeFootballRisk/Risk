alter table turninfo add column map text;
alter table turninfo add column allornothingenabled boolean;
update turninfo set allornothingenabled = false;
create table award_info ( id serial, name text, info text);
create table awards (id serial, user_id integer, award_id integer, award_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP);
insert into award_info (name, info) values ('Developer', 'A developer of College Football Risk');
insert into award_info (name, info) values ('Moderator', 'A game moderator of College Football Risk');
insert into award_info (name, info) values ('1', 'A participant in Season 1');
insert into award_info (name, info) values ('2', 'A participant in Season 2');
insert into award_info (name, info) values ('2.1', 'A participant in Grocery Store Risk');
-- Give Developer to Bakony, BlueSCar, Mau
insert into awards (user_id, award_id) values (1,1), (671,1), (25158,1);
-- Give Moderator to bobsled, reptar, rogue, capn, crosely, semi, wes, inspector
insert into awards (user_id, award_id) values (1054,2), (5472,2), (665,2), (1584,2), (856,2), (25158,2), (6257,2), (6820,2), (18300,2);
-- Give Developer to Jaket, Epic, Mojo, Mango, Luro, Iron, Capn, Dys
insert into awards (user_id, award_id) values (5258,1), (16785,1),(11828,1), (24309,1),(27640,1), (22345,1),(1584,1), (21676,1);
-- Give season 1 to those who played
insert into awards (user_id, award_id) select distinct(user_id) user_id, 3 award_id from past_turns where turn_id < 67;
-- Give season 2 to those who played
insert into awards (user_id, award_id) select distinct(user_id) user_id, 4 award_id from past_turns where turn_id >= 67;
