use crate::model::*;
use crate::schema::*;
use diesel::prelude::*;
use serde_json::Value;
use std::result::Result;

#[derive(Serialize, Queryable, Deserialize)]
pub struct Territory {
    id: i32,
    name: String,
    owner: String,
}

#[derive(Serialize, Queryable, Deserialize)]
pub struct TerritoryWithNeighbors {
    pub id: i32,
    pub name: String,
    pub owner: String,
    pub neighbors: Value,
}

#[derive(Serialize, Queryable, Deserialize)]
pub struct TerritoryHistory {
    pub season: i32,
    pub day: i32,
    pub territory: String,
    pub owner: String,
}

#[derive(Serialize, Queryable, Deserialize)]
pub struct TerritoryTurn {
    pub occupier: String,
    pub winner: String,
    pub teams: Vec<TeamInTurns>,
    pub players: Vec<PlayerInTurns>,
}

impl TerritoryWithNeighbors {
    pub fn load(season: i32, day: i32, conn: &PgConnection) -> Vec<TerritoryWithNeighbors> {
        territory_ownership_with_neighbors::table
            .filter(territory_ownership_with_neighbors::season.eq(season))
            .filter(territory_ownership_with_neighbors::day.eq(day))
            .select((
                territory_ownership_with_neighbors::territory_id,
                territory_ownership_with_neighbors::name,
                territory_ownership_with_neighbors::tname,
                territory_ownership_with_neighbors::neighbors,
            ))
            .load::<TerritoryWithNeighbors>(conn)
            .expect("Error loading neighbor territory info")
    }
}

impl TerritoryHistory {
    pub fn load(name: String, season: i32, conn: &PgConnection) -> Vec<TerritoryHistory> {
        territory_ownership_without_neighbors::table
            .filter(territory_ownership_without_neighbors::name.eq(name))
            .filter(territory_ownership_without_neighbors::season.eq(season))
            .select((
                territory_ownership_without_neighbors::season,
                territory_ownership_without_neighbors::day,
                territory_ownership_without_neighbors::name,
                territory_ownership_without_neighbors::tname,
            ))
            .load::<TerritoryHistory>(conn)
            .expect("Error loading neighbor territory info")
    }
}

impl TerritoryTurn {
    pub fn load(
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
            .filter(territory_ownership_without_neighbors::name.eq(&territory))
            .first::<(String, String)>(conn);
        let (owner, previous) = match result {
            Ok(duo) => duo,
            _ => ("NotFound".to_string(), "NotFound".to_string()),
        };
        let teams = TeamInTurns::load(&season, &day, &territory, &conn);
        let players = PlayerInTurns::load(&season, &day, &territory, &conn);
        match teams {
            Ok(teams) => {
                match players {
                    Ok(players) => {
                        Ok(TerritoryTurn {
                            occupier: owner,
                            winner: previous,
                            teams,
                            players,
                        })
                    }
                    _ => Err("Error".to_string()),
                }
            }
            _ => Err("Error".to_string()),
        }
    }
}
