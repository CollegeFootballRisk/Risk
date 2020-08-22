table! {
    new_turns (id) {
        id -> Int4,
        user_id -> Int4,
        season -> Nullable<Int4>,
        day -> Nullable<Int4>,
        territory -> Int4,
        mvp -> Bool,
        power -> Double,
        multiplier -> Nullable<Double>,
        weight -> Nullable<Double>,
        stars -> Int4,
        team -> Int4,
        alt_score -> Int4,
        merc -> Bool,
    }
}

table! {
    past_turns (id) {
        id -> Int4,
        user_id -> Int4,
        season -> Nullable<Int4>,
        day -> Nullable<Int4>,
        territory -> Int4,
        mvp -> Bool,
        power -> Double,
        multiplier -> Nullable<Double>,
        weight -> Nullable<Double>,
        stars -> Int4,
        team -> Int4,
        alt_score -> Int4,
        merc -> Bool,
    }
}

table! {
    stats (sequence) {
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
    territory_ownership (id) {
        id -> Int4,
        territory_id -> Int4,
        territory_name -> Nullable<Text>,
        owner_id -> Int4,
        day -> Int4,
        season -> Int4,
        previous_owner_id -> Int4,
        random_number -> Double,
        mvp -> Nullable<Int4>,
    }
}

table! {
    territory_stats (id) {
        team -> Int4,
        season -> Int4,
        day -> Int4,
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
    turninfo (id) {
        id -> Int4,
        season -> Nullable<Int4>,
        day -> Nullable<Int4>,
        complete -> Nullable<Bool>,
        active -> Nullable<Bool>,
        finale -> Nullable<Bool>,
        chaosrerolls -> Nullable<Int4>,
        chaosweight -> Nullable<Int4>,
        rollendtime -> Nullable<Timestamp>,
        rollstarttime -> Nullable<Timestamp>,
    }
}
