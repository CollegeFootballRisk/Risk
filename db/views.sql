drop view heat cascade;
drop view territory_ownership_without_neighbors cascade;
drop view moves cascade;
drop view players cascade;
drop view region_ownership cascade;
drop view statistics;
drop view territory_neighbor_history cascade;

--
-- Name: heat; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.heat AS
 SELECT territories.name,
    rd.season,
    rd.day,
    count(past_turns.territory) AS cumulative_players,
    COALESCE(sum(past_turns.power), (0)::double precision) AS cumulative_power
   FROM ((public.territories
     CROSS JOIN ( SELECT turninfo.id, turninfo.season,
            turninfo.day
           FROM public.turninfo
          WHERE (turninfo.complete = true)) rd)
     LEFT JOIN public.past_turns ON (((rd.id = past_turns.turn_id) AND (territories.id = past_turns.territory))))
  GROUP BY territories.name, rd.season, rd.day
  ORDER BY territories.name, rd.season DESC, rd.day DESC;


ALTER TABLE public.heat OWNER TO risk;

--
-- Name: territory_ownership_without_neighbors; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_ownership_without_neighbors AS
 SELECT territory_ownership.territory_id,
    turninfo.day,
    turninfo.season,
    territories.name,
    teams.tname AS owner,
    tex.tname AS prev_owner,
    territory_ownership."timestamp",
    territory_ownership.random_number,
    users.uname AS mvp
   FROM ((((public.territory_ownership
     LEFT JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     LEFT JOIN public.teams tex ON ((tex.id = territory_ownership.previous_owner_id)))
     LEFT JOIN public.territories ON ((territory_ownership.territory_id = territories.id)))
     LEFT JOIN public.turninfo ON ((territory_ownership.turn_id = turninfo.id))
     LEFT JOIN public.users ON ((users.id = territory_ownership.mvp)))
  ORDER BY turninfo.id DESC;


ALTER TABLE public.territory_ownership_without_neighbors OWNER TO risk;

--
-- Name: heat_full; Type: VIEW; Schema: public; Owner: risk
--

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
     LEFT JOIN public.territory_ownership_without_neighbors ON ((((territory_ownership_without_neighbors.name)::text = (heat.name)::text) AND (territory_ownership_without_neighbors.day = heat.day) AND (territory_ownership_without_neighbors.season = heat.season))));


ALTER TABLE public.heat_full OWNER TO risk;

--
-- Name: moves; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.moves AS
 SELECT turninfo.season,
    turninfo.day,
    past_turns.territory,
    foo.user_id,
    users.current_team AS team,
    past_turns.user_id AS player,
    past_turns.mvp,
    users.uname,
    users.turns,
    users.mvps,
    teams.tname,
    past_turns.power,
    past_turns.weight,
    past_turns.stars
   FROM (((( SELECT max(past_turns_1.id) AS id,
            past_turns_1.user_id
           FROM public.past_turns past_turns_1
          GROUP BY past_turns_1.user_id) foo
     JOIN public.past_turns ON ((past_turns.id = foo.id)))
     LEFT JOIN public.turninfo ON ((turninfo.id = past_turns.turn_id))
     LEFT JOIN public.users ON ((foo.user_id = users.id)))
     LEFT JOIN public.teams ON ((users.current_team = teams.id)))
  ORDER BY users.uname;


ALTER TABLE public.moves OWNER TO risk;

--
-- Name: odds; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.odds AS
 SELECT territory_stats.ones,
    territory_stats.twos,
    territory_stats.threes,
    territory_stats.fours,
    territory_stats.fives,
    ((((territory_stats.ones + territory_stats.twos) + territory_stats.threes) + territory_stats.fours) + territory_stats.fives) AS players,
    territory_stats.teampower,
    territory_stats.territory_power AS territorypower,
    territory_stats.chance,
    territory_stats.team,
    territory_stats.season,
    territory_stats.day,
    territories.name AS territory_name,
    teams.tname AS team_name,
    teams.color_1 AS color,
    teams.color_2 AS secondary_color,
    territory_ownership_without_neighbors.owner AS tname,
    territory_ownership_without_neighbors.prev_owner,
    territory_ownership_without_neighbors.mvp
   FROM (((public.territory_stats
     JOIN public.territories ON ((territories.id = territory_stats.territory)))
     JOIN public.teams ON ((teams.id = territory_stats.team)))
     JOIN public.territory_ownership_without_neighbors ON ((((territory_ownership_without_neighbors.name)::text = (territories.name)::text) AND (territory_ownership_without_neighbors.season = territory_stats.season) AND (territory_ownership_without_neighbors.day = (territory_stats.day + 1)))));


ALTER TABLE public.odds OWNER TO risk;

--
-- Name: players; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.players AS
 SELECT users.id,
    users.uname,
    users.platform,
    users.current_team,
    users.overall,
    users.turns,
    users.game_turns,
    users.mvps,
    users.streak,
    users.awards,
    teams.tname
   FROM (public.users
     JOIN public.teams ON ((teams.id = users.current_team)));


ALTER TABLE public.players OWNER TO risk;

--
-- Name: region_ownership; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.region_ownership AS
 SELECT count(DISTINCT territory_ownership.owner_id) AS owner_count,
    array_agg(DISTINCT territory_ownership.owner_id) AS owners,
    turninfo.day,
    turninfo.season,
    territories.region
   FROM (public.territory_ownership
     LEFT JOIN public.territories ON ((territory_ownership.territory_id = territories.id)) LEFT JOIN public.turninfo ON ((turninfo.id = territory_ownership.turn_id)))
  GROUP BY turninfo.day, turninfo.season, territories.region
  ORDER BY turninfo.season DESC, turninfo.day DESC;


ALTER TABLE public.region_ownership OWNER TO risk;

--
-- Name: rollinfo; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.rollinfo AS
 SELECT (turninfo.rollstarttime)::text AS rollstarttime,
    (turninfo.rollendtime)::text AS rollendtime,
    turninfo.chaosrerolls,
    turninfo.chaosweight,
    (territory_ownership_without_neighbors.day - 1) AS day,
    territory_ownership_without_neighbors.season,
    json_agg(json_build_object('territory', territory_ownership_without_neighbors.name, 'timestamp', territory_ownership_without_neighbors."timestamp", 'winner', territory_ownership_without_neighbors.owner, 'randomNumber', territory_ownership_without_neighbors.random_number)) AS json_agg
   FROM (public.territory_ownership_without_neighbors
     JOIN public.turninfo ON (((turninfo.day = (territory_ownership_without_neighbors.day - 1)) AND (turninfo.season = territory_ownership_without_neighbors.season))))
  GROUP BY territory_ownership_without_neighbors.day, territory_ownership_without_neighbors.season, turninfo.chaosrerolls, turninfo.rollstarttime, turninfo.rollendtime, turninfo.chaosweight;


ALTER TABLE public.rollinfo OWNER TO risk;

--
-- Name: statistics; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.statistics AS
 SELECT stats.sequence,
    turninfo.season,
    turninfo.day,
    stats.team,
    stats.rank,
    stats.territorycount,
    stats.playercount,
    stats.merccount,
    stats.starpower,
    stats.efficiency,
    stats.effectivepower,
    stats.ones,
    stats.twos,
    stats.threes,
    stats.fours,
    stats.fives,
    teams.tname,
    teams.logo
   FROM (public.stats
     JOIN public.teams ON ((teams.id = stats.team)) JOIN public.turninfo ON ((turninfo.id = stats.turn_id)));


ALTER TABLE public.statistics OWNER TO risk;

--
-- Name: team_player_moves; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.team_player_moves AS
 SELECT past_turns.id,
    turninfo.season,
    turninfo.day,
    teams.tname AS team,
    players.uname AS player,
    past_turns.stars,
    past_turns.mvp,
    territories.name AS territory,
    players.tname AS regularteam,
    past_turns.weight,
    past_turns.power,
    past_turns.multiplier
   FROM (((public.past_turns
     JOIN public.territories ON ((territories.id = past_turns.territory)))
     JOIN public.teams ON ((teams.id = past_turns.team)))
     JOIN public.turninfo ON ((past_turns.turn_id = turninfo.id))
     JOIN public.players ON ((players.id = past_turns.user_id)))
  ORDER BY territories.name, past_turns.team;


ALTER TABLE public.team_player_moves OWNER TO risk;

--
-- Name: territory_neighbor_history; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_neighbor_history AS
 SELECT territory_ownership.turn_id AS turn_id,
    territory_adjacency.territory_id AS id,
    json_agg(json_build_object('id', territory_ownership.territory_id, 'name', territories.name, 'shortName', territories.name, 'owner', teams.tname)) AS neighbors
   FROM (((public.territory_adjacency
     JOIN public.territory_ownership ON ((territory_ownership.territory_id = territory_adjacency.adjacent_id)))
     JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     JOIN public.territories ON ((territories.id = territory_ownership.territory_id)))
  WHERE (territory_adjacency.territory_id <> territory_adjacency.adjacent_id)
  GROUP BY territory_adjacency.territory_id, territory_ownership.turn_id
  ORDER BY territory_adjacency.territory_id;


ALTER TABLE public.territory_neighbor_history OWNER TO risk;

--
-- Name: territory_ownership_with_neighbors; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_ownership_with_neighbors AS
 SELECT territory_ownership.territory_id,
    turninfo.day,
    turninfo.season,
    territories.name,
    teams.tname,
    territory_neighbor_history.neighbors
   FROM (((public.territory_ownership
     JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     JOIN public.territories ON ((territory_ownership.territory_id = territories.id)))
     JOIN public.territory_neighbor_history ON ((territory_ownership.territory_id = territory_neighbor_history.id))
     JOIN public.turninfo ON ((territory_ownership.turn_id = turninfo.id)))
  WHERE ((turninfo.id = territory_neighbor_history.turn_id));


ALTER TABLE public.territory_ownership_with_neighbors OWNER TO risk;
