/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::{heat_full, odds, statistics};
use diesel::prelude::*;
use diesel::result::Error;
use diesel_citext::types::CiString;
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Stats {
    pub totalTurns: i32,
    pub gameTurns: i32,
    pub mvps: i32,
    pub streak: i32,
    // pub(crate) awards: i32,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct StatLeaderboard {
    /// The overall rank of the team (with ties unbroken)
    pub(crate) rank: i32, //determined by number of territories desc
    /// The name of the team
    pub(crate) name: CiString,
    /// The logo of the team
    pub(crate) logo: String,
    /// The number of territories won by the team that turn
    pub(crate) territoryCount: i32,
    /// The number of players playing for the team that turn (excludes mercs)
    pub(crate) playerCount: i32,
    /// The number of players from eliminated teams playing for the team that turn
    pub(crate) mercCount: i32,
    /// The total amount of starpower put forth by the team that turn
    pub(crate) starPower: f64,
    /// The starpower per territory
    pub(crate) efficiency: f64, //starpower/territoryCount
    /// Number of regions held by the team that turn
    pub(crate) regions: i32,
}

#[derive(Serialize, Deserialize, JsonSchema, Queryable, Debug)]
pub(crate) struct CurrentStrength {
    pub(crate) team: String,
    pub(crate) players: i32,
    pub(crate) mercs: i32,
    pub(crate) stars: f64,
    pub(crate) territories: i32,
}

#[derive(Serialize, Deserialize, JsonSchema, Queryable)]
pub(crate) struct StatHistory {
    pub(crate) sequence: i32,
    pub(crate) season: i32,
    pub(crate) day: i32,
    pub(crate) players: i32,
    pub(crate) territories: i32,
    pub(crate) starPower: f64,
    pub(crate) effectivePower: f64,
    pub(crate) starbreakdown: StarBreakdown,
}

#[derive(Serialize, Deserialize, JsonSchema, Queryable)]
pub(crate) struct StarBreakdown {
    pub(crate) ones: i32,
    pub(crate) twos: i32,
    pub(crate) threes: i32,
    pub(crate) fours: i32,
    pub(crate) fives: i32,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct Heat {
    pub(crate) territory: CiString,
    pub(crate) winner: CiString,
    pub(crate) players: i64,
    pub(crate) power: f64,
}

#[derive(Serialize, Deserialize, JsonSchema, Queryable, Debug)]
pub(crate) struct StarBreakdown64 {
    pub(crate) ones: i32,
    pub(crate) twos: i32,
    pub(crate) threes: i32,
    pub(crate) fours: i32,
    pub(crate) fives: i32,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
pub(crate) struct Odds {
    pub(crate) territory: CiString,
    pub(crate) owner: CiString,
    pub(crate) winner: CiString,
    pub(crate) mvp: Option<CiString>,
    pub(crate) players: i32,
    pub(crate) starBreakdown: StarBreakdown64,
    pub(crate) teamPower: f64,
    pub(crate) territoryPower: f64,
    pub(crate) chance: f64,
}

impl Heat {
    pub(crate) fn load(season: i32, day: i32, conn: &PgConnection) -> Vec<Heat> {
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
    pub(crate) fn load(team: String, conn: &PgConnection) -> Vec<StatHistory> {
        statistics::table
            .filter(statistics::tname.eq(CiString::from(team)))
            .select((
                statistics::turn_id,
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
    pub(crate) fn load(team: String, conn: &PgConnection) -> Result<CurrentStrength, Error> {
        statistics::table
            .select((
                statistics::tname,
                statistics::playercount,
                statistics::merccount,
                statistics::starpower,
                statistics::territorycount,
            ))
            .filter(statistics::tname.eq(CiString::from(team)))
            .order(statistics::turn_id.desc())
            .first::<CurrentStrength>(conn)
    }

    pub(crate) fn load_id(team: i32, conn: &PgConnection) -> Result<CurrentStrength, Error> {
        statistics::table
            .select((
                statistics::tname,
                statistics::playercount,
                statistics::merccount,
                statistics::starpower,
                statistics::territorycount,
            ))
            .filter(statistics::team.eq(team))
            .order(statistics::turn_id.desc())
            .first::<CurrentStrength>(conn)
    }
    /*        statistics::table
            .select((
                statistics::tname,
                statistics::playercount,
                statistics::merccount,
                statistics::starpower,
                statistics::territorycount,
            ))
            .inner_join(turninfo::table.on(turninfo::id.eq(statistics::turn_id)))
            .filter(statistics::tname.eq(CiString::from(team)))
            .filter(turninfo::complete.eq(true))
            .order(statistics::turn_id.desc())
            .first::<CurrentStrength>(conn)
    }*/
}

impl StatLeaderboard {
    pub(crate) fn load(
        season: i32,
        day: i32,
        conn: &PgConnection,
    ) -> Result<Vec<StatLeaderboard>, Error> {
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
                statistics::regions,
            ))
            .filter(statistics::season.eq(season))
            .filter(statistics::day.eq(day))
            .order(statistics::turn_id.desc())
            .load::<StatLeaderboard>(conn)
    }
}

impl Odds {
    pub(crate) fn load(
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
            .filter(odds::team_name.eq(CiString::from(team)))
            .load::<Odds>(conn)
    }
}
