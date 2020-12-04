table! {
    past_turns (id) {
        id -> Int4,
        user_id -> Nullable<Int4>,
        season -> Nullable<Int4>,
        day -> Nullable<Int4>,
        territory -> Int4,
        mvp -> Bool,
        power -> Nullable<Double>,
        multiplier -> Nullable<Double>,
        weight -> Nullable<Int4>,
        stars -> Nullable<Int4>,
        team -> Int4,
    }
}

table! {
    teams (id) {
        id -> Int4,
        tname -> Nullable<diesel_citext::sql_types::Citext>,
        tshortname -> Nullable<diesel_citext::sql_types::Citext>,
        creation_date -> Nullable<Timestamp>,
        logo -> Nullable<Text>,
        color_1 -> Nullable<Text>,
        color_2 -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Int4,
        uname -> diesel_citext::sql_types::Citext,
        platform -> diesel_citext::sql_types::Citext,
        join_date -> Nullable<Timestamp>,
        current_team -> Int4,
        overall -> Nullable<Int4>,
        turns -> Nullable<Int4>,
        game_turns -> Nullable<Int4>,
        mvps -> Nullable<Int4>,
        streak -> Nullable<Int4>,
        awards -> Nullable<Int4>,
        role_id -> Nullable<Int4>,
        playing_for -> Int4,
    }
}

table! {
    moves (player) {
        season -> Nullable<Int4>,
        day -> Nullable<Int4>,
        territory -> Nullable<Int4>,
        player -> Nullable<Int4>,
        team -> Nullable<Int4>,
        mvp -> Nullable<Int4>,
        uname -> Nullable<diesel_citext::sql_types::Citext>,
        turns -> Nullable<Int4>,
        mvps -> Nullable<Int4>,
        tname -> Nullable<diesel_citext::sql_types::Citext>,
        power -> Nullable<Double>,
        weight -> Nullable<Int4>,
        stars -> Nullable<Int4>,
    }
}

table! {
    territories (id) {
        id -> Int4,
        name -> diesel_citext::sql_types::Citext,
    }
}

table! {
    turninfo (id) {
        id -> Int4,
        season -> Nullable<Int4>,
        day -> Nullable<Int4>,
        complete -> Nullable<Bool>,
        active -> Nullable<Bool>,
        finale -> Nullable<Bool>,
    }
}

table! {
    team_player_moves (id) {
        id -> Int4,
        season -> Nullable<Int4>,
        day -> Nullable<Int4>,
        team -> Nullable<diesel_citext::sql_types::Citext>,
        player -> Nullable<diesel_citext::sql_types::Citext>,
        stars -> Nullable<Int4>,
        mvp -> Nullable<Bool>,
        territory -> Nullable<diesel_citext::sql_types::Citext>,
        regularteam -> Nullable<diesel_citext::sql_types::Citext>,
        weight -> Int4,
        power -> Double,
        multiplier -> Double,
    }
}

table! {
    captchas (id) {
        id -> Int4,
        title -> Text,
        content -> Text,
    }
}

table! {
    territory_ownership_with_neighbors (territory_id) {
        territory_id -> Int4,
        season -> Int4,
        day -> Int4,
        name -> diesel_citext::sql_types::Citext,
        tname -> diesel_citext::sql_types::Citext,
        neighbors -> Json,
    }
}

table! {
    rollinfo (day) {
        season -> Int4,
        day -> Int4,
        chaosrerolls -> Int4,
        chaosweight -> Int4,
        rollstarttime -> Text,
        rollendtime -> Text,
        json_agg -> Json,
    }
}

table! {
    territory_ownership_without_neighbors (territory_id) {
        territory_id -> Int4,
        season -> Int4,
        day -> Int4,
        name -> diesel_citext::sql_types::Citext,
        prev_owner -> diesel_citext::sql_types::Citext,
        owner -> diesel_citext::sql_types::Citext,
    }
}

table! {
    territory_ownership (id) {
        id -> Int4,
        territory_id -> Int4,
        territory_name -> Nullable<diesel_citext::sql_types::Citext>,
        owner_id -> Int4,
        day -> Int4,
        season -> Int4,
        previous_owner_id -> Int4,
        random_number -> Double,
        mvp -> Int4,
    }
}

table! {
    heat_full (name) {
        name -> diesel_citext::sql_types::Citext,
        season -> Int4,
        day -> Int4,
        cumulative_players -> Int8,
        cumulative_power -> Double,
        owner -> diesel_citext::sql_types::Citext,
    }
}

table! {
    statistics (sequence) {
        sequence -> Int4,
        season -> Int4,
        day -> Int4,
        team -> Int4,
        rank -> Int4,
        territorycount -> Int4,
        playercount -> Int4,
        merccount -> Int4,
        starpower -> Double,
        efficiency -> Double,
        effectivepower -> Double,
        ones -> Int4,
        twos -> Int4,
        threes -> Int4,
        fours -> Int4,
        fives -> Int4,
        tname -> diesel_citext::sql_types::Citext,
        logo -> Text,
    }
}

table! {
    territory_adjacency (id) {
        id -> Int4,
        territory_id -> Int4,
        adjacent_id -> Int4,
    }
}

table! {
    odds (territory_name) {
        players -> Int4,
        ones -> Int4,
        twos -> Int4,
        threes -> Int4,
        fours -> Int4,
        fives -> Int4,
        teampower -> Double,
        territorypower -> Double,
        chance -> Double,
        team -> Integer,
        season -> Integer,
        day -> Integer,
        territory_name -> diesel_citext::sql_types::Citext,
        tname -> diesel_citext::sql_types::Citext,
        prev_owner -> diesel_citext::sql_types::Citext,
        mvp -> Nullable<diesel_citext::sql_types::Citext>,
        color -> Text,
        secondary_color -> Text,
        team_name -> diesel_citext::sql_types::Citext,
    }
}
table! {
    new_turns (id) {
        id -> Int4,
        user_id -> Int4,
        season -> Int4,
        day -> Int4,
        territory -> Int4,
        mvp -> Bool,
        power -> Float,
        multiplier -> Float,
        weight -> Float,
        stars -> Int4,
        team -> Int4,
        alt_score -> Int4,
        merc -> Bool,
    }
}
allow_tables_to_appear_in_same_query!(
    past_turns,
    teams,
    users,
    territory_ownership,
    territory_adjacency,
    territories
);

allow_tables_to_appear_in_same_query!(new_turns, territories);
