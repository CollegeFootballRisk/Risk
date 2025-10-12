--
-- Name: rr_event; Type: TYPE; Schema: public; Owner: risk
--

DO $do$ BEGIN IF EXISTS (
    SELECT
    FROM pg_catalog.pg_roles
    WHERE rolname = 'risk'
) THEN RAISE NOTICE 'Role "risk" already exists. Skipping.';
ELSE CREATE ROLE risk;
END IF;
END $do$;
SET default_tablespace = '';
SET default_table_access_method = heap;
--
-- Name: audit_log; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.audit_log (
    id integer NOT NULL PRIMARY KEY,
    user_id integer NOT NULL,
    event integer NOT NULL,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    data json,
    cip text,
    ua character varying(256)
);
ALTER TABLE public.audit_log OWNER TO risk;
--
-- Name: audit_log_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.audit_log_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.audit_log_id_seq OWNER TO risk;
--
-- Name: award_info; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.award_info (
    id integer NOT NULL PRIMARY KEY,
    name text NOT NULL,
    info text NOT NULL
);
ALTER TABLE public.award_info OWNER TO risk;
--
-- Name: award_info_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.award_info_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.award_info_id_seq OWNER TO risk;
--
-- Name: bans; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.bans (
    id integer NOT NULL PRIMARY KEY,
    class integer,
    cip text,
    uname text,
    ua text,
    reason character varying(256) COLLATE pg_catalog."en_US.utf8"
);
ALTER TABLE public.bans OWNER TO risk;
--
-- Name: COLUMN bans.class; Type: COMMENT; Schema: public; Owner: postgres
--

COMMENT ON COLUMN public.bans.class IS '// Username: 1
// Prevent ban, username, for suspend flag: 2
// Allow login without email: 3
// Prevent ban, Reddit ban: 4';
--
-- Name: bans_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.bans_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.bans_id_seq OWNER TO risk;
--
-- Name: continuation_polls; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.continuation_polls (
    id integer NOT NULL PRIMARY KEY,
    question text NOT NULL DEFAULT 'Should this season be extended by seven more days?'::text,
    incrment integer NOT NULL DEFAULT 7,
    turn_id integer
);
ALTER TABLE public.continuation_polls OWNER TO risk;
--
-- Name: continuation_polls_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.continuation_polls_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.continuation_polls_id_seq OWNER TO risk;
--
-- Name: continuation_responses; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.continuation_responses (
    id integer NOT NULL PRIMARY KEY,
    poll_id integer NOT NULL,
    user_id integer NOT NULL,
    response boolean NOT NULL
);
ALTER TABLE public.continuation_responses OWNER TO risk;
--
-- Name: continuation_responses_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.continuation_responses_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.continuation_responses_id_seq OWNER TO risk;
--
-- Name: turninfo; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.turninfo (
    id integer NOT NULL PRIMARY KEY,
    season integer NOT NULL,
    day integer NOT NULL,
    complete boolean NOT NULL DEFAULT false,
    active boolean NOT NULL DEFAULT false,
    finale boolean NOT NULL DEFAULT false,
    chaosrerolls integer NOT NULL DEFAULT 0,
    chaosweight integer NOT NULL DEFAULT 1,
    rollendtime timestamp without time zone,
    rollstarttime timestamp without time zone,
    allornothingenabled boolean NOT NULL DEFAULT true,
    map text
);
ALTER TABLE public.turninfo OWNER TO risk;
--
-- Name: turns; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.turns (
    id integer NOT NULL PRIMARY KEY,
    user_id integer NOT NULL,
    territory integer NOT NULL,
    mvp boolean DEFAULT false NOT NULL,
    power double precision NOT NULL,
    multiplier double precision NOT NULL,
    weight double precision NOT NULL,
    stars integer NOT NULL,
    team integer NOT NULL,
    alt_score integer NOT NULL DEFAULT 0,
    merc boolean NOT NULL DEFAULT false,
    turn_id integer NOT NULL
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
FROM (
        public.turns
        JOIN public.turninfo ON ((turninfo.id = turns.turn_id))
    )
WHERE (turninfo.complete = true);
ALTER VIEW public.past_turns OWNER TO risk;
--
-- Name: territories_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.territories_seq START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.territories_seq OWNER TO risk;
--
-- Name: territories; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.territories (
    id integer DEFAULT nextval('public.territories_seq'::regclass) NOT NULL PRIMARY KEY,
    name text NOT NULL,
    region integer NOT NULL
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
FROM (
        (
            public.territories
            CROSS JOIN (
                SELECT turninfo.id,
                    turninfo.season,
                    turninfo.day
                FROM public.turninfo
                WHERE (turninfo.complete = true)
            ) rd
        )
        LEFT JOIN public.past_turns ON (
            (
                (rd.id = past_turns.turn_id)
                AND (territories.id = past_turns.territory)
            )
        )
    )
WHERE (territories.id > 0)
GROUP BY territories.name,
    rd.season,
    rd.day
ORDER BY territories.name,
    rd.season DESC,
    rd.day DESC;
ALTER VIEW public.heat OWNER TO risk;
--
-- Name: teams; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.teams (
    id integer NOT NULL PRIMARY KEY,
    tname text NOT NULL,
    tshortname text NOT NULL,
    creation_date timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    color_1 text NOT NULL DEFAULT '#FFF',
    color_2 text NOT NULL DEFAULT '#000',
    logo text,
    seasons integer [],
    respawn_count integer not null default 0
);
ALTER TABLE public.teams OWNER TO risk;
--
-- Name: territory_ownership; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.territory_ownership (
    id integer NOT NULL PRIMARY KEY,
    territory_id integer NOT NULL,
    owner_id integer NOT NULL,
    previous_owner_id integer NOT NULL,
    random_number double precision NOT NULL DEFAULT 0,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    mvp integer,
    turn_id integer NOT NULL,
    is_respawn boolean not null default false
);
ALTER TABLE public.territory_ownership OWNER TO risk;
--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    id integer NOT NULL PRIMARY KEY,
    uname text NOT NULL,
    platform text NOT NULL,
    join_date timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    current_team integer,
    auth_key text,
    overall integer DEFAULT 1,
    turns integer DEFAULT 0,
    game_turns integer DEFAULT 0,
    mvps integer DEFAULT 0,
    streak integer DEFAULT 0,
    awards integer DEFAULT 0,
    role_id integer DEFAULT 0,
    playing_for integer,
    past_teams integer [],
    awards_bak integer,
    discord_id bigint,
    is_alt boolean DEFAULT false,
    must_captcha boolean DEFAULT true
);
ALTER TABLE public.users OWNER TO risk;
--
-- Name: awards; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.awards (
    id integer NOT NULL PRIMARY KEY,
    user_id integer NOT NULL references users(id),
    award_id integer NOT NULL references award_info(id),
    award_date timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);
ALTER TABLE public.awards OWNER TO risk;
--
-- Name: awards_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.awards_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.awards_id_seq OWNER TO risk;
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
FROM (
        (
            (
                (
                    (
                        public.territory_ownership
                        LEFT JOIN public.teams ON ((teams.id = territory_ownership.owner_id))
                    )
                    LEFT JOIN public.teams tex ON ((tex.id = territory_ownership.previous_owner_id))
                )
                LEFT JOIN public.territories ON (
                    (
                        territory_ownership.territory_id = territories.id
                    )
                )
            )
            LEFT JOIN public.turninfo ON ((territory_ownership.turn_id = turninfo.id))
        )
        LEFT JOIN public.users ON ((users.id = territory_ownership.mvp))
    )
ORDER BY turninfo.id DESC;
ALTER VIEW public.territory_ownership_without_neighbors OWNER TO risk;
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
        WHEN (
            territory_ownership_without_neighbors.owner IS NULL
        ) THEN 'None'::text
        ELSE territory_ownership_without_neighbors.owner
    END AS owner
FROM (
        public.heat
        LEFT JOIN public.territory_ownership_without_neighbors ON (
            (
                (
                    (territory_ownership_without_neighbors.name)::text = (heat.name)::text
                )
                AND (
                    territory_ownership_without_neighbors.day = (heat.day + 1)
                )
                AND (
                    territory_ownership_without_neighbors.season = heat.season
                )
            )
        )
    );
ALTER VIEW public.heat_full OWNER TO risk;
--
-- Name: logs; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.logs (
    id integer NOT NULL PRIMARY KEY,
    route text,
    query text,
    payload text,
    "timestamp" timestamp without time zone DEFAULT CURRENT_TIMESTAMP
);
ALTER TABLE public.logs OWNER TO risk;
--
-- Name: logs_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.logs_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.logs_id_seq OWNER TO risk;
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
    past_turns.stars,
    users.overall AS current_stars
FROM (
        (
            (
                (
                    (
                        SELECT max(past_turns_1.id) AS id,
                            past_turns_1.user_id
                        FROM public.past_turns past_turns_1
                        GROUP BY past_turns_1.user_id
                    ) foo
                    JOIN public.past_turns ON ((past_turns.id = foo.id))
                )
                LEFT JOIN public.turninfo ON ((turninfo.id = past_turns.turn_id))
            )
            LEFT JOIN public.users ON ((foo.user_id = users.id))
        )
        LEFT JOIN public.teams ON ((users.current_team = teams.id))
    )
ORDER BY users.uname;
ALTER VIEW public.moves OWNER TO risk;
--
-- Name: territory_stats; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.territory_stats (
    team integer NOT NULL,
    ones integer NOT NULL DEFAULT 0,
    twos integer NOT NULL DEFAULT 0,
    threes integer NOT NULL DEFAULT 0,
    fours integer NOT NULL DEFAULT 0,
    fives integer NOT NULL DEFAULT 0,
    teampower double precision NOT NULL DEFAULT 0.0,
    chance double precision NOT NULL DEFAULT 0.0,
    id integer NOT NULL PRIMARY KEY,
    territory integer NOT NULL,
    territory_power double precision NOT NULL,
    turn_id integer NOT NULL
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
    (
        (
            (
                (territory_stats.ones + territory_stats.twos) + territory_stats.threes
            ) + territory_stats.fours
        ) + territory_stats.fives
    ) AS players,
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
FROM (
        (
            (
                (
                    public.territory_stats
                    JOIN public.territories ON ((territories.id = territory_stats.territory))
                )
                JOIN public.teams ON ((teams.id = territory_stats.team))
            )
            JOIN public.turninfo ON ((turninfo.id = territory_stats.turn_id))
        )
        JOIN public.territory_ownership_without_neighbors ON (
            (
                (
                    (territory_ownership_without_neighbors.name)::text = (territories.name)::text
                )
                AND (
                    territory_ownership_without_neighbors.season = turninfo.season
                )
                AND (
                    territory_ownership_without_neighbors.day = (turninfo.day + 1)
                )
            )
        )
    );
ALTER VIEW public.odds OWNER TO risk;
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
FROM (
        public.users
        JOIN public.teams ON ((teams.id = users.current_team))
    );
ALTER VIEW public.players OWNER TO risk;
--
-- Name: region_ownership; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.region_ownership AS
SELECT count(DISTINCT territory_ownership.owner_id) AS owner_count,
    array_agg(DISTINCT territory_ownership.owner_id) AS owners,
    turninfo.day,
    turninfo.season,
    territories.region
FROM (
        (
            public.territory_ownership
            LEFT JOIN public.territories ON (
                (
                    territory_ownership.territory_id = territories.id
                )
            )
        )
        LEFT JOIN public.turninfo ON ((turninfo.id = territory_ownership.turn_id))
    )
GROUP BY turninfo.day,
    turninfo.season,
    territories.region
ORDER BY turninfo.season DESC,
    turninfo.day DESC;
ALTER VIEW public.region_ownership OWNER TO risk;
--
-- Name: regions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.regions (
    id integer NOT NULL PRIMARY KEY,
    name text NOT NULL,
    submap integer not null default 0
);
ALTER TABLE public.regions OWNER TO risk;
--
-- Name: regions_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.regions_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.regions_id_seq OWNER TO risk;
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
    json_agg(
        json_build_object(
            'territory',
            territory_ownership_without_neighbors.name,
            'timestamp',
            territory_ownership_without_neighbors."timestamp",
            'winner',
            territory_ownership_without_neighbors.owner,
            'randomNumber',
            territory_ownership_without_neighbors.random_number
        )
    ) AS json_agg
FROM (
        public.territory_ownership_without_neighbors
        JOIN public.turninfo ON (
            (
                (
                    turninfo.day = (territory_ownership_without_neighbors.day - 1)
                )
                AND (
                    turninfo.season = territory_ownership_without_neighbors.season
                )
            )
        )
    )
GROUP BY territory_ownership_without_neighbors.day,
    territory_ownership_without_neighbors.season,
    turninfo.chaosrerolls,
    turninfo.rollstarttime,
    turninfo.rollendtime,
    turninfo.chaosweight;
ALTER VIEW public.rollinfo OWNER TO risk;
--
-- Name: stats; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.stats (
    team integer NOT NULL,
    rank integer NOT NULL,
    territorycount integer NOT NULL DEFAULT 0,
    playercount integer NOT NULL DEFAULT 0,
    merccount integer NOT NULL DEFAULT 0,
    starpower double precision NOT NULL DEFAULT 0.0,
    efficiency double precision NOT NULL DEFAULT 0.0,
    effectivepower double precision NOT NULL DEFAULT 0.0,
    ones integer NOT NULL DEFAULT 0,
    twos integer NOT NULL DEFAULT 0,
    threes integer NOT NULL DEFAULT 0,
    fours integer NOT NULL DEFAULT 0,
    fives integer NOT NULL DEFAULT 0,
    turn_id integer NOT NULL DEFAULT 0,
    id UUID PRIMARY KEY NOT NULL
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
    teams.logo,
    COALESCE(r.regions, (0)::bigint) AS regions
FROM (
        (
            (
                public.stats
                JOIN public.teams ON ((teams.id = stats.team))
            )
            JOIN public.turninfo ON ((turninfo.id = stats.turn_id))
        )
        LEFT JOIN (
            SELECT region_ownership.season,
                region_ownership.day,
                region_ownership.owners [1] AS team_id,
                count(DISTINCT region_ownership.region) AS regions
            FROM public.region_ownership
            WHERE (
                    (region_ownership.owner_count = 1)
                    AND (region_ownership.season > 2)
                    AND (region_ownership.region <> 22)
                )
            GROUP BY region_ownership.season,
                region_ownership.day,
                region_ownership.owners [1]
        ) r ON (
            (
                (r.season = turninfo.season)
                AND (r.day = (turninfo.day + 1))
                AND (r.team_id = stats.team)
            )
        )
    );
ALTER VIEW public.statistics OWNER TO risk;
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
FROM (
        (
            (
                (
                    (
                        public.past_turns
                        JOIN public.territories ON ((territories.id = past_turns.territory))
                    )
                    JOIN public.teams ON ((teams.id = past_turns.team))
                )
                JOIN public.turninfo ON ((past_turns.turn_id = turninfo.id))
            )
            LEFT JOIN public.users ON ((users.id = past_turns.user_id))
        )
        LEFT JOIN public.teams t2 ON ((t2.id = users.current_team))
    )
ORDER BY territories.name,
    past_turns.team;
ALTER VIEW public.team_player_moves OWNER TO risk;
--
-- Name: teams_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.teams_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.teams_id_seq OWNER TO risk;
--
-- Name: territory_adjacency; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.territory_adjacency (
    id integer primary key NOT NULL,
    territory_id integer NOT NULL,
    adjacent_id integer NOT NULL,
    note text,
    min_turn integer NOT NULL,
    max_turn integer NOT NULL
);
ALTER TABLE public.territory_adjacency OWNER TO risk;
--
-- Name: territory_adjacency_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.territory_adjacency_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.territory_adjacency_id_seq OWNER TO risk;
--
-- Name: territory_neighbor_history; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_neighbor_history AS
SELECT territory_ownership.turn_id,
    territory_adjacency.territory_id AS id,
    json_agg(
        json_build_object(
            'id',
            territory_ownership.territory_id,
            'name',
            territories.name,
            'shortName',
            territories.name,
            'owner',
            teams.tname
        )
    ) AS neighbors
FROM (
        (
            (
                public.territory_adjacency
                JOIN public.territory_ownership ON (
                    (
                        territory_ownership.territory_id = territory_adjacency.adjacent_id
                    )
                )
            )
            JOIN public.teams ON ((teams.id = territory_ownership.owner_id))
        )
        JOIN public.territories ON (
            (
                territories.id = territory_ownership.territory_id
            )
        )
    )
WHERE (
        (
            territory_adjacency.territory_id <> territory_adjacency.adjacent_id
        )
        AND (
            territory_adjacency.max_turn >= territory_ownership.turn_id
        )
        AND (
            territory_adjacency.min_turn < territory_ownership.turn_id
        )
    )
GROUP BY territory_adjacency.territory_id,
    territory_ownership.turn_id
ORDER BY territory_adjacency.territory_id;
ALTER VIEW public.territory_neighbor_history OWNER TO risk;
--
-- Name: territory_ownership_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.territory_ownership_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.territory_ownership_id_seq OWNER TO risk;
--
-- Name: territory_ownership_with_neighbors; Type: VIEW; Schema: public; Owner: risk
--

CREATE VIEW public.territory_ownership_with_neighbors AS
SELECT territories.id AS territory_id,
    turninfo.day,
    turninfo.season,
    territories.name,
    teams.tname,
    territories.region,
    regions.name AS region_name,
    json_agg(
        json_build_object(
            'id',
            t2.territory_id,
            'name',
            t3.name,
            'shortName',
            t3.name,
            'owner',
            t4.tname
        )
    ) AS neighbors
FROM (
        (
            (
                (
                    (
                        (
                            (
                                (
                                    public.territory_ownership
                                    JOIN public.turninfo ON ((turninfo.id = territory_ownership.turn_id))
                                )
                                JOIN public.territory_adjacency ON (
                                    (
                                        (
                                            territory_adjacency.territory_id = territory_ownership.territory_id
                                        )
                                        AND (
                                            territory_adjacency.max_turn >= territory_ownership.turn_id
                                        )
                                        AND (
                                            territory_adjacency.min_turn < territory_ownership.turn_id
                                        )
                                    )
                                )
                            )
                            JOIN public.territory_ownership t2 ON (
                                (
                                    (t2.turn_id = territory_ownership.turn_id)
                                    AND (
                                        t2.territory_id = territory_adjacency.adjacent_id
                                    )
                                    AND (
                                        t2.territory_id <> territory_ownership.territory_id
                                    )
                                )
                            )
                        )
                        JOIN public.territories ON (
                            (
                                territories.id = territory_ownership.territory_id
                            )
                        )
                    )
                    JOIN public.territories t3 ON ((t3.id = territory_adjacency.adjacent_id))
                )
                JOIN public.teams ON ((teams.id = territory_ownership.owner_id))
            )
            JOIN public.teams t4 ON ((t4.id = t2.owner_id))
        )
        JOIN public.regions ON ((regions.id = territories.region))
    )
GROUP BY territories.id,
    territories.name,
    teams.tname,
    territories.region,
    turninfo.season,
    turninfo.day,
    regions.name
ORDER BY territories.id;
ALTER VIEW public.territory_ownership_with_neighbors OWNER TO risk;
--
-- Name: territory_stats_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.territory_stats_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.territory_stats_id_seq OWNER TO risk;
--
-- Name: turninfo_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.turninfo_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.turninfo_id_seq OWNER TO risk;
--
-- Name: turns_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.turns_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.turns_id_seq OWNER TO risk;
--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: risk
--

CREATE SEQUENCE public.users_id_seq AS integer START WITH 1 INCREMENT BY 1 NO MINVALUE NO MAXVALUE CACHE 1;
ALTER SEQUENCE public.users_id_seq OWNER TO risk;
--
-- Name: territory_ownershi_idx_owner_id_turn_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX territory_ownershi_idx_owner_id_turn_id ON public.territory_ownership USING btree (owner_id, turn_id);
--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
--

REVOKE USAGE ON SCHEMA public
FROM PUBLIC;
GRANT ALL ON SCHEMA public TO PUBLIC;
CREATE TYPE public.rr_event AS ENUM ('notification', 'change_team');
ALTER TYPE public.rr_event OWNER TO risk;
--
-- Name: _final_median(numeric[]); Type: FUNCTION; Schema: public; Owner: risk
--

CREATE FUNCTION public._final_median(numeric []) RETURNS numeric LANGUAGE sql IMMUTABLE AS $_$
SELECT AVG(val)
FROM (
        SELECT val
        FROM unnest($1) val
        ORDER BY 1
        LIMIT 2 - MOD(array_upper($1, 1), 2) OFFSET CEIL(array_upper($1, 1) / 2.0) - 1
    ) sub;
$_$;
ALTER FUNCTION public._final_median(numeric []) OWNER TO risk;
--
-- Name: do_user_update(integer, integer); Type: FUNCTION; Schema: public; Owner: risk
--

CREATE FUNCTION public.do_user_update(turn integer, season integer) RETURNS boolean LANGUAGE plpgsql SECURITY DEFINER AS $$ BEGIN
UPDATE users
SET streak = streak + 1
WHERE id in (
        SELECT user_id
        FROM turns
        WHERE turns.turn_id = do_user_update.turn
    );
UPDATE users
SET streak = 0
WHERE id NOT IN (
        SELECT user_id
        FROM turns
        WHERE turns.turn_id = do_user_update.turn
    );
UPDATE users
SET mvps = mvps.mvps,
    turns = mvps.turnsz
FROM (
        SELECT user_id,
            SUM(
                case
                    when mvp = true THEN 1
                    ELSE 0
                END
            ) as mvps,
            count(*) as turnsz
        FROM turns
        GROUP BY user_id
    ) as mvps
WHERE mvps.user_id = users.id;
UPDATE users
SET game_turns = game_turns.game_turns
FROM (
        SELECT user_id,
            count(*) as game_turns
        FROM turns
            inner join turninfo on turninfo.id = turns.turn_id
        WHERE turninfo.season = do_user_update.season
        GROUP BY user_id
    ) as game_turns
WHERE game_turns.user_id = users.id;
UPDATE users
SET overall = overall.overall
FROM (
        select id,
            median(power) as overall
        from (
                select id,
                    case
                        when mvps >= 25 then 5
                        when mvps >= 10 then 4
                        when mvps >= 5 then 3
                        when mvps >= 1 then 2
                        else 1
                    end as power
                from users
                union all
                select id,
                    case
                        when turns >= 100 then 5
                        when turns >= 50 then 4
                        when turns >= 25 then 3
                        when turns >= 10 then 2
                        else 1
                    end as power
                from users
                union all
                select id,
                    case
                        when game_turns >= 40 then 5
                        when game_turns >= 25 then 4
                        when game_turns >= 10 then 3
                        when game_turns >= 5 then 2
                        else 1
                    end as power
                from users
                union all
                select id,
                    case
                        when awards >= 4 then 5
                        when awards >= 3 then 4
                        when awards >= 2 then 3
                        when awards >= 1 then 2
                        else 1
                    end as power
                from users
                union all
                select id,
                    case
                        when streak >= 25 then 5
                        when streak >= 10 then 4
                        when streak >= 5 then 3
                        when streak >= 3 then 2
                        else 1
                    end as power
                from users
            ) t
        group by 1
    ) as overall
where overall.id = users.id;
update users
set playing_for = -1
where playing_for not in (
        select distinct(owner_id)
        from territory_ownership
        where territory_ownership.turn_id = do_user_update.turn + 1
    );
return FOUND;
END;
$$;
ALTER FUNCTION public.do_user_update(turn integer, season integer) OWNER TO risk;
--
-- Name: median(numeric); Type: AGGREGATE; Schema: public; Owner: risk
--

CREATE AGGREGATE public.median(numeric) (
    SFUNC = array_append,
    STYPE = numeric [],
    INITCOND = '{}',
    FINALFUNC = public._final_median
);
ALTER AGGREGATE public.median(numeric) OWNER TO risk;