use crate::db::DbConn;
use crate::model::{TeamInfo, TeamPlayerMoves};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/teams")]
pub fn teams(conn: DbConn) -> Result<Json<Vec<TeamInfo>>, Status> {
    let teams = TeamInfo::load(&conn);
    if teams.len() as i32 >= 1 {
        std::result::Result::Ok(Json(teams))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}

#[get("/team/players?<season>&<day>&<team>")]
pub fn teamplayersbymoves(
    season: i32,
    day: i32,
    team: Option<String>,
    conn: DbConn,
) -> Result<Json<Vec<TeamPlayerMoves>>, Status> {
    let moves = TeamPlayerMoves::load(season, day, team, &conn);
    if moves.len() as i32 >= 1 {
        std::result::Result::Ok(Json(moves))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}
