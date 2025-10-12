/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::db::DbConn;
use crate::error::Result;
use crate::model::{PlayerSummary, PlayerWithTurnsAndAdditionalTeam, TeamMerc, TeamPlayer, User};
use crate::Error;
use rocket::serde::json::Json;

/// # Team Roster
/// Get all of the players on a team (returns all players on all teams if no team is provided).
#[openapi(tag = "Players", ignore = "conn")]
#[get("/players?<team>")]
pub(crate) async fn players(team: Option<String>, conn: DbConn) -> Result<Json<Vec<TeamPlayer>>> {
    match team {
        Some(team) => {
            let team_name: String = urlencoding::decode(&team)?.into_owned();
            //println!("{}", team);
            if let Ok(users) = conn.run(|c| TeamPlayer::load(vec![team_name], c)).await {
                Ok(Json(users))
            } else {
                Error::not_found()
            }
        }
        None => {
            if let Ok(users) = conn.run(TeamPlayer::loadall).await {
                Ok(Json(users))
            } else {
                Error::not_found()
            }
        }
    }
}

/// # Team Mercenary Roster
/// Get all of the mercenary players on a team (returns all players on all teams if no team is provided).
#[openapi(tag = "Players", ignore = "conn")]
#[get("/mercs?<team>")]
pub(crate) async fn mercs(team: String, conn: DbConn) -> Result<Json<Vec<TeamMerc>>> {
    let team_name: String = urlencoding::decode(&team)?.into_owned();
    //println!("{}", team);
    if let Ok(users) = conn.run(|c| TeamMerc::load_mercs(vec![team_name], c)).await {
        Ok(Json(users))
    } else {
        Error::not_found()
    }
}

/// # Player List
/// Returns all players, but provides simplified data structure for smaller payload size. Unlike
/// other methods, this one will return before a player has been part of a roll.
#[openapi(tag = "Players", ignore = "conn")]
#[get("/players/full")]
pub(crate) async fn player_full(conn: DbConn) -> Result<Json<Vec<PlayerSummary>>> {
    Ok(Json(conn.run(PlayerSummary::load).await?))
}

/// # Player Batching
///
/// Batch retrieval of players
/// - `players` should be a comma-separated list of standardized usernames without spaces.
#[openapi(tag = "Players", ignore = "conn")]
#[get("/players/batch?<players>")]
pub(crate) async fn player_multifetch(
    players: Option<String>,
    conn: DbConn,
) -> Result<Json<Vec<PlayerWithTurnsAndAdditionalTeam>>> {
    match players {
        Some(player) => Ok(Json(
            conn.run(move |c| {
                PlayerWithTurnsAndAdditionalTeam::load_all(
                    player
                        .split(',')
                        .map(std::string::ToString::to_string)
                        .collect::<Vec<String>>(),
                    true,
                    c,
                )
            })
            .await,
        )),
        None => Error::not_found(),
    }
}

/// # Player Search
/// Search for players by name
#[openapi(tag = "Players", ignore = "conn")]
#[get("/players/search?<s>&<limit>")]
pub(crate) async fn search(
    mut s: String,
    limit: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<String>>> {
    let count = match limit {
        Some(x) => {
            if x <= 50 {
                x
            } else {
                10
            }
        }
        None => 10,
    };
    s.push('%');
    Ok(conn
        .run(move |c| User::search(s, count, c))
        .await
        .map_err(|_| Error::NotFound {})?
        .into())
}

/// # Player Information
/// Retrieve information about individual player
#[openapi(tag = "Players", ignore = "conn")]
#[get("/player?<player>")]
pub(crate) async fn player(
    player: String,
    conn: DbConn,
) -> Result<Json<PlayerWithTurnsAndAdditionalTeam>> {
    let users = conn
        .run(|c| PlayerWithTurnsAndAdditionalTeam::load(vec![player], true, c))
        .await
        .ok_or(Error::NotFound {})?;
    Ok(Json(users))
}
