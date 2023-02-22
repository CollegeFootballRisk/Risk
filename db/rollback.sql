delete from stats where turn_id = 144;
delete from territory_stats where turn_id = 144;
delete from territory_ownership where turn_id = 145;
delete from territory_adjacency where max_turn = 145;
delete from turns where turn_id = 145;
delete from turninfo where id = 145;
update turninfo set complete = false, active=false, rollstarttime='2023-02-21T03:30:00Z' where id = 144;
update users set streak = 0, mvps=0, game_turns=0, turns =0;
update turns set mvp = false where turn_id = 144;
update users set turns=sub.turns from (select count(*) turns,user_id from turns where turn_id < 144 group by user_id) sub where sub.user_id = users.id;
update users set game_turns=sub.turns from (select count(*) turns,user_id from turns where turn_id > 119 and turn_id < 144 group by user_id) sub where sub.user_id = users.id;
update users set mvps = x.mvps from (select user_id,mvps,d.count from (select user_id, count(*) from turns where mvp = true group by user_id) d inner join users on users.id=d.user_id where d.count = mvps) x where x.user_id = users.id;
update users set mvps = x.count from (select user_id, count(*) from turns where mvp = true group by user_id) x where x.user_id = users.id;
update users set streak=x.streak from (select user_id, streak streak from (with move_made as (select distinct user_id, turn_id, RANK() OVER(PARTITION BY user_id ORDER BY turn_id) rank FROM turns where turn_id < 144), streak AS (select *, turn_id - rank date_group FROM move_made), output AS (select DISTINCT user_id, date_group, count(*) streak, min(turn_id) started_on, max(turn_id) ended_on FROM streak group by 1,2) select * FROM output where STREAK >= 1) d where d.ended_on = 143) x where x.user_id=users.id;
   UPDATE users SET overall = overall.overall FROM (select id
    , median(power) as overall
from (
    select id
        , case
            when mvps >= 25 then 5
            when mvps >= 10 then 4
            when mvps >= 5 then 3
            when mvps >= 1 then 2
            else 1 end as power
    from users
    union all
    select id
        , case
            when turns >= 100 then 5
            when turns >= 50 then 4
            when turns >= 25 then 3
            when turns >= 10 then 2
            else 1 end as power
    from users
    union all
    select id
        , case
            when game_turns >= 40 then 5
            when game_turns >= 25 then 4
            when game_turns >= 10 then 3
            when game_turns >= 5 then 2
            else 1 end as power
    from users
    union all
    select id
        , case
            when awards >= 4 then 5
            when awards >= 3 then 4
            when awards >= 2 then 3
            when awards >= 1 then 2
            else 1 end as power
    from users
    union all
    select id
        , case
            when streak >= 25 then 5
            when streak >= 10 then 4
            when streak >= 5 then 3
            when streak >= 3 then 2
            else 1 end as power
    from users
    ) t
group by 1) as overall where overall.id= users.id;

update users set playing_for = 61 where current_team = 61;

-- make sure to update users overall
-- update sequences
 SELECT setval('audit_log_id_seq', (SELECT MAX(id) FROM audit_log));
 SELECT setval('award_info_id_seq', (SELECT MAX(id) FROM award_info));
 SELECT setval('awards_id_seq', (SELECT MAX(id) FROM awards));
SELECT setval('bans_id_seq', (SELECT MAX(id) FROM bans));
 SELECT setval('captchas_id_seq', (SELECT MAX(id) FROM captchas));
 SELECT setval('continuation_polls_id_seq', (SELECT MAX(id) FROM continuation_polls));
 SELECT setval('continuation_responses_id_seq', (SELECT MAX(id) FROM continuation_responses));
 SELECT setval('logs_id_seq', (SELECT MAX(id) FROM logs));
 SELECT setval('regions_id_seq', (SELECT MAX(id) FROM regions));
SELECT setval('teams_id_seq', (SELECT MAX(id) FROM teams));
SELECT setval('territories_seq', (SELECT MAX(id) FROM territories));
SELECT setval('territory_adjacency_id_seq', (SELECT MAX(id) FROM territory_adjacency));
SELECT setval('territory_ownership_id_seq', (SELECT MAX(id) FROM territory_ownership));
SELECT setval('territory_stats_id_seq', (SELECT MAX(id) FROM territory_stats));
 SELECT setval('turninfo_id_seq', (SELECT MAX(id) FROM turninfo));
 SELECT setval('user_info_id_seq', (SELECT MAX(id) FROM user_info));
 SELECT setval('users_id_seq', (SELECT MAX(id) FROM users));