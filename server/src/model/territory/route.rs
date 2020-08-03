use crate::db::DbConn;
use crate::model::{Latest, TerritoryHistory, TerritoryTurn, TerritoryWithNeighbors};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/territories?<day>&<season>")]
pub fn territories(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<TerritoryWithNeighbors>>, Status> {
    match Latest::latest(&conn) {
        Ok(current) => {
            let territories = TerritoryWithNeighbors::load(
                season.unwrap_or(current.season),
                day.unwrap_or(current.day),
                &conn,
            );
            if territories.len() as i32 >= 1 {
                std::result::Result::Ok(Json(territories))
            } else {
                std::result::Result::Err(Status::BadRequest)
            }
        }
        _ => std::result::Result::Err(Status::BadRequest),
    }
}

#[get("/territory/history?<territory>&<season>")]
pub fn territoryhistory(
    territory: String,
    season: i32,
    conn: DbConn,
) -> Result<Json<Vec<TerritoryHistory>>, Status> {
    let territories = TerritoryHistory::load(territory, season, &conn);
    if territories.len() as i32 >= 1 {
        std::result::Result::Ok(Json(territories))
    } else {
        std::result::Result::Err(Status::BadRequest)
    }
}

#[get("/territory/turn?<territory>&<season>&<day>")]
pub fn territory_turn(
    territory: String,
    season: i32,
    day: i32,
    conn: DbConn,
) -> Result<Json<TerritoryTurn>, Status> {
    let turn = TerritoryTurn::load(season, day, territory, &conn);
    match turn {
        Ok(turn) => std::result::Result::Ok(Json(turn)),
        _ => std::result::Result::Err(Status::BadRequest),
    }
}
