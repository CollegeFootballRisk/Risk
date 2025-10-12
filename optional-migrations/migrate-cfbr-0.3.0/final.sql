-- To be run immediately before 3.0
delete from turns where turn_id = 117;
drop view public.team_player_moves;

CREATE VIEW public.team_player_moves AS
 SELECT past_turns.id,
    turninfo.season,
    turninfo.day,
    teams.tname AS team,
    users.uname AS player,
    past_turns.stars,
    past_turns.mvp,
    territories.name AS territory,
    t2.tname AS regularteam,
    past_turns.weight,
    past_turns.power,
    past_turns.multiplier
   FROM ((((public.past_turns
     JOIN public.territories ON ((territories.id = past_turns.territory)))
     JOIN public.teams ON ((teams.id = past_turns.team)))
     JOIN public.turninfo ON ((past_turns.turn_id = turninfo.id)))
     left JOIN public.users ON ((users.id = past_turns.user_id)))
     left JOIN public.teams t2 ON ((t2.id = users.current_team))
  ORDER BY territories.name, past_turns.team;


ALTER TABLE public.team_player_moves OWNER TO risk;
update territory_ownership set owner_id = 0 where territory_id = 20 and turn_id in (119, 118);
 update stats set territorycount = 1 where turn_id = 117 and team = 131;
 update stats set territorycount = 1 where turn_id = 118 and team = 131;
 update stats set rank = 1 where turn_id >= 117;
select setval('turninfo_id_seq',119);

insert into territories (id, name, region) values (204, 'Redwood', 9);
update territory_adjacency set adjacent_id = 204, note='Season3Adjust' where id in (583,300,664); update territory_adjacency set territory_id = 204, note='Season3Adjust' where id in (602,601,600);insert into territory_adjacency (territory_id, adjacent_id, min_turn, max_turn, note) values (204,18,118,99999, 'Season3Adjust'), (18,204,118,99999,'Season3Adjust');
insert into territory_adjacency (territory_id, adjacent_id, min_turn, max_turn, note) values (204,98,118,99999, 'Season3Adjust'), (98,204,118,99999,'Season3Adjust');
insert into territory_ownership (territory_id,owner_id,previous_owner_id,random_number,turn_id) values (204,0,0,0,117),(204,0,0,0,118);
insert into territory_stats (team,ones,twos,threes,fours,fives,teampower,chance,territory,territory_power,turn_id) values (0,0,0,0,0,0,0,1,204,0,117);
insert into territory_stats (team,ones,twos,threes,fours,fives,teampower,chance,territory,territory_power,turn_id) values (0,0,0,0,0,0,0,1,204,0,118);

insert into stats (team,rank,territorycount,playercount,merccount,starpower,efficiency,effectivepower,ones,twos,threes,fours,fives,turn_id) select team,rank,territorycount,playercount,merccount,starpower,efficiency,effectivepower,ones,twos,threes,fours,fives,117 from stats where turn_id=116;
delete from stats where turn_id = 117 and playercount = 0;
update territory_adjacency set adjacent_id = 204 where adjacent_id = 98 and territory_id = 40;update territory_adjacency set territory_id = 204 where territory_id = 98 and adjacent_id = 40;


alter table turninfo drop constraint turninfo_pkey;update turninfo set id = id+1 where id > 66; alter table turninfo add constraint turninfo_pkey PRIMARY KEY (id);
select setval('turninfo_id_seq', (select max(id) from turninfo));
insert into turninfo (id,season,day,complete,active,finale,allornothingenabled) values (67,1,67,true,false,true,false);
update stats set turn_id = turn_id + 1 where turn_id >66;
update territory_adjacency set min_turn = min_turn + 1 where min_turn > 66;
update territory_adjacency set max_turn = max_turn + 1 where max_turn > 66;
update territory_ownership set turn_id = turn_id + 1 where turn_id > 66;
insert into territory_ownership (territory_id,owner_id,previous_owner_id,random_number,timestamp,mvp,turn_id) select territory_id,owner_id,previous_owner_id,random_number,timestamp,mvp,67 from territory_ownership where turn_id = 68;
update territory_stats set turn_id = turn_id + 1, id=id+132 where turn_id > 66;
insert into territory_stats (team, ones,twos,threes,fours,fives,teampower,chance,id,territory,territory_power,turn_id) select team, ones,twos,threes,fours,fives,teampower,chance,63174+territory,territory, territory_power, 67  from territory_stats where turn_id = 117 and territory != 204;
select setval('territory_stats_id_seq', (select max(id) from territory_stats));
alter table turns drop constraint turns_user_id_season_day_key;
update turns set turn_id = turn_id + 1 where turn_id >66;
alter table turns add unique(user_id, turn_id);
update user_info set turn_id = turn_id + 1 where turn_id >66;
update turninfo set day=0,season=2 where id = 67;
update stats set turn_id = 67 where turn_id = 66; insert into stats (team,rank,territorycount,playercount,merccount,starpower,efficiency,effectivepower,ones,twos,threes,fours,fives,turn_id) select team,rank,territorycount,playercount,merccount,starpower,efficiency,effectivepower,ones,twos,threes,fours,fives,turn_id+1 from stats where turn_id = 65;

update territory_adjacency set min_turn = min_turn + 1 where min_turn = 119 and territory_id = 98;
update territories set name = 'Redwood' where id = 98;
update territories set name = 'Palo Alto' where id = 204;
update territory_ownership set owner_id = 125 where territory_id = 204 and turn_id in (119, 120);
update territory_ownership set owner_id = 0 where territory_id = 98 and turn_id in (119,120);
update territory_adjacency set min_turn = 119 where min_turn = 120;
update territory_adjacency set min_turn = 120 where territory_id = 204 and min_turn = 119;