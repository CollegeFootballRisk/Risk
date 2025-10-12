// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;

    audit_log (id) {
        id -> Int4,
        user_id -> Int4,
        event -> Int4,
        timestamp -> Timestamp,
        data -> Nullable<Json>,
        cip -> Nullable<Text>,
        #[max_length = 256]
        ua -> Nullable<Varchar>,
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

    awards (id) {
        id -> Int4,
        user_id -> Int4,
        award_id -> Int4,
        award_date -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    bans (id) {
        id -> Int4,
        class -> Nullable<Int4>,
        cip -> Nullable<Text>,
        uname -> Nullable<Text>,
        ua -> Nullable<Text>,
        #[max_length = 256]
        reason -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    continuation_polls (id) {
        id -> Int4,
        question -> Text,
        incrment -> Int4,
        turn_id -> Nullable<Int4>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    continuation_responses (id) {
        id -> Int4,
        poll_id -> Int4,
        user_id -> Int4,
        response -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    logs (id) {
        id -> Int4,
        route -> Nullable<Text>,
        query -> Nullable<Text>,
        payload -> Nullable<Text>,
        timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    regions (id) {
        id -> Int4,
        name -> Text,
        submap -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    stats (id) {
        team -> Int4,
        rank -> Int4,
        territorycount -> Int4,
        playercount -> Int4,
        merccount -> Int4,
        starpower -> Float8,
        efficiency -> Float8,
        effectivepower -> Float8,
        ones -> Int4,
        twos -> Int4,
        threes -> Int4,
        fours -> Int4,
        fives -> Int4,
        turn_id -> Int4,
        id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    teams (id) {
        id -> Int4,
        tname -> Text,
        tshortname -> Text,
        creation_date -> Nullable<Timestamp>,
        color_1 -> Text,
        color_2 -> Text,
        logo -> Nullable<Text>,
        seasons -> Nullable<Array<Nullable<Int4>>>,
        respawn_count -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    territories (id) {
        id -> Int4,
        name -> Text,
        region -> Int4,
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
    }
}

diesel::table! {
    use diesel::sql_types::*;

    territory_ownership (id) {
        id -> Int4,
        territory_id -> Int4,
        owner_id -> Int4,
        previous_owner_id -> Int4,
        random_number -> Float8,
        timestamp -> Nullable<Timestamp>,
        mvp -> Nullable<Int4>,
        turn_id -> Int4,
        is_respawn -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    territory_stats (id) {
        team -> Int4,
        ones -> Int4,
        twos -> Int4,
        threes -> Int4,
        fours -> Int4,
        fives -> Int4,
        teampower -> Float8,
        chance -> Float8,
        id -> Int4,
        territory -> Int4,
        territory_power -> Float8,
        turn_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    turninfo (id) {
        id -> Int4,
        season -> Int4,
        day -> Int4,
        complete -> Bool,
        active -> Bool,
        finale -> Bool,
        chaosrerolls -> Int4,
        chaosweight -> Int4,
        rollendtime -> Nullable<Timestamp>,
        rollstarttime -> Nullable<Timestamp>,
        allornothingenabled -> Bool,
        map -> Nullable<Text>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    turns (id) {
        id -> Int4,
        user_id -> Int4,
        territory -> Int4,
        mvp -> Bool,
        power -> Float8,
        multiplier -> Float8,
        weight -> Float8,
        stars -> Int4,
        team -> Int4,
        alt_score -> Int4,
        merc -> Bool,
        turn_id -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    users (id) {
        id -> Int4,
        uname -> Text,
        platform -> Text,
        join_date -> Nullable<Timestamp>,
        current_team -> Nullable<Int4>,
        auth_key -> Nullable<Text>,
        overall -> Nullable<Int4>,
        turns -> Nullable<Int4>,
        game_turns -> Nullable<Int4>,
        mvps -> Nullable<Int4>,
        streak -> Nullable<Int4>,
        awards -> Nullable<Int4>,
        role_id -> Nullable<Int4>,
        playing_for -> Nullable<Int4>,
        past_teams -> Nullable<Array<Nullable<Int4>>>,
        awards_bak -> Nullable<Int4>,
        discord_id -> Nullable<Int8>,
        is_alt -> Nullable<Bool>,
        must_captcha -> Nullable<Bool>,
    }
}

diesel::joinable!(awards -> award_info (award_id));
diesel::joinable!(awards -> users (user_id));
diesel::joinable!(territories -> regions (region));

diesel::allow_tables_to_appear_in_same_query!(
    audit_log,
    award_info,
    awards,
    bans,
    continuation_polls,
    continuation_responses,
    logs,
    regions,
    stats,
    teams,
    territories,
    territory_adjacency,
    territory_ownership,
    territory_stats,
    turninfo,
    turns,
    users,
);
