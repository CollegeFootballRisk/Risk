use crate::db::DbConn;
use crate::model::{Roll, TurnInfo};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/turns")]
pub async fn turns(conn: DbConn) -> Result<Json<Vec<TurnInfo>>, Status> {
    let turns = conn.run(|c| TurnInfo::load(c)).await;
    if turns.len() as i32 >= 1 {
        std::result::Result::Ok(Json(turns))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}

#[get("/turns/all")]
pub async fn all_turns(conn: DbConn) -> Result<Json<Vec<TurnInfo>>, Status> {
    let turns = conn.run(|c| TurnInfo::loadall(c)).await;
    if turns.len() as i32 >= 1 {
        std::result::Result::Ok(Json(turns))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}

#[get("/roll/log?<season>&<day>")]
pub async fn rolllog(season: i32, day: i32, conn: DbConn) -> Result<Json<Roll>, Status> {
    let roll = conn.run(move |c| Roll::load(season, day, c)).await;
    match roll {
        Ok(roll) => std::result::Result::Ok(Json(roll)),
        _ => std::result::Result::Err(Status::BadRequest),
    }
}
