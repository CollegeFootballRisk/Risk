
alter table territory_ownership add column turn_id integer;
update territory_ownership set turn_id = turninfo.id from turninfo where turninfo.day = territory_ownership.day and turninfo.season = territory_ownership.season;

alter table continuation_polls add column turn_id integer;
update continuation_polls c set turn_id = t.id from turninfo t where t.day = c.day and c.season = t.season;

alter table new_turns add column turn_id integer;
update new_turns c set turn_id = t.id from turninfo t where t.day = c.day and c.season = t.season;

alter table past_turns add column turn_id integer;
update past_turns c set turn_id = t.id from turninfo t where t.day = c.day and c.season = t.season;

alter table stats add column turn_id integer;
update stats c set turn_id = t.id from turninfo t where t.day = c.day and c.season = t.season;

alter table territory_stats add column turn_id integer;
update territory_stats c set turn_id = t.id from turninfo t where t.day = c.day and c.season = t.season;

drop view heat_full;

drop view heat;

CREATE VIEW public.heat AS
 SELECT territories.name,
    rd.season,
    rd.day,
    count(past_turns.territory) AS cumulative_players,
    COALESCE(sum(past_turns.power), (0)::double precision) AS cumulative_power
   FROM ((public.territories
     CROSS JOIN ( SELECT turninfo.id, turninfo.season, turninfo.day FROM public.turninfo
          WHERE (turninfo.complete = true)) rd)
     LEFT JOIN public.past_turns ON (((rd.id = past_turns.turn_id) AND (territories.id = past_turns.territory))))
  GROUP BY territories.name, rd.season, rd.day
  ORDER BY territories.name, rd.season DESC, rd.day DESC;

drop view territory_ownership_with_neighbors;
drop view rollinfo;
drop view territory_ownership_without_neighbors;




CREATE VIEW public.rollinfo AS
 SELECT (turninfo.rollstarttime)::text AS rollstarttime,
    (turninfo.rollendtime)::text AS rollendtime,
    turninfo.chaosrerolls,
    turninfo.chaosweight,
    (territory_ownership_without_neighbors.day - 1) AS day,
    territory_ownership_without_neighbors.season,
    json_agg(json_build_object('territory', territory_ownership_without_neighbors.name, 'timestamp', territory_owners
hip_without_neighbors."timestamp", 'winner', territory_ownership_without_neighbors.owner, 'randomNumber', territory_o
wnership_without_neighbors.random_number)) AS json_agg
   FROM (public.territory_ownership_without_neighbors
     JOIN public.turninfo ON (((turninfo.day = (territory_ownership_without_neighbors.day - 1)) AND (turninfo.season
= territory_ownership_without_neighbors.season))))
  GROUP BY territory_ownership_without_neighbors.day, territory_ownership_without_neighbors.season, turninfo.chaosrer
olls, turninfo.rollstarttime, turninfo.rollendtime, turninfo.chaosweight;

CREATE VIEW public.heat_full AS
 SELECT heat.name,
    heat.season,
    heat.day,
    heat.cumulative_players,
    heat.cumulative_power,
        CASE
            WHEN (territory_ownership_without_neighbors.owner IS NULL) THEN 'None'::public.citext
            ELSE territory_ownership_without_neighbors.owner
        END AS owner
   FROM (public.heat
     LEFT JOIN public.territory_ownership_without_neighbors ON ((((territory_ownership_without_neighbors.name)::text
= (heat.name)::text) AND (territory_ownership_without_neighbors.day = heat.day) AND (territory_ownership_without_neighbors.season = heat.season))));



alter table stats drop column sequence;
alter table stats drop column day;
alter table stats drop column season;
alter table territory_ownership drop column day; 
alter table territory_ownership drop column season;
alter table continuation_polls drop column day;
alter table continuation_polls drop column season;
alter table new_turns drop column day;
alter table new_turns drop column season;
alter table past_turns drop column day;
alter table past_turns drop column season;
alter table territory_stats drop column season;
alter table territory_stats drop column day;

