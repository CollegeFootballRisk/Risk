--
-- PostgreSQL database dump
--

-- Dumped from database version 14.5
-- Dumped by pg_dump version 14.5

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: citext; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS citext WITH SCHEMA public;


--
-- Name: EXTENSION citext; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION citext IS 'data type for case-insensitive character strings';


--
-- Name: pgcrypto; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;


--
-- Name: EXTENSION pgcrypto; Type: COMMENT; Schema: -; Owner: 
--

COMMENT ON EXTENSION pgcrypto IS 'cryptographic functions';


--
-- Name: rr_event; Type: TYPE; Schema: public; Owner: risk
--

CREATE TYPE public.rr_event AS ENUM (
    'notification',
    'change_team'
);


ALTER TYPE public.rr_event OWNER TO risk;

--
-- Name: _final_median(numeric[]); Type: FUNCTION; Schema: public; Owner: risk
--

CREATE FUNCTION public._final_median(numeric[]) RETURNS numeric
    LANGUAGE sql IMMUTABLE
    AS $_$
   SELECT AVG(val)
   FROM (
     SELECT val
     FROM unnest($1) val
     ORDER BY 1
     LIMIT  2 - MOD(array_upper($1, 1), 2)
     OFFSET CEIL(array_upper($1, 1) / 2.0) - 1
   ) sub;
$_$;


ALTER FUNCTION public._final_median(numeric[]) OWNER TO risk;

--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: risk
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO risk;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: risk
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO risk;

--
-- Name: do_user_update(integer, integer); Type: FUNCTION; Schema: public; Owner: risk
--

CREATE FUNCTION public.do_user_update(turn integer, season integer) RETURNS boolean
    LANGUAGE plpgsql SECURITY DEFINER
    AS $$
BEGIN
    UPDATE users SET streak = streak+1 WHERE id in (SELECT user_id FROM turns WHERE turns.turn_id = do_user_update.turn);
    UPDATE users SET streak = 0 WHERE id NOT IN (SELECT user_id FROM turns WHERE turns.turn_id = do_user_update.turn);
    UPDATE users SET mvps = mvps.mvps, turns = mvps.turnsz FROM (SELECT user_id, SUM(case when mvp=true THEN 1 ELSE 0 END) as mvps, count(*) as turnsz FROM turns GROUP BY user_id) as mvps WHERE mvps.user_id = users.id;
    UPDATE users SET game_turns = game_turns.game_turns FROM (SELECT user_id, count(*) as game_turns FROM turns inner join turninfo on turninfo.id = turns.turn_id WHERE turninfo.season = do_user_update.season GROUP BY user_id) as game_turns WHERE game_turns.user_id = users.id;
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
update users set playing_for = -1 where playing_for not in (select distinct(owner_id) from territory_ownership where territory_ownership.turn_id = do_user_update.turn+1);
    return FOUND;
    END;
    $$;


ALTER FUNCTION public.do_user_update(turn integer, season integer) OWNER TO risk;

--
-- Name: territorypower(integer, integer, integer); Type: FUNCTION; Schema: public; Owner: risk
--

CREATE FUNCTION public.territorypower(integer, integer, integer) RETURNS bigint
    LANGUAGE sql IMMUTABLE STRICT
    AS $_$SELECT sum(power) FROM past_turns WHERE season = $1 AND day = $2 AND territory = $3 limit 1;$_$;


ALTER FUNCTION public.territorypower(integer, integer, integer) OWNER TO risk;

--
-- Name: median(numeric); Type: AGGREGATE; Schema: public; Owner: risk
--

CREATE AGGREGATE public.median(numeric) (
    SFUNC = array_append,
    STYPE = numeric[],
    INITCOND = '{}',
    FINALFUNC = public._final_median
);


ALTER AGGREGATE public.median(numeric) OWNER TO risk;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: audit_log; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.audit_log (
    id integer NOT NULL,
    user_id integer NOT NULL,
    event integer NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    data json
);


ALTER TABLE public.audit_log OWNER TO risk;

--
-- Name: audit_log_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.audit_log_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.audit_log_id_seq OWNER TO risk;

--
-- Name: audit_log_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.audit_log_id_seq OWNED BY public.audit_log.id;


--
-- Name: award_info; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.award_info (
    id integer NOT NULL,
    name text,
    info text
);


ALTER TABLE public.award_info OWNER TO risk;

--
-- Name: award_info_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.award_info_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.award_info_id_seq OWNER TO risk;

--
-- Name: award_info_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.award_info_id_seq OWNED BY public.award_info.id;


--
-- Name: awards; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.awards (
    id integer NOT NULL,
    user_id integer,
    award_id integer,
    award_date timestamp with time zone DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE public.awards OWNER TO risk;

--
-- Name: awards_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.awards_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.awards_id_seq OWNER TO risk;

--
-- Name: awards_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.awards_id_seq OWNED BY public.awards.id;


--
-- Name: captchas; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.captchas (
    id integer NOT NULL,
    title character varying(16),
    content public.citext
);


ALTER TABLE public.captchas OWNER TO risk;

--
-- Name: captchas_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.captchas_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.captchas_id_seq OWNER TO risk;

--
-- Name: captchas_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.captchas_id_seq OWNED BY public.captchas.id;


--
-- Name: continuation_polls; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.continuation_polls (
    id integer NOT NULL,
    question text DEFAULT 'Should this season be extended by seven more days?'::text,
    incrment integer DEFAULT 7,
    turn_id integer
);


ALTER TABLE public.continuation_polls OWNER TO risk;

--
-- Name: continuation_polls_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.continuation_polls_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.continuation_polls_id_seq OWNER TO risk;

--
-- Name: continuation_polls_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.continuation_polls_id_seq OWNED BY public.continuation_polls.id;


--
-- Name: continuation_responses; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.continuation_responses (
    id integer NOT NULL,
    poll_id integer,
    user_id integer,
    response boolean
);


ALTER TABLE public.continuation_responses OWNER TO risk;

--
-- Name: continuation_responses_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.continuation_responses_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.continuation_responses_id_seq OWNER TO risk;

--
-- Name: continuation_responses_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.continuation_responses_id_seq OWNED BY public.continuation_responses.id;


--
-- Name: turninfo; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.turninfo (
    id integer NOT NULL,
    season integer,
    day integer,
    complete boolean,
    active boolean,
    finale boolean,
    chaosrerolls integer DEFAULT 0,
    chaosweight integer DEFAULT 1,
    rollendtime timestamp without time zone,
    rollstarttime timestamp without time zone,
    allornothingenabled boolean,
    map text
);


ALTER TABLE public.turninfo OWNER TO risk;

--
-- Name: turns; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.turns (
    id integer NOT NULL,
    user_id integer,
    territory integer,
    mvp boolean DEFAULT false NOT NULL,
    power double precision,
    multiplier double precision,
    weight double precision,
    stars integer,
    team integer,
    alt_score integer,
    merc boolean DEFAULT false,
    turn_id integer
);


ALTER TABLE public.turns OWNER TO risk;

--
-- Name: past_turns; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.past_turns AS
 SELECT turns.id,
    turns.user_id,
    turns.territory,
    turns.mvp,
    turns.power,
    turns.multiplier,
    turns.weight,
    turns.stars,
    turns.team,
    turns.alt_score,
    turns.merc,
    turns.turn_id
   FROM (public.turns
     JOIN public.turninfo ON ((turninfo.id = turns.turn_id)))
  WHERE (turninfo.complete = true);


ALTER TABLE public.past_turns OWNER TO risk;

--
-- Name: territories_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.territories_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.territories_seq OWNER TO postgres;

--
-- Name: territories; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.territories (
    id integer DEFAULT nextval('public.territories_seq'::regclass) NOT NULL,
    name public.citext,
    region integer
);


ALTER TABLE public.territories OWNER TO risk;

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
     CROSS JOIN ( SELECT turninfo.id,
            turninfo.season,
            turninfo.day
           FROM public.turninfo
          WHERE (turninfo.complete = true)) rd)
     LEFT JOIN public.past_turns ON (((rd.id = past_turns.turn_id) AND (territories.id = past_turns.territory))))
  WHERE (territories.id > 0)
  GROUP BY territories.name, rd.season, rd.day
  ORDER BY territories.name, rd.season DESC, rd.day DESC;


ALTER TABLE public.heat OWNER TO risk;

--
-- Name: teams; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.teams (
    id integer NOT NULL,
    tname public.citext,
    tshortname public.citext,
    creation_date timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    color_1 text,
    color_2 text,
    logo text,
    seasons integer[]
);


ALTER TABLE public.teams OWNER TO risk;

--
-- Name: territory_ownership; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.territory_ownership (
    id integer NOT NULL,
    territory_id integer,
    owner_id integer,
    previous_owner_id integer,
    random_number double precision,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    mvp integer,
    turn_id integer
);


ALTER TABLE public.territory_ownership OWNER TO risk;

--
-- Name: users; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.users (
    id integer NOT NULL,
    uname public.citext NOT NULL,
    platform public.citext NOT NULL,
    join_date timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    current_team integer DEFAULT '-1'::integer NOT NULL,
    auth_key public.citext,
    overall integer DEFAULT 1,
    turns integer DEFAULT 0,
    game_turns integer DEFAULT 0,
    mvps integer DEFAULT 0,
    streak integer DEFAULT 0,
    awards integer DEFAULT 0,
    role_id integer DEFAULT 0,
    playing_for integer DEFAULT '-1'::integer,
    past_teams integer[],
    awards_bak integer,
    discord_id bigint,
    is_alt boolean DEFAULT false
);


ALTER TABLE public.users OWNER TO risk;

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
   FROM (((((public.territory_ownership
     LEFT JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     LEFT JOIN public.teams tex ON ((tex.id = territory_ownership.previous_owner_id)))
     LEFT JOIN public.territories ON ((territory_ownership.territory_id = territories.id)))
     LEFT JOIN public.turninfo ON ((territory_ownership.turn_id = turninfo.id)))
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
-- Name: logs; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.logs (
    id integer NOT NULL,
    route text,
    query text,
    payload text,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);


ALTER TABLE public.logs OWNER TO risk;

--
-- Name: logs_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.logs_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.logs_id_seq OWNER TO risk;

--
-- Name: logs_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.logs_id_seq OWNED BY public.logs.id;


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
   FROM ((((( SELECT max(past_turns_1.id) AS id,
            past_turns_1.user_id
           FROM public.past_turns past_turns_1
          GROUP BY past_turns_1.user_id) foo
     JOIN public.past_turns ON ((past_turns.id = foo.id)))
     LEFT JOIN public.turninfo ON ((turninfo.id = past_turns.turn_id)))
     LEFT JOIN public.users ON ((foo.user_id = users.id)))
     LEFT JOIN public.teams ON ((users.current_team = teams.id)))
  ORDER BY users.uname;


ALTER TABLE public.moves OWNER TO risk;

--
-- Name: territory_stats; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.territory_stats (
    team integer,
    ones integer,
    twos integer,
    threes integer,
    fours integer,
    fives integer,
    teampower double precision,
    chance double precision,
    id integer NOT NULL,
    territory integer,
    territory_power double precision,
    turn_id integer
);


ALTER TABLE public.territory_stats OWNER TO risk;

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
    turninfo.season,
    turninfo.day,
    territories.name AS territory_name,
    teams.tname AS team_name,
    teams.color_1 AS color,
    teams.color_2 AS secondary_color,
    territory_ownership_without_neighbors.owner AS tname,
    territory_ownership_without_neighbors.prev_owner,
    territory_ownership_without_neighbors.mvp
   FROM ((((public.territory_stats
     JOIN public.territories ON ((territories.id = territory_stats.territory)))
     JOIN public.teams ON ((teams.id = territory_stats.team)))
     JOIN public.turninfo ON ((turninfo.id = territory_stats.turn_id)))
     JOIN public.territory_ownership_without_neighbors ON ((((territory_ownership_without_neighbors.name)::text = (territories.name)::text) AND (territory_ownership_without_neighbors.season = turninfo.season) AND (territory_ownership_without_neighbors.day = (turninfo.day + 1)))));


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
   FROM ((public.territory_ownership
     LEFT JOIN public.territories ON ((territory_ownership.territory_id = territories.id)))
     LEFT JOIN public.turninfo ON ((turninfo.id = territory_ownership.turn_id)))
  GROUP BY turninfo.day, turninfo.season, territories.region
  ORDER BY turninfo.season DESC, turninfo.day DESC;


ALTER TABLE public.region_ownership OWNER TO risk;

--
-- Name: regions; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.regions (
    id integer NOT NULL,
    name public.citext
);


ALTER TABLE public.regions OWNER TO risk;

--
-- Name: regions_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.regions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.regions_id_seq OWNER TO risk;

--
-- Name: regions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.regions_id_seq OWNED BY public.regions.id;


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
-- Name: stats; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.stats (
    team integer,
    rank integer,
    territorycount integer,
    playercount integer,
    merccount integer,
    starpower double precision,
    efficiency double precision,
    effectivepower double precision,
    ones integer,
    twos integer,
    threes integer,
    fours integer,
    fives integer,
    turn_id integer
);


ALTER TABLE public.stats OWNER TO risk;

--
-- Name: statistics; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.statistics AS
 SELECT stats.turn_id,
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
   FROM ((public.stats
     JOIN public.teams ON ((teams.id = stats.team)))
     JOIN public.turninfo ON ((turninfo.id = stats.turn_id)));


ALTER TABLE public.statistics OWNER TO risk;

--
-- Name: team_player_moves; Type: VIEW; Schema: public; Owner: risk
--

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

--
-- Name: teams_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.teams_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.teams_id_seq OWNER TO risk;

--
-- Name: teams_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.teams_id_seq OWNED BY public.teams.id;


--
-- Name: temp_moves; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.temp_moves (
    id integer,
    user_id integer,
    target integer
);


ALTER TABLE public.temp_moves OWNER TO postgres;

--
-- Name: territory_adjacency; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.territory_adjacency (
    id integer,
    territory_id integer,
    adjacent_id integer,
    note text,
    min_turn integer,
    max_turn integer
);


ALTER TABLE public.territory_adjacency OWNER TO risk;

--
-- Name: territory_adjacency_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.territory_adjacency_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.territory_adjacency_id_seq OWNER TO risk;

--
-- Name: territory_adjacency_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.territory_adjacency_id_seq OWNED BY public.territory_adjacency.id;


--
-- Name: territory_neighbor_history; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_neighbor_history AS
 SELECT territory_ownership.turn_id,
    territory_adjacency.territory_id AS id,
    json_agg(json_build_object('id', territory_ownership.territory_id, 'name', territories.name, 'shortName', territories.name, 'owner', teams.tname)) AS neighbors
   FROM (((public.territory_adjacency
     JOIN public.territory_ownership ON ((territory_ownership.territory_id = territory_adjacency.adjacent_id)))
     JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     JOIN public.territories ON ((territories.id = territory_ownership.territory_id)))
  WHERE ((territory_adjacency.territory_id <> territory_adjacency.adjacent_id) AND (territory_adjacency.max_turn >= territory_ownership.turn_id) AND (territory_adjacency.min_turn < territory_ownership.turn_id))
  GROUP BY territory_adjacency.territory_id, territory_ownership.turn_id
  ORDER BY territory_adjacency.territory_id;


ALTER TABLE public.territory_neighbor_history OWNER TO risk;

--
-- Name: territory_ownership_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.territory_ownership_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.territory_ownership_id_seq OWNER TO risk;

--
-- Name: territory_ownership_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.territory_ownership_id_seq OWNED BY public.territory_ownership.id;


--
-- Name: territory_ownership_with_neighbors; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_ownership_with_neighbors AS
 SELECT territory_ownership.territory_id,
    turninfo.day,
    turninfo.season,
    territories.name,
    territories.region,
    teams.tname,
    territory_neighbor_history.neighbors
   FROM ((((public.territory_ownership
     JOIN public.teams ON ((teams.id = territory_ownership.owner_id)))
     JOIN public.territories ON ((territory_ownership.territory_id = territories.id)))
     JOIN public.turninfo ON ((territory_ownership.turn_id = turninfo.id)))
     LEFT JOIN public.territory_neighbor_history ON (((territory_ownership.territory_id = territory_neighbor_history.id) AND (territory_neighbor_history.turn_id = turninfo.id))))
  WHERE ((turninfo.id = territory_neighbor_history.turn_id) OR (territory_neighbor_history.neighbors IS NULL));


ALTER TABLE public.territory_ownership_with_neighbors OWNER TO risk;

--
-- Name: territory_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.territory_stats_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.territory_stats_id_seq OWNER TO risk;

--
-- Name: territory_stats_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.territory_stats_id_seq OWNED BY public.territory_stats.id;


--
-- Name: turninfo_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.turninfo_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.turninfo_id_seq OWNER TO risk;

--
-- Name: turninfo_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.turninfo_id_seq OWNED BY public.turninfo.id;


--
-- Name: turns_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.turns_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.turns_id_seq OWNER TO risk;

--
-- Name: turns_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.turns_id_seq OWNED BY public.turns.id;


--
-- Name: user_info; Type: TABLE; Schema: public; Owner: risk
--

CREATE TABLE public.user_info (
    id integer NOT NULL,
    user_id integer NOT NULL,
    team integer NOT NULL,
    turn_id integer NOT NULL,
    ip text,
    pip text,
    ua text,
    co text,
    ac text,
    al text,
    cp text,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.user_info OWNER TO risk;

--
-- Name: user_info_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.user_info_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.user_info_id_seq OWNER TO risk;

--
-- Name: user_info_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.user_info_id_seq OWNED BY public.user_info.id;


--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.users_id_seq OWNER TO risk;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: risk
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: audit_log id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.audit_log ALTER COLUMN id SET DEFAULT nextval('public.audit_log_id_seq'::regclass);


--
-- Name: award_info id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.award_info ALTER COLUMN id SET DEFAULT nextval('public.award_info_id_seq'::regclass);


--
-- Name: awards id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.awards ALTER COLUMN id SET DEFAULT nextval('public.awards_id_seq'::regclass);


--
-- Name: captchas id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.captchas ALTER COLUMN id SET DEFAULT nextval('public.captchas_id_seq'::regclass);


--
-- Name: continuation_polls id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.continuation_polls ALTER COLUMN id SET DEFAULT nextval('public.continuation_polls_id_seq'::regclass);


--
-- Name: continuation_responses id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.continuation_responses ALTER COLUMN id SET DEFAULT nextval('public.continuation_responses_id_seq'::regclass);


--
-- Name: logs id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.logs ALTER COLUMN id SET DEFAULT nextval('public.logs_id_seq'::regclass);


--
-- Name: regions id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.regions ALTER COLUMN id SET DEFAULT nextval('public.regions_id_seq'::regclass);


--
-- Name: teams id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.teams ALTER COLUMN id SET DEFAULT nextval('public.teams_id_seq'::regclass);


--
-- Name: territory_adjacency id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.territory_adjacency ALTER COLUMN id SET DEFAULT nextval('public.territory_adjacency_id_seq'::regclass);


--
-- Name: territory_ownership id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.territory_ownership ALTER COLUMN id SET DEFAULT nextval('public.territory_ownership_id_seq'::regclass);


--
-- Name: territory_stats id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.territory_stats ALTER COLUMN id SET DEFAULT nextval('public.territory_stats_id_seq'::regclass);


--
-- Name: turninfo id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.turninfo ALTER COLUMN id SET DEFAULT nextval('public.turninfo_id_seq'::regclass);


--
-- Name: turns id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.turns ALTER COLUMN id SET DEFAULT nextval('public.turns_id_seq'::regclass);


--
-- Name: user_info id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.user_info ALTER COLUMN id SET DEFAULT nextval('public.user_info_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: captchas captchas_pkey; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.captchas
    ADD CONSTRAINT captchas_pkey PRIMARY KEY (id);


--
-- Name: continuation_responses continuation_responses_poll_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.continuation_responses
    ADD CONSTRAINT continuation_responses_poll_id_user_id_key UNIQUE (user_id, poll_id);


--
-- Name: teams teams_pkey; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.teams
    ADD CONSTRAINT teams_pkey PRIMARY KEY (id);


--
-- Name: territories territories_pkey; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.territories
    ADD CONSTRAINT territories_pkey PRIMARY KEY (id);


--
-- Name: territory_adjacency territory_adjacency_id_key; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.territory_adjacency
    ADD CONSTRAINT territory_adjacency_id_key UNIQUE (id);


--
-- Name: territory_ownership territory_ownership_pkey; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.territory_ownership
    ADD CONSTRAINT territory_ownership_pkey PRIMARY KEY (id);


--
-- Name: turninfo turninfo_pkey; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.turninfo
    ADD CONSTRAINT turninfo_pkey PRIMARY KEY (id);


--
-- Name: turns turns_user_id_season_day_key; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.turns
    ADD CONSTRAINT turns_user_id_season_day_key UNIQUE (user_id, turn_id);


--
-- Name: turninfo unique_season_day; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.turninfo
    ADD CONSTRAINT unique_season_day UNIQUE (season, day);


--
-- Name: users unique_table; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT unique_table UNIQUE (uname, platform);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: risk
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- PostgreSQL database dump complete
--

