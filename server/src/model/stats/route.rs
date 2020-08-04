use crate::db::DbConn;
use crate::model::{CurrentStrength, Heat, Latest, Odds, StatHistory, StatLeaderboard};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/stats/team?<team>")]
pub fn currentstrength(team: String, conn: DbConn) -> Result<Json<CurrentStrength>, Status> {
    let strength = CurrentStrength::load(team, &conn);
    match strength {
        Ok(strength) => std::result::Result::Ok(Json(strength)),
        _ => std::result::Result::Err(Status::BadRequest),
    }
}

#[get("/stats/leaderboard?<season>&<day>")]
pub fn leaderboard(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<StatLeaderboard>>, Status> {
    match (season, day) {
        (Some(season), Some(day)) => {
            let leaderboard = StatLeaderboard::load(season, day, &conn);
            match leaderboard {
                Ok(strength) => std::result::Result::Ok(Json(strength)),
                _ => std::result::Result::Err(Status::BadRequest),
            }
        }
        _ => match Latest::latest(&conn) {
            Ok(current) => {
                dbg!(&current.day - 1);
                let leaderboard = StatLeaderboard::load(current.season, current.day - 1, &conn);
                match leaderboard {
                    Ok(strength) => std::result::Result::Ok(Json(strength)),
                    _ => std::result::Result::Err(Status::BadRequest),
                }
            }
            _ => std::result::Result::Err(Status::BadRequest),
        },
    }
}

#[get("/heat?<season>&<day>")]
pub fn heat(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<Heat>>, Status> {
    match Latest::latest(&conn) {
        Ok(current) => {
            let heat = Heat::load(
                season.unwrap_or(current.season),
                day.unwrap_or(current.day - 1),
                &conn,
            );
            if heat.len() as i32 >= 1 {
                std::result::Result::Ok(Json(heat))
            } else {
                std::result::Result::Err(Status::BadRequest)
            }
        }
        _ => std::result::Result::Err(Status::BadRequest),
    }
}

#[get("/stats/team/history?<team>")]
pub fn stathistory(team: String, conn: DbConn) -> Result<Json<Vec<StatHistory>>, Status> {
    let history = StatHistory::load(team, &conn);
    if history.len() as i32 >= 1 {
        std::result::Result::Ok(Json(history))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}

#[get("/team/odds?<season>&<day>&<team>")]
pub fn odds(season: i32, day: i32, team: String, conn: DbConn) -> Result<Json<Vec<Odds>>, Status> {
    let odds = Odds::load(season, day, team, &conn);
    match odds {
        Ok(odds) => {
            if odds.len() as i32 >= 1 {
                std::result::Result::Ok(Json(odds))
            } else {
                std::result::Result::Err(Status::BadRequest)
            }
        }
        Err(e) => {
            dbg!(e);
            std::result::Result::Err(Status::BadRequest)
        }
    }
}
