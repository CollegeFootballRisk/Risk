/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::catchers::Status;
use crate::db::DbConn;
use crate::model::{
    Claims, PlayerSummary, PlayerWithTurns, PlayerWithTurnsAndAdditionalTeam, TeamMerc, TeamPlayer,
    User,
};
use crate::sys::SysInfo;
use crate::Error;
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::State;

/// # Team Roster
/// Get all of the players on a team (returns all players on all teams if no team is provided).
#[openapi(tag = "Players", ignore = "conn")]
#[get("/players?<team>")]
pub(crate) async fn players(
    team: Option<String>,
    conn: DbConn,
) -> Result<Json<Vec<TeamPlayer>>, crate::Error> {
    match team {
        Some(team) => {
            let team_name: String = urlencoding::decode(&team)?.into_owned();
            //println!("{}", team);
            if let Ok(users) = conn.run(|c| TeamPlayer::load(vec![team_name], c)).await {
                std::result::Result::Ok(Json(users))
            } else {
                Error::not_found()
            }
        }
        None => {
            if let Ok(users) = conn.run(|c| TeamPlayer::loadall(c)).await {
                std::result::Result::Ok(Json(users))
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
pub(crate) async fn mercs(team: String, conn: DbConn) -> Result<Json<Vec<TeamMerc>>, crate::Error> {
    let team_name: String = urlencoding::decode(&team)?.into_owned();
    //println!("{}", team);
    if let Ok(users) = conn.run(|c| TeamMerc::load_mercs(vec![team_name], c)).await {
        std::result::Result::Ok(Json(users))
    } else {
        std::result::Result::Err(crate::Error::NotFound {})
    }
}

/// # Me
/// Retrieves all information about currently logged-in user. Should not be accessed by any
/// scraping programs.
#[openapi(skip)]
#[get("/me")]
pub(crate) async fn me(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<PlayerWithTurnsAndAdditionalTeam>, crate::Error> {
    let c = Claims::from_private_cookie(cookies, config)?;
    let username = c.0.user.clone();
    let user = conn
        .run(move |connection| {
            PlayerWithTurnsAndAdditionalTeam::load(vec![username], false, connection)
        })
        .await
        .ok_or(Error::NotFound {})?;
    if user.name.to_lowercase() == c.0.user.to_lowercase() {
        std::result::Result::Ok(Json(user))
    } else {
        std::result::Result::Err(Error::NotFound {})
    }
}

/// # Player List
/// Returns all players, but provides simplified data structure for smaller payload size. Unlike
/// other methods, this one will return before a player has been part of a roll.
#[openapi(tag = "Players", ignore = "conn")]
#[get("/players/full")]
pub(crate) async fn player_full(conn: DbConn) -> Result<Json<Vec<PlayerSummary>>, Error> {
    Ok(Json(conn.run(move |c| PlayerSummary::load(c)).await?))
}

/// # Player Batching
/// Batch retrieval of players
#[openapi(tag = "Players", ignore = "conn")]
#[get("/players/batch?<players>")]
pub(crate) async fn player_multifetch(
    players: Option<String>,
    conn: DbConn,
) -> Result<Json<Vec<PlayerWithTurns>>, Status> {
    match players {
        Some(player) => std::result::Result::Ok(Json(
            conn.run(move |c| {
                PlayerWithTurns::load(
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
        None => std::result::Result::Err(Status(rocket::http::Status::NotFound)),
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
) -> Result<Json<Vec<String>>, crate::Error> {
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
        .map_err(|_| crate::Error::NotFound {})?
        .into())
}

/// # Player Information
/// Retrieve information about individual player
#[openapi(tag = "Players", ignore = "conn")]
#[get("/player?<player>")]
pub(crate) async fn player(
    player: String,
    conn: DbConn,
) -> Result<Json<PlayerWithTurnsAndAdditionalTeam>, crate::Error> {
    let users = conn
        .run(|c| PlayerWithTurnsAndAdditionalTeam::load(vec![player], true, c))
        .await
        .ok_or(crate::Error::NotFound {})?;
    std::result::Result::Ok(Json(users))
}
