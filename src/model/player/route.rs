use crate::db::DbConn;
use crate::model::{Claims, PlayerWithTurns, PlayerWithTurnsAndAdditionalTeam, TeamPlayer};
use rocket::http::CookieJar;
use rocket::http::Status;
use rocket::State;
use rocket_contrib::json::Json;

#[get("/players?<team>")]
pub async fn players(team: Option<String>, conn: DbConn) -> Result<Json<Vec<TeamPlayer>>, Status> {
    match team {
        Some(team) => {
            let parsed_team_name: Result<String, urlencoding::FromUrlEncodingError> =
                urlencoding::decode(&team);
            match parsed_team_name {
                Ok(team) => {
                    println!("{}", team);
                    let users = conn.run(|c| TeamPlayer::load(vec![team], c)).await;
                    if users.len() as i32 >= 1 {
                        std::result::Result::Ok(Json(users))
                    } else {
                        std::result::Result::Err(Status::NotFound)
                    }
                }
                _ => std::result::Result::Err(Status::Conflict),
            }
        }
        None => {
            let users = conn.run(|c| TeamPlayer::loadall(c)).await;
            if users.len() as i32 >= 1 {
                std::result::Result::Ok(Json(users))
            } else {
                std::result::Result::Err(Status::NotFound)
            }
        }
    }
}

#[get("/me")]
pub async fn me(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    key: State<'_, String>,
) -> Result<Json<PlayerWithTurnsAndAdditionalTeam>, Status> {
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                Ok(c) => {
                    let username = c.0.user.clone();
                    let users = conn
                        .run(move |connection| {
                            PlayerWithTurnsAndAdditionalTeam::load(
                                vec![username],
                                false,
                                connection,
                            )
                        })
                        .await;
                    match users {
                        Some(user) => {
                            if user.name.to_lowercase() == c.0.user.to_lowercase() {
                                std::result::Result::Ok(Json(user))
                            } else {
                                std::result::Result::Err(Status::NotFound)
                            }
                        }
                        None => std::result::Result::Err(Status::NotFound),
                    }
                }
                Err(_e) => std::result::Result::Err(Status::BadRequest),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

#[get("/players/batch?<players>")]
pub async fn player_multifetch(
    players: Option<String>,
    conn: DbConn,
) -> Result<Json<Vec<PlayerWithTurns>>, Status> {
    match players {
        Some(player) => {
            std::result::Result::Ok(Json(
                conn.run(move |c| {
                    PlayerWithTurns::load(
                        player.split(',').map(|s| s.to_string()).collect::<Vec<String>>(),
                        true,
                        &c,
                    )
                })
                .await,
            ))
        }
        None => std::result::Result::Err(Status::NotFound),
    }
}

#[get("/player?<player>")]
pub async fn player(
    player: String,
    conn: DbConn,
) -> Result<Json<PlayerWithTurnsAndAdditionalTeam>, Status> {
    let users = conn.run(|c| PlayerWithTurnsAndAdditionalTeam::load(vec![player], true, c)).await;
    //if users.len() as i32 == 1 {
    match users {
        Some(user) => std::result::Result::Ok(Json(user)),
        None => std::result::Result::Err(Status::NotFound),
    }
    // } else {
    //   std::result::Result::Err(Status::NotFound)
    //}
}
