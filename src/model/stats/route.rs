/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::catchers::Status;
use crate::db::DbConn;
use crate::model::{CurrentStrength, Heat, Latest, Odds, StatHistory, StatLeaderboard};
use rocket::serde::json::Json;

/// # Team Statistics
/// Gives current team strength (from prior day's move).
#[openapi(tag = "Stats")]
#[get("/stats/team?<team>")]
pub async fn currentstrength(team: String, conn: DbConn) -> Result<Json<CurrentStrength>, Status> {
    let strength = conn.run(|c| CurrentStrength::load(team, c)).await;
    match strength {
        Ok(strength) => std::result::Result::Ok(Json(strength)),
        _ => std::result::Result::Err(Status(rocket::http::Status::BadRequest)),
    }
}

/// # Leaderboard
/// Provides team ranks on a given season/day, or on the prior day if season/day not provided.
#[openapi(tag = "Stats")]
#[get("/stats/leaderboard?<season>&<day>")]
pub async fn leaderboard(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<StatLeaderboard>>, Status> {
    match (season, day) {
        (Some(season), Some(day)) => {
            let leaderboard = conn
                .run(move |c| StatLeaderboard::load(season, day, c))
                .await;
            match leaderboard {
                Ok(strength) => std::result::Result::Ok(Json(strength)),
                _ => std::result::Result::Err(Status(rocket::http::Status::BadRequest)),
            }
        }
        _ => {
            match conn.run(|c| Latest::latest(c)).await {
                Ok(current) => {
                    //dbg!(&current.day - 1);
                    let leaderboard = conn
                        .run(move |c| StatLeaderboard::load(current.season, current.day - 1, c))
                        .await;
                    match leaderboard {
                        Ok(strength) => std::result::Result::Ok(Json(strength)),
                        _ => std::result::Result::Err(Status(rocket::http::Status::BadRequest)),
                    }
                }
                _ => std::result::Result::Err(Status(rocket::http::Status::BadRequest)),
            }
        }
    }
}

/// # Heat Map
/// Information necessary to generate a heatmap of moves on a given day. Defaults to prior day if
/// no season/day are specified.
#[openapi(tag = "Stats")]
#[get("/heat?<season>&<day>")]
pub async fn heat(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<Heat>>, Status> {
    match conn.run(|c| Latest::latest(c)).await {
        Ok(current) => {
            let heat = conn
                .run(move |c| {
                    Heat::load(
                        season.unwrap_or(current.season),
                        day.unwrap_or(current.day - 1),
                        c,
                    )
                })
                .await;
            if heat.len() as i32 >= 1 {
                std::result::Result::Ok(Json(heat))
            } else {
                std::result::Result::Err(Status(rocket::http::Status::BadRequest))
            }
        }
        _ => std::result::Result::Err(Status(rocket::http::Status::BadRequest)),
    }
}

/// # Team History
/// Gives historical team statistics for a given team.
#[openapi(tag = "Stats")]
#[get("/stats/team/history?<team>")]
pub async fn stathistory(team: String, conn: DbConn) -> Result<Json<Vec<StatHistory>>, Status> {
    let history = conn.run(|c| StatHistory::load(team, c)).await;
    if history.len() as i32 >= 1 {
        std::result::Result::Ok(Json(history))
    } else {
        std::result::Result::Err(Status(rocket::http::Status::NotFound))
    }
}

/// # Team Odds
/// Gives odds for a team on a given day
#[openapi(tag = "Teams")]
#[get("/team/odds?<season>&<day>&<team>")]
pub async fn odds(
    season: i32,
    day: i32,
    team: String,
    conn: DbConn,
) -> Result<Json<Vec<Odds>>, Status> {
    let odds = conn.run(move |c| Odds::load(season, day, team, c)).await;
    match odds {
        Ok(odds) => {
            if odds.len() as i32 >= 1 {
                std::result::Result::Ok(Json(odds))
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
