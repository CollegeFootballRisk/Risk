CREATE OR REPLACE FUNCTION do_user_update(day integer, season integer)
RETURNS BOOLEAN LANGUAGE plpgsql SECURITY DEFINER AS $$
BEGIN
    UPDATE users SET streak = streak+1 WHERE id in (SELECT user_id FROM past_turns WHERE past_turns.day = do_user_update.day AND past_turns.season = do_user_update.season);
    UPDATE users SET streak = 0 WHERE id NOT IN (SELECT user_id FROM past_turns WHERE past_turns.day = do_user_update.day AND past_turns.season = do_user_update.season);
    UPDATE users SET mvps = mvps.mvps, turns = mvps.turns FROM (SELECT user_id, SUM(case when mvp=true THEN 1 ELSE 0 END) as mvps, count(*) as turns FROM past_turns GROUP BY user_id) as mvps WHERE mvps.user_id = users.id;
    UPDATE users SET game_turns = game_turns.game_turns FROM (SELECT user_id, count(*) as game_turns FROM past_turns WHERE past_turns.season = do_user_update.season GROUP BY user_id) as game_turns WHERE game_turns.user_id = users.id;
    UPDATE users SET overall = overall.overall FROM (SELECT id, 
        _final_median(array[
            (case 
            when mvps >= 25 then 5
            when mvps >=10 then 4
            when mvps>=5 THEN 3
            when mvps >= 1 THEN 2
            when mvps = 0 THEN 1
            else 0 end),
            (case 
            when turns >= 100 then 5 
            when turns >= 50 then 4 
            when turns >= 25 then 3 
            when turns >= 10 then 2 
            when turns >= 0 then 1 
            else 0 end),
            (case
            when game_turns >= 40 then 5
            when game_turns >= 25 then 4
            when game_turns >= 10 then 3
            when game_turns >= 5 then 2
            when game_turns >= 0 then 1
            else 0 end),
            (case
            when awards >= 4 then 5
            when awards >= 3 then 4
            when awards >= 2 then 3
            when awards >= 1 then 2
            when awards >= 0 then 1
            else 0 end),
            (case
            when streak >= 25 then 5
            when streak >= 10 then 4
            when streak >= 5 then 3
            when streak >= 3 then 2
            when streak >= 0 then 1
            else 0 end)
            ]) as overall FROM users GROUP BY id) as overall where overall.id= users.id;
    return FOUND;
    END;
    $$;

SELECT user_id, SUM(case when mvp=true THEN 1 ELSE 0 END) FROM past_turns GROUP_BY user_id;

CREATE OR REPLACE FUNCTION _final_median(numeric[])
   RETURNS numeric AS
$$
   SELECT AVG(val)
   FROM (
     SELECT val
     FROM unnest($1) val
     ORDER BY 1
     LIMIT  2 - MOD(array_upper($1, 1), 2)
     OFFSET CEIL(array_upper($1, 1) / 2.0) - 1
   ) sub;
$$
LANGUAGE 'sql' IMMUTABLE;

CREATE AGGREGATE median(numeric) (
  SFUNC=array_append,
  STYPE=numeric[],
  FINALFUNC=_final_median,
  INITCOND='{}'
);