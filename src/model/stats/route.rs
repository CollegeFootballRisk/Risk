use crate::db::DbConn;
use crate::model::{CurrentStrength, Heat, Latest, Odds, StatHistory, StatLeaderboard};
use rocket::http::Status;
use rocket_contrib::json::Json;

#[get("/stats/team?<team>")]
pub async fn currentstrength(team: String, conn: DbConn) -> Result<Json<CurrentStrength>, Status> {
    let strength = conn.run(|c| CurrentStrength::load(team, c)).await;
    match strength {
        Ok(strength) => std::result::Result::Ok(Json(strength)),
        _ => std::result::Result::Err(Status::BadRequest),
    }
}

#[get("/stats/leaderboard?<season>&<day>")]
pub async fn leaderboard(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<StatLeaderboard>>, Status> {
    match (season, day) {
        (Some(season), Some(day)) => {
            let leaderboard = conn.run(|c| StatLeaderboard::load(season, day, c)).await;
            match leaderboard {
                Ok(strength) => std::result::Result::Ok(Json(strength)),
                _ => std::result::Result::Err(Status::BadRequest),
            }
        }
        _ => {
            match conn.run(|c| Latest::latest(c)).await {
                Ok(current) => {
                    //dbg!(&current.day - 1);
                    let leaderboard = conn.run(|c| StatLeaderboard::load(current.season, current.day - 1, c)).await;
                    match leaderboard {
                        Ok(strength) => std::result::Result::Ok(Json(strength)),
                        _ => std::result::Result::Err(Status::BadRequest),
                    }
                }
                _ => std::result::Result::Err(Status::BadRequest),
            }
        }
    }
}

#[get("/heat?<season>&<day>")]
pub async fn heat(
    season: Option<i32>,
    day: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<Heat>>, Status> {
    match conn.run(|c| Latest::latest(c)).await {
        Ok(current) => {
            let heat =
                conn.run(|c| Heat::load(season.unwrap_or(current.season), day.unwrap_or(current.day - 1), c)).await;
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
pub async fn stathistory(team: String, conn: DbConn) -> Result<Json<Vec<StatHistory>>, Status> {
    let history = conn.run(|c| StatHistory::load(team, c)).await;
    if history.len() as i32 >= 1 {
        std::result::Result::Ok(Json(history))
    } else {
        std::result::Result::Err(Status::NotFound)
    }
}

#[get("/team/odds?<season>&<day>&<team>")]
pub async fn odds(season: i32, day: i32, team: String, conn: DbConn) -> Result<Json<Vec<Odds>>, Status> {
    let odds = conn.run(move |c| Odds::load(season, day, team, c)).await;
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
