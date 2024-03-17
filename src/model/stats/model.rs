/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::{heat_full, odds, statistics};
use diesel::prelude::*;
use diesel::result::Error;

use schemars::JsonSchema;

/// The statistics for a Player
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Stats {
    /// The number of turns submitted in all seasons by a Player
    pub totalTurns: i32,
    /// The number of turns submitted this seasons by a Player
    pub gameTurns: i32,
    /// The number of turns submitted in all seasons by a Player for which they were the MVP
    pub mvps: i32,
    /// The number of consecutive turns submitted by a Player
    pub streak: i32,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct StatLeaderboard {
    /// The overall rank of the team (with ties unbroken)
    pub(crate) rank: i32, //determined by number of territories desc
    /// The name of the team
    pub(crate) name: String,
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
    pub(crate) regions: i64,
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

#[derive(Serialize, Deserialize, Debug, JsonSchema, Queryable)]
/// Breakdown of the number of players having each number of overall stars.
pub(crate) struct StarBreakdown {
    /// The number of players having one star.
    pub(crate) ones: i32,
    /// The number of players having two stars.
    pub(crate) twos: i32,
    /// The number of players having three stars.
    pub(crate) threes: i32,
    /// The number of players having four stars.
    pub(crate) fours: i32,
    /// The number of players having five stars.
    pub(crate) fives: i32,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
/// Object for creating heatmaps that provides territory statistics for a given turn.
pub(crate) struct Heat {
    /// The (unstandardized) name of the territory.
    pub(crate) territory: String,
    /// The (unstandardized) name of the team that owned the territory _after_ the turn.
    pub(crate) winner: String,
    /// The number of players on all teams who submitted a move on the territory on the requested turn.
    pub(crate) players: i64,
    /// The overall power (the sum of the weight * multiplier of each player) submitted by all teams submitted for the territory on the requested turn.
    pub(crate) power: f64,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
/// Statistics pertaining to a particular team's probability of winning a territory.
pub(crate) struct Odds {
    /// The (unstandardized) name of the territory.
    pub(crate) territory: String,
    /// The (unstandardized) name of the team that owned the territory _prior_ to the turn.
    pub(crate) owner: String,
    /// The (unstandardized) name of the team that owned the territory _after_ the turn.
    pub(crate) winner: String,
    /// The (unstandardized) name of the player that won a territory for a team that turn.
    /// MVPs must have had > 0 star power (i.e. could not lose triple-or-nothing, if enabled) and must not have been flagged as an alt for that turn
    pub(crate) mvp: Option<String>,
    /// The total number of players the requested team submitted for the territory on the requested turn.
    pub(crate) players: i32,
    /// Breakdown of the overall star values (how many players of each star classification) for the players the requested team submitted for the territory on the requested turn.
    pub(crate) starBreakdown: StarBreakdown,
    /// The overall power (the sum of the weight * multiplier of each player) submitted by the requested team submitted for the territory on the requested turn.
    pub(crate) teamPower: f64,
    /// The overall power (the sum of the weight * multiplier of each player) submitted by all teams submitted for the territory on the requested turn.
    pub(crate) territoryPower: f64,
    /// The likelihood the requested team wins the territory on the requested turn.
    pub(crate) chance: f64,
}

impl Heat {
    pub(crate) fn load(season: i32, day: i32, conn: &mut PgConnection) -> Vec<Heat> {
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
    pub(crate) fn load(team: String, conn: &mut PgConnection) -> Vec<StatHistory> {
        statistics::table
            .filter(statistics::tname.eq(team))
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
    pub(crate) fn load(team: String, conn: &mut PgConnection) -> Result<CurrentStrength, Error> {
        statistics::table
            .select((
                statistics::tname,
                statistics::playercount,
                statistics::merccount,
                statistics::starpower,
                statistics::territorycount,
            ))
            .filter(statistics::tname.eq(team))
            .order(statistics::turn_id.desc())
            .first::<CurrentStrength>(conn)
    }

    // pub(crate) fn load_id(team: i32, conn: &mut PgConnection) -> Result<CurrentStrength, Error> {
    //     statistics::table
    //         .select((
    //             statistics::tname,
    //             statistics::playercount,
    //             statistics::merccount,
    //             statistics::starpower,
    //             statistics::territorycount,
    //         ))
    //         .filter(statistics::team.eq(team))
    //         .order(statistics::turn_id.desc())
    //         .first::<CurrentStrength>(conn)
    // }
    /*        statistics::table
            .select((
                statistics::tname,
                statistics::playercount,
                statistics::merccount,
                statistics::starpower,
                statistics::territorycount,
            ))
            .inner_join(turninfo::table.on(turninfo::id.eq(statistics::turn_id)))
            .filter(statistics::tname.eq(String::from(team)))
            .filter(turninfo::complete.eq(true))
            .order(statistics::turn_id.desc())
            .first::<CurrentStrength>(conn)
    }*/
}

impl StatLeaderboard {
    pub(crate) fn load(
        season: i32,
        day: i32,
        conn: &mut PgConnection,
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
        conn: &mut PgConnection,
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
