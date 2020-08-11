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
        tname -> Nullable<Text>,
        tshortname -> Nullable<Text>,
        creation_date -> Nullable<Timestamp>,
        logo -> Nullable<Text>,
        color_1 -> Nullable<Text>,
        color_2 -> Nullable<Text>,
    }
}

table! {
    users (id) {
        id -> Int4,
        uname -> Text,
        platform -> Text,
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
        uname -> Nullable<Text>,
        turns -> Nullable<Int4>,
        mvps -> Nullable<Int4>,
        tname -> Nullable<Text>,
        power -> Nullable<Int4>,
        weight -> Nullable<Int4>,
        stars -> Nullable<Int4>,
    }
}

table! {
    territories (id) {
        id -> Int4,
        name -> Text,
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
        team -> Nullable<Text>,
        player -> Nullable<Text>,
        stars -> Nullable<Int4>,
        mvp -> Nullable<Bool>,
        territory -> Nullable<Text>,
        regularteam -> Nullable<Text>,
        weight -> Int4,
        power -> Int4,
        multiplier -> Int4,
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
        name -> Text,
        tname -> Text,
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
        name -> Text,
        prev_owner -> Text,
        owner -> Text,
    }
}

table! {
    territory_ownership (id) {
        id -> Int4,
        territory_id -> Int4,
        territory_name -> Nullable<Text>,
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
        name -> Text,
        season -> Int4,
        day -> Int4,
        cumulative_players -> Int8,
        cumulative_power -> Double,
        owner -> Text,
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
        stars -> Int4,
        efficiency -> Double,
        effectivepower -> Double,
        ones -> Int4,
        twos -> Int4,
        threes -> Int4,
        fours -> Int4,
        fives -> Int4,
        tname -> Text,
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
        territory_name -> Text,
        tname -> Text,
        prev_owner -> Text,
        mvp -> Text,
        color -> Text,
        secondary_color -> Text,
        team_name -> Text,
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
