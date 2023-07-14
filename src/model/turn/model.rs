/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::schema::{rollinfo, turn};
use diesel::prelude::*;
use diesel::result::Error;
use schemars::JsonSchema;
use serde_json::Value;

#[derive(Serialize)]
pub(crate) struct Turn {
    pub(crate) season: i32,
    pub(crate) day: i32,
    pub(crate) stars: i32,
    pub(crate) mvp: bool,
    pub(crate) territory: String,
    pub(crate) team: String,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct LastTurn {
    pub(crate) season: Option<i32>,
    pub(crate) day: Option<i32>,
    pub(crate) stars: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PastTurn {
    /// The Season in which the Turn was made
    pub season: i32,
    /// The Day within a Season in which the Turn was made
    pub day: i32,
    /// The number of Stars held by the Player at the time the turn was made
    pub stars: i32,
    /// Whether the Player won the Territory for their Team
    pub mvp: bool,
    /// The name of the Territory on which the Turn was placed
    pub territory: String, //should be string
    /// The name of the Team for which the Turn was placed (may not correspond
    ///  with the user's `current_team`, as it's derived from their `playing_for` Team.)
    pub team: String, //should be string
    /// The starpower of the Move before adjusting for any `multiplier`
    pub weight: f64,
    /// The product of all `multiplier`s for a Player, including, but not limited to,
    /// triple-or-nothing, defense, and/or region bonuses
    pub multiplier: f64,
    /// The total power the Player has for the Turn, after adjusting for all `multiplier`s (`weight` * `mulitplier`)
    pub power: f64,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Clone)]
pub(crate) struct TurnInfo {
    pub(crate) id: i32,
    pub(crate) season: i32,
    pub(crate) day: i32,
    pub(crate) complete: Option<bool>,
    pub(crate) active: Option<bool>,
    pub(crate) finale: Option<bool>,
    // TODO: Fix
    pub(crate) rollTime: Option<chrono::NaiveDateTime>,
    pub(crate) allOrNothingEnabled: Option<bool>,
    pub(crate) map: Option<String>,
}

#[derive(Debug, Queryable, Serialize, Deserialize, JsonSchema, Clone)]
#[allow(unreachable_pub)]
pub struct Latest {
    pub(crate) season: i32,
    pub(crate) day: i32,
    pub(crate) id: i32,
}

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct Roll {
    pub(crate) startTime: String,
    pub(crate) endTime: String,
    pub(crate) territoryRolls: Value,
}

impl TurnInfo {
    pub(crate) fn load(conn: &mut PgConnection) -> Vec<TurnInfo> {
        turn::table
            .select((
                turn::id,
                turn::season,
                turn::day,
                turn::complete,
                turn::active,
                turn::finale,
                turn::rollstarttime,
                turn::allornothingenabled,
                turn::map,
            ))
            .filter(turn::complete.eq(true).or(turn::active.eq(true)))
            .order_by(turn::id.desc()) // always desc so downstream know how to parse this consistently
            .load::<TurnInfo>(conn)
            .expect("Error loading TurnInfo")
    }

    pub(crate) fn loadall(conn: &mut PgConnection) -> Vec<TurnInfo> {
        turn::table
            .select((
                turn::id,
                turn::season,
                turn::day,
                turn::complete,
                turn::active,
                turn::finale,
                turn::rollstarttime,
                turn::allornothingenabled,
                turn::map,
            ))
            .order_by(turn::id)
            .load::<TurnInfo>(conn)
            .expect("Error loading TurnInfo")
    }

    pub(crate) fn latest(conn: &mut PgConnection) -> Result<TurnInfo, diesel::result::Error> {
        turn::table
            .select((
                turn::id,
                turn::season,
                turn::day,
                turn::complete,
                turn::active,
                turn::finale,
                turn::rollstarttime,
                turn::allornothingenabled,
                turn::map,
            ))
            .filter(turn::active.eq(Some(true)))
            .order_by(turn::id.desc())
            .first::<TurnInfo>(conn)
    }
}

impl Latest {
    pub(crate) fn latest(conn: &mut PgConnection) -> Result<Latest, diesel::result::Error> {
        turn::table
            .select((turn::season, turn::day, turn::id))
            .order(turn::id.desc())
            .first::<Latest>(conn)
    }
}

impl Roll {
    pub(crate) fn load(season: i32, day: i32, conn: &mut PgConnection) -> Result<Roll, Error> {
        rollinfo::table
            .select((
                rollinfo::rollstarttime,
                rollinfo::rollendtime,
                rollinfo::json_agg,
            ))
            .filter(rollinfo::day.eq(day))
            .filter(rollinfo::season.eq(season))
            .first::<Roll>(conn)
    }
}
