/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::catchers::Status;
use crate::db::DbConn;
use crate::model::{Roll, TurnInfo};
use rocket::serde::json::Json;
/// # List of Turns
/// Returns information about all past and present. Eventually will allow filtering by season.
#[openapi(tag = "Turns")]
#[get("/turns")]
pub async fn turns(conn: DbConn) -> Result<Json<Vec<TurnInfo>>, Status> {
    let turns = conn.run(|c| TurnInfo::load(c)).await;
    if turns.len() as i32 >= 1 {
        std::result::Result::Ok(Json(turns))
    } else {
        std::result::Result::Err(Status(rocket::http::Status::NotFound))
    }
}

/// # List of Turns
/// Returns information about all past, present, and upcoming turns.
#[openapi(tag = "Turns")]
#[get("/turns/all")]
pub async fn all_turns(conn: DbConn) -> Result<Json<Vec<TurnInfo>>, Status> {
    let turns = conn.run(|c| TurnInfo::loadall(c)).await;
    if turns.len() as i32 >= 1 {
        std::result::Result::Ok(Json(turns))
    } else {
        std::result::Result::Err(Status(rocket::http::Status::NotFound))
    }
}

/// # Audit Log
/// List of random numbers used to determine victors on a given day. Returns 502 error if no day
/// specified.
#[openapi(tag = "Turns")]
#[get("/roll/log?<season>&<day>")]
pub async fn rolllog(season: i32, day: i32, conn: DbConn) -> Result<Json<Roll>, Status> {
    let roll = conn.run(move |c| Roll::load(season, day, c)).await;
    match roll {
        Ok(roll) => std::result::Result::Ok(Json(roll)),
        _ => std::result::Result::Err(Status(rocket::http::Status::BadRequest)),
    }
}
