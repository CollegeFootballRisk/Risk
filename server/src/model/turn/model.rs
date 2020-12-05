use crate::schema::{past_turns, rollinfo, turninfo};
use diesel::prelude::*;
use diesel::result::Error;
use serde_json::Value;

#[derive(Serialize)]
pub struct Turn {
    pub season: i32,
    pub day: i32,
    pub stars: i32,
    pub mvp: bool,
    pub territory: String,
    pub team: String,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct LastTurn {
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub stars: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct PastTurn {
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub stars: Option<i32>,
    pub mvp: bool,
    pub territory: String,    //should be string
    pub team: Option<String>, //should be string
}

#[derive(Insertable, Serialize, Deserialize)]
#[table_name = "past_turns"]
pub struct NewTurn {
    pub user_id: Option<i32>,
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub territory: Option<i32>,
    pub mvp: bool,
    pub power: Option<f64>,
    pub multiplier: Option<f64>,
    pub weight: Option<i32>,
    pub stars: Option<i32>,
    pub team: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct TurnInfo {
    pub id: i32,
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub complete: Option<bool>,
    pub active: Option<bool>,
    pub finale: Option<bool>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Latest {
    pub season: i32,
    pub day: i32,
}

#[derive(Serialize, Queryable, Deserialize)]
pub struct Roll {
    pub startTime: String,
    pub endTime: String,
    pub chaosRerolls: i32,
    pub chaosWeight: i32,
    pub territoryRolls: Value,
}

impl TurnInfo {
    pub fn load(conn: &PgConnection) -> Vec<TurnInfo> {
        turninfo::table
            .select((
                turninfo::id,
                turninfo::season,
                turninfo::day,
                turninfo::complete,
                turninfo::active,
                turninfo::finale,
            ))
            .filter(turninfo::complete.eq(true).or(turninfo::active.eq(true)))
            .load::<TurnInfo>(conn)
            .expect("Error loading TurnInfo")
    }
}

impl Latest {
    pub fn latest(conn: &PgConnection) -> Result<Latest, &str> {
        use diesel::dsl::{min,max};
        let season = turninfo::table.select(max(turninfo::season)).first::<Option<i32>>(conn);
        match season {
            Ok(season) => {
                let day = turninfo::table
                    .select(min(turninfo::day))
                    .filter(turninfo::season.eq(season.unwrap_or(0)))
                    .filter(turninfo::complete.eq(false))
                    .filter(turninfo::active.eq(true))
                    .first::<Option<i32>>(conn);
                match day {
                    Ok(day) => {
                        Ok(Latest {
                            season: season.unwrap_or(0),
                            day: day.unwrap_or(0),
                        })
                    }
                    _ => Err("Database Error"),
                }
            }
            _ => Err("Database Error"),
        }
    }
}

impl Roll {
    pub fn load(season: i32, day: i32, conn: &PgConnection) -> Result<Roll, Error> {
        rollinfo::table
            .select((
                rollinfo::rollstarttime,
                rollinfo::rollendtime,
                rollinfo::chaosrerolls,
                rollinfo::chaosweight,
                rollinfo::json_agg,
            ))
            .filter(rollinfo::day.eq(day))
            .filter(rollinfo::season.eq(season))
            .first::<Roll>(conn)
    }
}
