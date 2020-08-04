use crate::db::DbConn;
use crate::model::{PlayerWithTurns, PlayerWithTurnsAndAdditionalTeam, TeamPlayer};
use rocket::http::Cookies;
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/players?<team>")]
pub fn players(team: String, conn: DbConn) -> Result<Json<Vec<TeamPlayer>>, Status> {
    let parsed_team_name: Result<String, urlencoding::FromUrlEncodingError> =
        urlencoding::decode(&team);
    match parsed_team_name {
        Ok(team) => {
            println!("{}", team);
            let users = TeamPlayer::load(vec![team], &conn);
            if users.len() as i32 >= 1 {
                std::result::Result::Ok(Json(users))
            } else {
                std::result::Result::Err(Status::NotFound)
            }
        }
        _ => std::result::Result::Err(Status::Conflict),
    }
}

#[get("/me")]
pub fn me(
    cookies: Cookies,
    conn: DbConn,
) -> Result<Json<PlayerWithTurnsAndAdditionalTeam>, Status> {
    let player: String = cookies
        .get("username")
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or_else(|| "".to_string());
    let users = PlayerWithTurnsAndAdditionalTeam::load(vec![player.clone()], false, &conn);
    if users.name.to_lowercase() == player.to_lowercase() {
        std::result::Result::Ok(Json(users))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}

#[get("/players/batch?<players>")]
pub fn player_multifetch(
    players: Option<String>,
    conn: DbConn,
) -> Result<Json<Vec<PlayerWithTurns>>, Status> {
    match players {
        Some(player) => std::result::Result::Ok(Json(PlayerWithTurns::load(
            player
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
            true,
            &conn,
        ))),
        None => std::result::Result::Err(Status::NotFound),
    }
}

#[get("/player?<player>")]
pub fn player(player: String, conn: DbConn) -> Result<Json<PlayerWithTurns>, Status> {
    let mut users = PlayerWithTurns::load(vec![player], true, &conn);
    if users.len() as i32 == 1 {
        std::result::Result::Ok(Json(users.remove(0)))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}
