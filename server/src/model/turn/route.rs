use crate::db::DbConn;
use crate::model::{Roll, TurnInfo};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/turns")]
pub fn turns(conn: DbConn) -> Result<Json<Vec<TurnInfo>>, Status> {
    let turns = TurnInfo::load(&conn);
    if turns.len() as i32 >= 1 {
        std::result::Result::Ok(Json(turns))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}

#[get("/roll/log?<season>&<day>")]
pub fn rolllog(season: i32, day: i32, conn: DbConn) -> Result<Json<Roll>, Status> {
    let roll = Roll::load(season, day, &conn);
    match roll {
        Ok(roll) => std::result::Result::Ok(Json(roll)),
        _ => std::result::Result::Err(Status::BadRequest),
    }
}
