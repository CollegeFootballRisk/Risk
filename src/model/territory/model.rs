/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::{PlayerInTurns, TeamInTurns};
use crate::schema::{
    regions, territory_ownership_with_neighbors, territory_ownership_without_neighbors,
};
use diesel::prelude::*;
use diesel_citext::types::CiString;
use schemars::JsonSchema;
use serde_json::Value;
use std::result::Result;

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct Territory {
    id: i32,
    name: String,
    owner: String,
    region: i32,
    region_name: i32,
}

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct TerritoryWithNeighbors {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) owner: String,
    pub(crate) region: i32,
    pub(crate) region_name: String,
    pub(crate) neighbors: Value,
}

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct TerritoryHistory {
    pub(crate) season: i32,
    pub(crate) day: i32,
    pub(crate) territory: String,
    pub(crate) owner: String,
}

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct TerritoryTurn {
    pub(crate) occupier: String,
    pub(crate) winner: String,
    pub(crate) teams: Vec<TeamInTurns>,
    pub(crate) players: Vec<PlayerInTurns>,
}

impl TerritoryWithNeighbors {
    pub(crate) fn load(season: i32, day: i32, conn: &PgConnection) -> Vec<TerritoryWithNeighbors> {
        territory_ownership_with_neighbors::table
            .filter(territory_ownership_with_neighbors::season.eq(season))
            .filter(territory_ownership_with_neighbors::day.eq(day))
            .inner_join(
                regions::table.on(regions::id.eq(territory_ownership_with_neighbors::region)),
            )
            .select((
                territory_ownership_with_neighbors::territory_id,
                territory_ownership_with_neighbors::name,
                territory_ownership_with_neighbors::tname,
                territory_ownership_with_neighbors::region,
                regions::name,
                territory_ownership_with_neighbors::neighbors,
            ))
            .load::<TerritoryWithNeighbors>(conn)
            .expect("Error loading neighbor territory info")
    }
}

impl TerritoryHistory {
    pub(crate) fn load(name: String, season: i32, conn: &PgConnection) -> Vec<TerritoryHistory> {
        territory_ownership_without_neighbors::table
            .filter(territory_ownership_without_neighbors::name.eq(CiString::from(name)))
            .filter(territory_ownership_without_neighbors::season.eq(season))
            .select((
                territory_ownership_without_neighbors::season,
                territory_ownership_without_neighbors::day,
                territory_ownership_without_neighbors::name,
                territory_ownership_without_neighbors::owner,
            ))
            .load::<TerritoryHistory>(conn)
            .expect("Error loading neighbor territory info")
    }
}

impl TerritoryTurn {
    pub(crate) fn load(
        season: i32,
        day: i32,
        territory: String,
        conn: &PgConnection,
    ) -> Result<TerritoryTurn, String> {
        let result = territory_ownership_without_neighbors::table
            .select((
                territory_ownership_without_neighbors::owner,
                territory_ownership_without_neighbors::prev_owner,
            ))
            .filter(territory_ownership_without_neighbors::day.eq(&day))
            .filter(territory_ownership_without_neighbors::season.eq(&season))
            .filter(
                territory_ownership_without_neighbors::name.eq(CiString::from(territory.clone())),
            )
            .first::<(CiString, CiString)>(conn);
        let (owner, previous) = match result {
            Ok(duo) => duo,
            _ => (CiString::from("NotFound"), CiString::from("NotFound")),
        };
        let teams = TeamInTurns::load(&season, &day, &territory, conn);
        let players = PlayerInTurns::load(&season, &day, &territory, conn);
        match teams {
            Ok(teams) => match players {
                Ok(players) => Ok(TerritoryTurn {
                    occupier: owner.into(),
                    winner: previous.into(),
                    teams,
                    players,
                }),
                _ => Err("Error".to_string()),
            },
            _ => Err("Error".to_string()),
        }
    }
}
