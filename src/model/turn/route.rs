/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::db::DbConn;
use crate::error::{Error, Result};
use crate::model::{Roll, TurnInfo};
use rocket::serde::json::Json;
/// # List of Turns
/// Returns information about all past and present. Eventually will allow filtering by season.
#[openapi(tag = "Turns", ignore = "conn")]
#[get("/turns")]
pub(crate) async fn turns(conn: DbConn) -> Result<Json<Vec<TurnInfo>>> {
    let turns = conn.run(TurnInfo::load).await;
    if turns.len() as i32 >= 1 {
        Ok(Json(turns))
    } else {
        Err(Error::NotFound {})
    }
}

/// # List of Turns
/// Returns information about all past, present, and upcoming turns.
#[openapi(tag = "Turns", ignore = "conn")]
#[get("/turns/all")]
pub(crate) async fn all_turns(conn: DbConn) -> Result<Json<Vec<TurnInfo>>> {
    let turns = conn.run(TurnInfo::loadall).await;
    if turns.len() as i32 >= 1 {
        Ok(Json(turns))
    } else {
        Err(Error::NotFound {})
    }
}

/// # Audit Log
/// List of random numbers used to determine victors on a given day. Returns 502 error if no day
/// specified.
#[openapi(tag = "Turns", ignore = "conn")]
#[get("/roll/log?<season>&<day>")]
pub(crate) async fn rolllog(season: i32, day: i32, conn: DbConn) -> Result<Json<Roll>> {
    let roll = conn.run(move |c| Roll::load(season, day, c)).await;
    match roll {
        Ok(roll) => Ok(Json(roll)),
        _ => Err(Error::BadRequest {}),
    }
}
