-- This script was generated by the ERD tool in pgAdmin 4.
-- Please log an issue at https://redmine.postgresql.org/projects/pgadmin4/issues/new if you find any bugs, including reproduction steps.

CREATE SEQUENCE IF NOT EXISTS public.award_info_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.award_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.ban_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.log_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.region_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.team_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.territory_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.territory_adjacency_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.territory_ownership_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.territory_statistic_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.turn_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.role_id_seq START 1;
CREATE SEQUENCE IF NOT EXISTS public.permission_id_seq START 1;

CREATE TABLE IF NOT EXISTS public.audit_log
(
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    player_id uuid NOT NULL,
    event integer NOT NULL,
    data json,
    session_id uuid NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.award_info
(
    id integer NOT NULL DEFAULT nextval('public.award_info_id_seq'::regclass),
    name text COLLATE pg_catalog."default",
    info text COLLATE pg_catalog."default",
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.award
(
    id integer NOT NULL DEFAULT nextval('public.award_id_seq'::regclass),
    player_id uuid NOT NULL,
    award_id integer NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.ban
(
    id integer NOT NULL DEFAULT nextval('public.ban_id_seq'::regclass),
    class integer,
    cip character varying(128) COLLATE pg_catalog."default",
    name character varying(64) COLLATE pg_catalog."default",
    ua character varying(256) COLLATE pg_catalog."default",
    reason character varying(256) COLLATE pg_catalog."en_US.utf8",
    foreign_service character varying(20),
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id)
);

COMMENT ON COLUMN public.ban.class
    IS '// Username: 1
// Prevent ban, playername, for suspend flag: 2
// Allow login without email: 3
// Prevent ban, Reddit ban: 4';

CREATE TABLE IF NOT EXISTS public.log
(
    id integer NOT NULL DEFAULT nextval('public.log_id_seq'::regclass),
    route text COLLATE pg_catalog."default",
    query text COLLATE pg_catalog."default",
    payload text COLLATE pg_catalog."default",
    created timestamp without time zone DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.region
(
    id integer NOT NULL DEFAULT nextval('public.region_id_seq'::regclass),
    name character varying(64) COLLATE pg_catalog."default" NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.team_statistic
(
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    team integer,
    rank integer,
    territory_count integer,
    player_count integer,
    merc_count integer,
    starpower double precision,
    efficiency double precision,
    effective_power double precision,
    ones integer,
    twos integer,
    threes integer,
    fours integer,
    fives integer,
    turn_id integer,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.team
(
    id integer NOT NULL DEFAULT nextval('public.team_id_seq'::regclass),
    name character varying(64) COLLATE pg_catalog."default" NOT NULL,
    primary_color text COLLATE pg_catalog."default" NOT NULL,
    secondary_color text COLLATE pg_catalog."default" NOT NULL,
    logo text COLLATE pg_catalog."default" NOT NULL,
    seasons integer[],
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT teams_pkey PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.territory
(
    id integer NOT NULL DEFAULT nextval('public.territory_id_seq'::regclass),
    name character varying(64) COLLATE pg_catalog."default" NOT NULL,
    region integer,
    CONSTRAINT territories_pkey PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.territory_adjacency
(
    id integer NOT NULL DEFAULT nextval('public.territory_adjacency_id_seq'::regclass),
    territory_id integer NOT NULL,
    adjacent_id integer NOT NULL,
    note text COLLATE pg_catalog."default",
    min_turn integer NOT NULL,
    max_turn integer NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id),
    CONSTRAINT territory_adjacency_id_key UNIQUE (id)
);

CREATE TABLE IF NOT EXISTS public.territory_ownership
(
    id integer NOT NULL DEFAULT nextval('public.territory_ownership_id_seq'::regclass),
    turn_id integer NOT NULL,
    territory_id integer NOT NULL,
    owner_id integer NOT NULL,
    previous_owner_id integer NOT NULL,
    random_number double precision,
    mvp uuid,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    CONSTRAINT territory_ownership_pkey PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.territory_statistic
(
    id integer NOT NULL DEFAULT nextval('public.territory_statistic_id_seq'::regclass),
    team integer,
    ones integer,
    twos integer,
    threes integer,
    fours integer,
    fives integer,
    teampower double precision,
    chance double precision,
    territory integer,
    territory_power double precision,
    turn_id integer,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.turn
(
    id integer NOT NULL DEFAULT nextval('public.turn_id_seq'::regclass),
    season integer NOT NULL,
    day integer NOT NULL,
    complete boolean NOT NULL DEFAULT false,
    active boolean NOT NULL DEFAULT true,
    finale boolean NOT NULL DEFAULT false,
    rerolls integer NOT NULL DEFAULT 0,
    roll_start timestamp without time zone NOT NULL DEFAULT NOW()+ interval '1 day',
    roll_end timestamp without time zone,
    all_or_nothing boolean NOT NULL DEFAULT false,
    map text COLLATE pg_catalog."default",
    random_seed double precision,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    CONSTRAINT turninfo_pkey PRIMARY KEY (id),
    CONSTRAINT unique_season_day UNIQUE (season, day)
);

CREATE TABLE IF NOT EXISTS public.move
(
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    player_id uuid NOT NULL,
    session_id uuid NOT NULL,
    territory_id integer NOT NULL,
    is_mvp boolean NOT NULL DEFAULT false,
    power double precision NOT NULL,
    multiplier double precision NOT NULL,
    weight double precision NOT NULL,
    stars integer NOT NULL,
    team_id integer NOT NULL,
    alt_score integer NOT NULL,
    is_merc boolean NOT NULL DEFAULT false,
    turn_id integer NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id),
    CONSTRAINT turns_player_id_turn_id_key UNIQUE (player_id, turn_id)
);

CREATE TABLE IF NOT EXISTS public."player"
(
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    name character varying(64) COLLATE pg_catalog."default" NOT NULL,
    main_team integer,
    playing_for integer,
    overall integer NOT NULL DEFAULT 1,
    turns integer NOT NULL DEFAULT 0,
    game_turns integer NOT NULL DEFAULT 0,
    mvps integer NOT NULL DEFAULT 0,
    streak integer NOT NULL DEFAULT 0,
    is_alt boolean NOT NULL DEFAULT false,
    must_captcha boolean NOT NULL DEFAULT true,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    CONSTRAINT players_pkey PRIMARY KEY (id),
    CONSTRAINT unique_table UNIQUE (name)
);

CREATE TABLE IF NOT EXISTS public.authentication_method
(
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    player_id uuid NOT NULL,
    platform character varying(10) NOT NULL,
    foreign_id character varying(256) NOT NULL,
    foreign_name character varying(128),
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id),
    UNIQUE (foreign_id, platform)
        INCLUDE(platform, foreign_id)
);

CREATE TABLE IF NOT EXISTS public.session
(
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    player_id uuid NOT NULL,
    authentication_method_id uuid NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    player_agent character varying(512) NOT NULL,
    ip_address INET NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires timestamp without time zone,
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.role
(
    id integer NOT NULL DEFAULT nextval('public.role_id_seq'::regclass),
    name character varying(24) NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.permission
(
    id integer NOT NULL DEFAULT nextval('public.permission_id_seq'::regclass),
    name character varying(24) NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS public.role_permission
(
    role_id integer NOT NULL,
    permission_id integer NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    PRIMARY KEY (id),
    UNIQUE (role_id, permission_id)
        INCLUDE(role_id, permission_id)
);

CREATE TABLE IF NOT EXISTS public.player_role
(
    role_id integer NOT NULL,
    player_id uuid NOT NULL,
    created timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated timestamp without time zone NOT NULL DEFAULT CURRENT_TIMESTAMP,
    createdby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    updatedby uuid NOT NULL DEFAULT 'a147b32b-6779-462c-b20b-5f5bef4702fa',
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    PRIMARY KEY (id),
    UNIQUE (role_id, player_id)
        INCLUDE(role_id, player_id)
);

ALTER TABLE IF EXISTS public.audit_log
    ADD FOREIGN KEY (player_id)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;

ALTER TABLE IF EXISTS public.audit_log
    ADD FOREIGN KEY (session_id)
    REFERENCES public."session" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;

ALTER TABLE IF EXISTS public.audit_log
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.audit_log
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.award
    ADD FOREIGN KEY (player_id)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.award
    ADD FOREIGN KEY (award_id)
    REFERENCES public.award_info (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.award
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.award
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.ban
    ADD FOREIGN KEY (name)
    REFERENCES public."player" (name) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.ban
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.ban
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.team_statistic
    ADD FOREIGN KEY (team)
    REFERENCES public.team (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.team_statistic
    ADD FOREIGN KEY (turn_id)
    REFERENCES public.turn (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory
    ADD FOREIGN KEY (region)
    REFERENCES public.region (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_adjacency
    ADD FOREIGN KEY (territory_id)
    REFERENCES public.territory (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_adjacency
    ADD FOREIGN KEY (adjacent_id)
    REFERENCES public.territory (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_adjacency
    ADD FOREIGN KEY (min_turn)
    REFERENCES public.turn (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_adjacency
    ADD FOREIGN KEY (max_turn)
    REFERENCES public.turn (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_adjacency
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_adjacency
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_ownership
    ADD FOREIGN KEY (territory_id)
    REFERENCES public.territory (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_ownership
    ADD FOREIGN KEY (mvp)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_statistic
    ADD FOREIGN KEY (turn_id)
    REFERENCES public.turn (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_statistic
    ADD FOREIGN KEY (territory)
    REFERENCES public.territory (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.territory_statistic
    ADD FOREIGN KEY (team)
    REFERENCES public.team (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.turn
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.turn
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.move
    ADD FOREIGN KEY (player_id)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;

ALTER TABLE IF EXISTS public.move
    ADD FOREIGN KEY (session_id)
    REFERENCES public."session" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.move
    ADD FOREIGN KEY (territory_id)
    REFERENCES public.territory (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.move
    ADD FOREIGN KEY (team_id)
    REFERENCES public.team (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.move
    ADD FOREIGN KEY (turn_id)
    REFERENCES public.turn (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.move
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.move
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public."player"
    ADD FOREIGN KEY (main_team)
    REFERENCES public.team (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public."player"
    ADD FOREIGN KEY (playing_for)
    REFERENCES public.team (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public."player"
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public."player"
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.authentication_method
    ADD FOREIGN KEY (player_id)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.session
    ADD FOREIGN KEY (player_id)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.session
    ADD FOREIGN KEY (authentication_method_id)
    REFERENCES public.authentication_method (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.role
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.role
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.permission
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.permission
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.role_permission
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.role_permission
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.role_permission
    ADD FOREIGN KEY (role_id)
    REFERENCES public.role (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.role_permission
    ADD FOREIGN KEY (permission_id)
    REFERENCES public.permission (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.player_role
    ADD FOREIGN KEY (role_id)
    REFERENCES public.role (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.player_role
    ADD FOREIGN KEY (player_id)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.player_role
    ADD FOREIGN KEY (createdby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;


ALTER TABLE IF EXISTS public.player_role
    ADD FOREIGN KEY (updatedby)
    REFERENCES public."player" (id) MATCH SIMPLE
    ON UPDATE NO ACTION
    ON DELETE NO ACTION
    NOT VALID;

CREATE  FUNCTION update_field_alignment()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated = now();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        "player"
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        audit_log
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        award
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        ban
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        "log"
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        team
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        territory_adjacency
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        territory_ownership
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        turn
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        move
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        authentication_method
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        session
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        role
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        permission
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        role_permission
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

CREATE TRIGGER update_player_task_updated_on
    BEFORE UPDATE
    ON
        player_role
    FOR EACH ROW
EXECUTE PROCEDURE update_field_alignment();

INSERT INTO public."player" (id, name, overall, turns, game_turns, mvps, streak, is_alt, must_captcha, created, updated, createdby, updatedby) values ('a147b32b-6779-462c-b20b-5f5bef4702fa', 'System', 0, 0, 0, 0, 0, false, false, '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."permission" (id, name, created, updated, createdby, updatedby) values (0, 'Root', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa', 'a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."permission" (name, created, updated, createdby, updatedby) values ('view_all', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa', 'a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."permission" (name, created, updated, createdby, updatedby) values ('modify_all', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa', 'a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."permission" (name, created, updated, createdby, updatedby) values ('upsert_move', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa', 'a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role" (id, name, created, updated, createdby, updatedby) values (0, 'Root', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role" (name, created, updated, createdby, updatedby) values ('Admin', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role" (name, created, updated, createdby, updatedby) values ('User', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values ((select id from public.role where name = 'Root'),(select id from public.permission where name = 'Root'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values ((select id from public.role where name = 'Root'),(select id from public.permission where name = 'view_all'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values ((select id from public.role where name = 'Root'),(select id from public.permission where name = 'modify_all'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'view_all'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'modify_all'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values ((select id from public.role where name = 'User'),(select id from public.permission where name = 'upsert_move'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."player_role" (role_id, player_id, created, updated, createdby, updatedby) values (0, 'a147b32b-6779-462c-b20b-5f5bef4702fa', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."player" (name, overall, turns, game_turns, mvps, streak, is_alt, must_captcha, created, updated, createdby, updatedby) values ('CollegeFootballRisk', 0, 0, 0, 0, 0, false, false, '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."player_role" (role_id, player_id, created, updated, createdby, updatedby) values (0, (select id from public."player" where name = 'CollegeFootballRisk'), '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."permission" (id, name, created, updated, createdby, updatedby) values
  (4, 'modify_banlist', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'), 
  (5, 'login_as', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  (6, 'reroll', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  (7, 'modify_player', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  (8, 'delete_player', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  (9, 'alter_global_settings', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  (10, 'manage_permissions', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  (11, 'send_system_notice', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role" (name, created, updated, createdby, updatedby) values ('Reroll', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role" (name, created, updated, createdby, updatedby) values ('Security', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role" (name, created, updated, createdby, updatedby) values ('Moderator', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role" (name, created, updated, createdby, updatedby) values ('LoginAs', '1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');



INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values 
  ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'modify_banlist'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'), 
  ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'login_as'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'delete_player'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'modify_player'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'alter_global_settings'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'manage_permissions'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Admin'),(select id from public.permission where name = 'send_system_notice'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values 
((select id from public.role where name = 'Reroll'),(select id from public.permission where name = 'reroll'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values 
  ((select id from public.role where name = 'Security'),(select id from public.permission where name = 'view_all'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Security'),(select id from public.permission where name = 'modify_banlist'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Security'),(select id from public.permission where name = 'modify_player'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values 
  ((select id from public.role where name = 'Moderator'),(select id from public.permission where name = 'view_all'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Moderator'),(select id from public.permission where name = 'modify_banlist'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa'),
  ((select id from public.role where name = 'Moderator'),(select id from public.permission where name = 'send_system_notice'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');

INSERT INTO public."role_permission" (role_id, permission_id, created, updated, createdby, updatedby) values 
((select id from public.role where name = 'LoginAs'),(select id from public.permission where name = 'login_as'),'1920-01-17T12:00:01', '1920-01-17T12:00:01', 'a147b32b-6779-462c-b20b-5f5bef4702fa','a147b32b-6779-462c-b20b-5f5bef4702fa');











