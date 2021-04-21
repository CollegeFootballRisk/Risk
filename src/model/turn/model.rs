use crate::schema::{rollinfo, turninfo};
use diesel::prelude::*;
use diesel::result::Error;
use schemars::JsonSchema;
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

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub struct LastTurn {
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub stars: Option<i32>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PastTurn {
    pub season: i32,
    pub day: i32,
    pub stars: i32,
    pub mvp: bool,
    pub territory: String, //should be string
    pub team: String,      //should be string
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub struct TurnInfo {
    pub id: i32,
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub complete: Option<bool>,
    pub active: Option<bool>,
    pub finale: Option<bool>,
    pub rollTime: Option<crate::catchers::NaiveDateTime>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Latest {
    pub season: i32,
    pub day: i32,
}

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
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
                turninfo::rollstarttime,
            ))
            .filter(turninfo::complete.eq(true).or(turninfo::active.eq(true)))
            .order_by(turninfo::id)
            .load::<TurnInfo>(conn)
            .expect("Error loading TurnInfo")
    }

    pub fn loadall(conn: &PgConnection) -> Vec<TurnInfo> {
        turninfo::table
            .select((
                turninfo::id,
                turninfo::season,
                turninfo::day,
                turninfo::complete,
                turninfo::active,
                turninfo::finale,
                turninfo::rollstarttime,
            ))
            .order_by(turninfo::id)
            .load::<TurnInfo>(conn)
            .expect("Error loading TurnInfo")
    }
}

impl Latest {
    pub fn latest(conn: &PgConnection) -> Result<Latest, String> {
        use diesel::dsl::{max, min};
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
                        match (season, day) {
                            (Some(season), Some(day)) => {
                                Ok(Latest {
                                    season,
                                    day,
                                })
                            }
                            (Some(season), None) => {
                                let dayz = turninfo::table
                                    .select(max(turninfo::day))
                                    .filter(turninfo::season.eq(season))
                                    .first::<Option<i32>>(conn);
                                let day: i32 = dayz.unwrap_or(Some(0)).unwrap_or(0);
                                Ok(Latest {
                                    season,
                                    day,
                                })
                            }
                            _ => {
                                Ok(Latest {
                                    season: 0,
                                    day: 0,
                                })
                            }
                        }
                    }
                    _ => Err("Database Error".to_owned()),
                }
            }
            _ => Err("Database Error".to_owned()),
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
