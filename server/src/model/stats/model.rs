use crate::schema::*;
use diesel::prelude::*;
use diesel::result::Error;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Stats {
    pub totalTurns: i32,
    pub gameTurns: i32,
    pub mvps: i32,
    pub streak: i32,
    pub awards: i32,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct StatLeaderboard {
    pub rank: i32, //determined by number of territories desc
    pub name: String,
    pub logo: String,
    pub territoryCount: i32,
    pub playerCount: i32,
    pub mercCount: i32,
    pub starPower: f64,
    pub efficiency: f64, //starpower/territoryCount
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct CurrentStrength {
    pub team: String,
    pub players: i32,
    pub mercs: i32,
    pub stars: f64,
    pub territories: i32,
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct StatHistory {
    pub sequence: i32,
    pub season: i32,
    pub day: i32,
    pub players: i32,
    pub territories: i32,
    pub starPower: f64,
    pub effectivePower: f64,
    pub starbreakdown: StarBreakdown,
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct StarBreakdown {
    pub ones: i32,
    pub twos: i32,
    pub threes: i32,
    pub fours: i32,
    pub fives: i32,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Heat {
    pub territory: String,
    pub winner: String,
    pub players: i64,
    pub power: f64,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct StarBreakdown64 {
    pub ones: i32,
    pub twos: i32,
    pub threes: i32,
    pub fours: i32,
    pub fives: i32,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Odds {
    pub territory: String,
    pub owner: String,
    pub winner: String,
    pub mvp: String,
    pub players: i32,
    pub starBreakdown: StarBreakdown64,
    pub teamPower: f64,
    pub territoryPower: f64,
    pub chance: f64,
}

impl Heat {
    pub fn load(season: i32, day: i32, conn: &PgConnection) -> Vec<Heat> {
        heat_full::table
            .filter(heat_full::season.eq(season))
            .filter(heat_full::day.eq(day))
            .select((
                heat_full::name,
                heat_full::owner,
                heat_full::cumulative_players,
                heat_full::cumulative_power,
            ))
            .load::<Heat>(conn)
            .expect("Error loading heat")
    }
}

impl StatHistory {
    pub fn load(team: String, conn: &PgConnection) -> Vec<StatHistory> {
        statistics::table
            .filter(statistics::tname.eq(team))
            .select((
                statistics::sequence,
                statistics::season,
                statistics::day,
                statistics::playercount,
                statistics::territorycount,
                statistics::starpower,
                statistics::effectivepower,
                (
                    statistics::ones,
                    statistics::twos,
                    statistics::threes,
                    statistics::fours,
                    statistics::fives,
                ),
            ))
            .load::<StatHistory>(conn)
            .expect("Error loading stathistory")
    }
}

impl CurrentStrength {
    pub fn load(team: String, conn: &PgConnection) -> Result<CurrentStrength, Error> {
        statistics::table
            .select((
                statistics::tname,
                statistics::playercount,
                statistics::merccount,
                statistics::starpower,
                statistics::territorycount,
            ))
            .filter(statistics::tname.eq(team))
            .order(statistics::season.desc())
            .order(statistics::day.desc())
            .first::<CurrentStrength>(conn)
    }
}

impl StatLeaderboard {
    pub fn load(season: i32, day: i32, conn: &PgConnection) -> Result<Vec<StatLeaderboard>, Error> {
        statistics::table
            .select((
                statistics::rank,
                statistics::tname,
                statistics::logo,
                statistics::territorycount,
                statistics::playercount,
                statistics::merccount,
                statistics::starpower,
                statistics::efficiency,
            ))
            .filter(statistics::season.eq(season))
            .filter(statistics::day.eq(day))
            .order(statistics::sequence.desc())
            .load::<StatLeaderboard>(conn)
    }
}

impl Odds {
    pub fn load(
        season: i32,
        day: i32,
        team: String,
        conn: &PgConnection,
    ) -> Result<Vec<Odds>, Error> {
        odds::table
            .select((
                odds::territory_name,
                odds::prev_owner,
                odds::tname,
                odds::mvp,
                odds::players,
                (
                    odds::ones,
                    odds::twos,
                    odds::threes,
                    odds::fours,
                    odds::fives,
                ),
                odds::teampower,
                odds::territorypower,
                odds::chance,
            ))
            .filter(odds::day.eq(day))
            .filter(odds::season.eq(season))
            .filter(odds::team_name.eq(team))
            .load::<Odds>(conn)
    }
}
