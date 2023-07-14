/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::{PlayerInTurns, TeamInTurns};
use crate::schema::{
    team, territory, territory_ownership, territory_ownership_with_neighbors,
    territory_ownership_without_neighbors, turninfo,
};
use diesel::prelude::*;

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

#[derive(Debug, Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct TerritoryWithNeighbors {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) owner: String,
    pub(crate) region: i32,
    pub(crate) region_name: String,
    pub(crate) neighbors: Option<Value>,
}

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct TerritoryHistory {
    pub(crate) season: i32,
    pub(crate) day: i32,
    pub(crate) territory: String,
    pub(crate) owner: String,
}

#[derive(Serialize, Queryable, Deserialize, JsonSchema, Debug)]
pub(crate) struct TerritoryTurn {
    pub(crate) occupier: String,
    pub(crate) winner: String,
    pub(crate) team: Vec<TeamInTurns>,
    pub(crate) players: Vec<PlayerInTurns>,
}

impl TerritoryWithNeighbors {
    pub(crate) fn load(
        season: i32,
        day: i32,
        conn: &mut PgConnection,
    ) -> Vec<TerritoryWithNeighbors> {
        territory_ownership_with_neighbors::table
            .filter(territory_ownership_with_neighbors::season.eq(season))
            .filter(territory_ownership_with_neighbors::day.eq(day))
            .select((
                territory_ownership_with_neighbors::territory_id,
                territory_ownership_with_neighbors::name,
                territory_ownership_with_neighbors::tname,
                territory_ownership_with_neighbors::region,
                territory_ownership_with_neighbors::region_name,
                territory_ownership_with_neighbors::neighbors,
            ))
            .load::<TerritoryWithNeighbors>(conn)
            .expect("Error loading neighbor territory info")
    }
}

#[derive(Serialize, Deserialize, Debug, QueryableByName)]
struct TemporaryInts {
    #[diesel(sql_type = diesel::sql_types::Integer)]
    min: i32,
}

impl TerritoryHistory {
    pub(crate) fn load(
        name: String,
        season: i32,
        conn: &mut PgConnection,
    ) -> Vec<TerritoryHistory> {
        territory_ownership_without_neighbors::table
            .filter(territory_ownership_without_neighbors::name.eq(name))
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

    pub(crate) fn load_by_team_in_season(
        team: String,
        season: i32,
        conn: &mut PgConnection,
    ) -> Result<Vec<TerritoryHistory>, diesel::result::Error> {
        territory_ownership::table
            .inner_join(team::table.on(territory_ownership::owner_id.eq(team::id)))
            .inner_join(turninfo::table.on(territory_ownership::turn_id.eq(turninfo::id)))
            .inner_join(
                territory::table.on(territory_ownership::territory_id.eq(territory::id)),
            )
            .filter(turninfo::season.eq(season))
            .filter(team::tname.eq(team))
            .filter(
                territory_ownership::id.eq_any(
                    diesel::sql_query(
                        "select min(territory_ownership.id) 
                        from territory_ownership 
                        inner join turninfo 
                        on turninfo.id = territory_ownership.turn_id 
                        where season = $1 
                        group by season, owner_id, territory_id",
                    )
                    .bind::<diesel::sql_types::Integer, _>(season)
                    .load::<TemporaryInts>(conn)?
                    .iter()
                    .map(|v| v.min)
                    .collect::<Vec<i32>>(),
                ),
            )
            .select((
                turninfo::season,
                turninfo::day,
                territory::name,
                team::tname,
            ))
            .load::<TerritoryHistory>(conn)
    }
}

impl TerritoryTurn {
    pub(crate) fn load(
        season: i32,
        day: i32,
        territory: String,
        conn: &mut PgConnection,
    ) -> Result<TerritoryTurn, String> {
        let result = territory_ownership_without_neighbors::table
            .select((
                territory_ownership_without_neighbors::owner,
                territory_ownership_without_neighbors::prev_owner,
            ))
            .filter(territory_ownership_without_neighbors::day.eq(&day))
            .filter(territory_ownership_without_neighbors::season.eq(&season))
            .filter(territory_ownership_without_neighbors::name.eq(territory.clone()))
            .first::<(String, String)>(conn);
        let (owner, previous) = match result {
            Ok(duo) => duo,
            _ => (String::from("NotFound"), String::from("NotFound")),
        };
        let team = TeamInTurns::load(&season, &day, &territory, conn);
        let players = PlayerInTurns::load(&season, &day, &territory, conn);
        match team {
            Ok(team) => match players {
                Ok(players) => Ok(TerritoryTurn {
                    occupier: owner,
                    winner: previous,
                    team,
                    players,
                }),
                _ => Err("Error".to_string()),
            },
            _ => Err("Error".to_string()),
        }
    }
}
