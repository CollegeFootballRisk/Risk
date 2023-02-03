/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::catchers::Status;
use crate::db::DbConn;
use crate::model::{Latest, TerritoryHistory, TerritoryTurn, TerritoryWithNeighbors};
use rocket::serde::json::Json;

/// # Territory Ownership
/// Gives territory ownership information
#[openapi(tag = "Territories", ignore = "conn")]
#[get("/territories?<day>&<season>")]
pub(crate) async fn territories(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<TerritoryWithNeighbors>>, Status> {
    match conn.run(move |c| Latest::latest(c)).await {
        Ok(current) => {
            let territories = conn
                .run(move |c| {
                    TerritoryWithNeighbors::load(
                        season.unwrap_or(current.season),
                        day.unwrap_or(current.day),
                        c,
                    )
                })
                .await;
            if territories.len() as i32 >= 1 {
                std::result::Result::Ok(Json(territories))
            } else {
                std::result::Result::Err(Status(rocket::http::Status::BadRequest))
            }
        }
        Err(e) => {
            dbg!(e);
            std::result::Result::Err(Status(rocket::http::Status::BadRequest))
        }
    }
}

/// # Territory Owners
/// Gives a list of all owners of a territory in a given season.
#[openapi(tag = "Territories", ignore = "conn")]
#[get("/territory/history?<territory>&<season>")]
pub(crate) async fn territoryhistory(
    territory: String,
    season: i32,
    conn: DbConn,
) -> Result<Json<Vec<TerritoryHistory>>, Status> {
    let territories = conn
        .run(move |c| TerritoryHistory::load(territory, season, c))
        .await;
    if territories.len() as i32 >= 1 {
        std::result::Result::Ok(Json(territories))
    } else {
        std::result::Result::Err(Status(rocket::http::Status::BadRequest))
    }
}

/// # Territory Moves
/// Gives a list of all moves given on a particular day.
#[openapi(tag = "Territories", ignore = "conn")]
#[get("/territory/turn?<territory>&<season>&<day>")]
pub(crate) async fn territory_turn(
    territory: String,
    season: i32,
    day: i32,
    conn: DbConn,
) -> Result<Json<TerritoryTurn>, Status> {
    let turn = conn
        .run(move |c| TerritoryTurn::load(season, day, territory, c))
        .await;
    match turn {
        Ok(turn) => std::result::Result::Ok(Json(turn)),
        _ => std::result::Result::Err(Status(rocket::http::Status::BadRequest)),
    }
}
