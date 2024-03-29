/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

table! {
    past_turns (id) {
        id -> Int4,
        user_id -> Int4,
        turn_id -> Int4,
        territory -> Int4,
        mvp -> Bool,
        power -> Double,
        multiplier -> Double,
        weight -> Double,
        stars -> Int4,
        team -> Int4,
        alt_score -> Int4,
        merc -> Bool,
    }
}

table! {
    teams (id) {
        id -> Int4,
        tname -> Text,
        tshortname -> Text,
        creation_date -> Timestamp,
        logo -> Nullable<Text>,
        color_1 -> Text,
        color_2 -> Text,
        seasons -> Array<Int4>,
        respawn_count -> Int4,
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
        role_id -> Nullable<Int4>,
        playing_for -> Int4,
        is_alt -> Bool,
        must_captcha -> Bool,
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
        power -> Nullable<Double>,
        weight -> Nullable<Int4>,
        stars -> Nullable<Int4>,
    }
}

table! {
    territories (id) {
        id -> Int4,
        name -> Text,
        region -> Int4,
    }
}

table! {
    regions (id){
        id -> Int4,
        name -> Text,
        submap  -> Int4,
    }
}

table! {
    turninfo (id) {
        id -> Int4,
        season -> Int4,
        day -> Int4,
        complete -> Nullable<Bool>,
        active -> Nullable<Bool>,
        finale -> Nullable<Bool>,
        chaosrerolls -> Nullable<Int4>,
        chaosweight -> Nullable<Int4>,
        rollendtime -> Nullable<Timestamp>,
        rollstarttime -> Nullable<Timestamp>,
        map -> Nullable<Text>,
        allornothingenabled -> Nullable<Bool>,
    }
}

//TODO: Get rid of this.
// ^^^ Blocked by not being on diesel 2.0
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
        weight -> Double,
        power -> Double,
        multiplier -> Double,
    }
}

table! {
    captchas (id) {
        id -> Int4,
        title -> Text,
        content -> Text,
        creation -> Timestamp,
    }
}

table! {
    territory_ownership_with_neighbors (territory_id) {
        territory_id -> Int4,
        season -> Int4,
        day -> Int4,
        name -> Text,
        tname -> Text,
        region-> Int4,
        region_name -> Text,
        neighbors -> Nullable<Json>,
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
        owner_id -> Int4,
        turn_id -> Int4,
        previous_owner_id -> Int4,
        random_number -> Double,
        mvp -> Nullable<Int4>,
        is_respawn -> Bool,
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
    cfbr_stats (player) {
        player -> Text,
        team -> Text,
        turnsplayed -> Int4,
        stars -> Int4,
    }
}

table! {
    statistics (turn_id) {
        turn_id -> Int4,
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
        tname -> Text,
        logo -> Text,
        regions -> Int8,
    }
}

table! {
    stats (turn_id) {
        turn_id -> Int4,
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
    }
}

table! {
    territory_adjacency (id) {
        id -> Int4,
        territory_id -> Int4,
        adjacent_id -> Int4,
        note -> Text,
        min_turn -> Int4,
        max_turn -> Int4,
    }
}

table! {
    territory_stats (id) {
        team -> Int4,
        turn_id -> Int4,
        ones -> Int4,
        twos -> Int4,
        threes -> Int4,
        fours -> Int4,
        fives -> Int4,
        teampower -> Double,
        chance -> Double,
        id -> Int4,
        territory -> Int4,
        territory_power -> Double,
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
        mvp -> Nullable<Text>,
        color -> Text,
        secondary_color -> Text,
        team_name -> Text,
    }
}
table! {
    turns (id) {
        id -> Int4,
        user_id -> Int4,
        turn_id -> Int4,
        territory -> Int4,
        mvp -> Bool,
        power -> Double,
        multiplier -> Double,
        weight -> Double,
        stars -> Int4,
        team -> Int4,
        alt_score -> Int4,
        merc -> Bool,
    }
}
table! {
    continuation_polls (id){
        id -> Int4,
        turn_id -> Int4,
        question -> Text,
        incrment -> Int4,
    }
}
table! {
    continuation_responses (id){
        id -> Int4,
        poll_id -> Int4,
        user_id -> Int4,
        response -> Bool,
    }
}

table! {
    region_ownership (day){
        owner_count -> Int8,
        owners -> Array<Int4>,
        day -> Int4,
        season -> Int4,
        region -> Int4,
    }
}

table! {
    logs (id){
        id -> Int4,
        route -> Text,
        query -> Text,
        payload -> Text,
    }
}

table! {
    award_info (id){
        id -> Int4,
        name -> Text,
        info -> Text,
    }
}

table! {
    awards (id){
        id -> Int4,
        award_id -> Int4,
        user_id -> Int4,
    }
}

table! {
    audit_log (id) {
        id -> Int4,
        user_id -> Int4,
        event -> Int4,
        timestamp -> Timestamp,
        data -> Nullable<Json>,
        cip -> Nullable<Text>,
        ua -> Nullable<Text>,
    }
}

table! {
    bans (id){
        id -> Int4,
        // IP: 0
        // Username: 1
        // Prevent ban, username, for suspend flag: 2
        // Allow login without email: 3
        // Prevent ban, reddit ban: 4
        class -> Int4,
        cip -> Text,
        uname -> Text,
        ua -> Text,
    }
}

allow_tables_to_appear_in_same_query!(users, bans);

allow_tables_to_appear_in_same_query!(users, awards, award_info);
diesel::joinable!(awards -> users (user_id));
diesel::joinable!(awards -> award_info (award_id));
diesel::joinable!(statistics -> turninfo (turn_id));
diesel::joinable!(territories -> regions (region));

allow_tables_to_appear_in_same_query!(
    past_turns,
    teams,
    users,
    territory_ownership,
    territory_adjacency,
    territories,
    turninfo,
);

allow_tables_to_appear_in_same_query!(turns, territories);
allow_tables_to_appear_in_same_query!(regions, territory_ownership_with_neighbors);
allow_tables_to_appear_in_same_query!(regions, territories);

allow_tables_to_appear_in_same_query!(turns, turninfo);
allow_tables_to_appear_in_same_query!(continuation_polls, turninfo);
allow_tables_to_appear_in_same_query!(statistics, turninfo);
