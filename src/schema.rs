// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType, diesel::query_builder::QueryId)]
    #[diesel(postgres_type(name = "EventType"))]
    pub struct EventType;
}

diesel::table! {
    use diesel::sql_types::*;

    audit_log (id) {
        id -> Uuid,
        player_id -> Uuid,
        event -> Int4,
        data -> Nullable<Json>,
        session_id -> Uuid,
        created -> Timestamp,
        createdby -> Uuid,
        updated -> Timestamp,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    authentication_method (id) {
        id -> Uuid,
        player_id -> Uuid,
        #[max_length = 10]
        platform -> Varchar,
        #[max_length = 256]
        foreign_id -> Varchar,
        #[max_length = 128]
        foreign_name -> Nullable<Varchar>,
        published -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    award (id) {
        id -> Int4,
        player_id -> Uuid,
        award_id -> Int4,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    award_info (id) {
        id -> Int4,
        name -> Text,
        info -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    ban (id) {
        id -> Int4,
        class -> Nullable<Int4>,
        #[max_length = 128]
        cip -> Nullable<Varchar>,
        #[max_length = 64]
        name -> Nullable<Varchar>,
        #[max_length = 256]
        ua -> Nullable<Varchar>,
        #[max_length = 256]
        reason -> Nullable<Varchar>,
        #[max_length = 20]
        foreign_service -> Nullable<Varchar>,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::EventType;

    event (id) {
        id -> Uuid,
        event_type -> EventType,
        #[max_length = 256]
        before -> Nullable<Varchar>,
        #[max_length = 256]
        after -> Nullable<Varchar>,
        #[max_length = 256]
        description -> Nullable<Varchar>,
        turn_id -> Nullable<Int4>,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    log (id) {
        id -> Int4,
        route -> Nullable<Text>,
        query -> Nullable<Text>,
        payload -> Nullable<Text>,
        created -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    #[sql_name = "move"]
    move_ (id) {
        id -> Uuid,
        player_id -> Uuid,
        session_id -> Uuid,
        territory_id -> Int4,
        is_mvp -> Bool,
        power -> Float8,
        multiplier -> Float8,
        weight -> Float8,
        stars -> Int4,
        team_id -> Int4,
        alt_score -> Int4,
        is_merc -> Bool,
        turn_id -> Int4,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    permission (id) {
        id -> Int4,
        #[max_length = 24]
        name -> Varchar,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    player (id) {
        id -> Uuid,
        #[max_length = 64]
        name -> Varchar,
        main_team -> Nullable<Int4>,
        playing_for -> Nullable<Int4>,
        overall -> Int4,
        turns -> Int4,
        game_turns -> Int4,
        mvps -> Int4,
        streak -> Int4,
        is_alt -> Bool,
        must_captcha -> Bool,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    player_role (id) {
        role_id -> Int4,
        player_id -> Uuid,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
        id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    region (id) {
        id -> Int4,
        #[max_length = 64]
        name -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    role (id) {
        id -> Int4,
        #[max_length = 24]
        name -> Varchar,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    role_permission (id) {
        role_id -> Int4,
        permission_id -> Int4,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
        id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    session (id) {
        id -> Uuid,
        player_id -> Uuid,
        authentication_method_id -> Uuid,
        is_active -> Bool,
        #[max_length = 512]
        player_agent -> Varchar,
        ip_address -> Inet,
        created -> Timestamp,
        expires -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    team (id) {
        id -> Int4,
        #[max_length = 64]
        name -> Varchar,
        primary_color -> Text,
        secondary_color -> Text,
        logo -> Text,
        seasons -> Nullable<Array<Nullable<Int4>>>,
        created -> Timestamp,
        updated -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    team_statistic (id) {
        id -> Uuid,
        team -> Nullable<Int4>,
        rank -> Nullable<Int4>,
        territory_count -> Nullable<Int4>,
        player_count -> Nullable<Int4>,
        merc_count -> Nullable<Int4>,
        starpower -> Nullable<Float8>,
        efficiency -> Nullable<Float8>,
        effective_power -> Nullable<Float8>,
        ones -> Nullable<Int4>,
        twos -> Nullable<Int4>,
        threes -> Nullable<Int4>,
        fours -> Nullable<Int4>,
        fives -> Nullable<Int4>,
        turn_id -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    territory (id) {
        id -> Int4,
        #[max_length = 64]
        name -> Varchar,
        region -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    territory_adjacency (id) {
        id -> Int4,
        territory_id -> Int4,
        adjacent_id -> Int4,
        note -> Nullable<Text>,
        min_turn -> Int4,
        max_turn -> Int4,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    territory_ownership (id) {
        id -> Int4,
        turn_id -> Int4,
        territory_id -> Int4,
        owner_id -> Int4,
        previous_owner_id -> Int4,
        random_number -> Nullable<Float8>,
        mvp -> Nullable<Uuid>,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    territory_statistic (id) {
        id -> Int4,
        team -> Nullable<Int4>,
        ones -> Nullable<Int4>,
        twos -> Nullable<Int4>,
        threes -> Nullable<Int4>,
        fours -> Nullable<Int4>,
        fives -> Nullable<Int4>,
        teampower -> Nullable<Float8>,
        chance -> Nullable<Float8>,
        territory -> Nullable<Int4>,
        territory_power -> Nullable<Float8>,
        turn_id -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    turn (id) {
        id -> Int4,
        season -> Int4,
        day -> Int4,
        complete -> Bool,
        active -> Bool,
        finale -> Bool,
        rerolls -> Int4,
        roll_start -> Timestamp,
        roll_end -> Nullable<Timestamp>,
        all_or_nothing -> Bool,
        map -> Nullable<Text>,
        random_seed -> Nullable<Float8>,
        created -> Timestamp,
        updated -> Timestamp,
        createdby -> Uuid,
        updatedby -> Uuid,
    }
}

diesel::joinable!(audit_log -> session (session_id));
diesel::joinable!(authentication_method -> player (player_id));
diesel::joinable!(award -> award_info (award_id));
diesel::joinable!(move_ -> session (session_id));
diesel::joinable!(move_ -> team (team_id));
diesel::joinable!(move_ -> territory (territory_id));
diesel::joinable!(move_ -> turn (turn_id));
diesel::joinable!(player_role -> role (role_id));
diesel::joinable!(role_permission -> permission (permission_id));
diesel::joinable!(role_permission -> role (role_id));
diesel::joinable!(session -> authentication_method (authentication_method_id));
diesel::joinable!(session -> player (player_id));
diesel::joinable!(team_statistic -> team (team));
diesel::joinable!(team_statistic -> turn (turn_id));
diesel::joinable!(territory -> region (region));
diesel::joinable!(territory_ownership -> player (mvp));
diesel::joinable!(territory_ownership -> territory (territory_id));
diesel::joinable!(territory_statistic -> team (team));
diesel::joinable!(territory_statistic -> territory (territory));
diesel::joinable!(territory_statistic -> turn (turn_id));

diesel::allow_tables_to_appear_in_same_query!(
    audit_log,
    authentication_method,
    award,
    award_info,
    ban,
    log,
    move_,
    permission,
    player,
    player_role,
    region,
    role,
    role_permission,
    session,
    team,
    team_statistic,
    territory,
    territory_adjacency,
    territory_ownership,
    territory_statistic,
    turn,
);
