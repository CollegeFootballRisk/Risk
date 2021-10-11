/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::db::DbConn;
use crate::model::{
    Claims, ClientInfo, CurrentStrength, Latest, MoveInfo, PlayerWithTurnsAndAdditionalTeam, Poll,
    PollResponse, Ratings, Stats, TeamInfo, TeamWithColors, UpdateUser,
};
use crate::schema::{
    cfbr_stats, new_turns, region_ownership, territory_adjacency, territory_ownership, users,
};
use crate::sys::SysInfo;
use diesel::prelude::*;
use diesel::result::Error;
use rocket::http::{CookieJar, Status};
use rocket::State;
use std::net::SocketAddr;
extern crate rand;
use diesel_citext::types::CiString;
use rand::{thread_rng, Rng};
use rocket::serde::json::Json;

#[get("/join?<team>", rank = 1)]
pub async fn join_team(
    team: i32,
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<String>, Status> {
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(
                config.settings.cookie_key.as_bytes(),
                cookie.value().to_string(),
            ) {
                Ok(c) => {
                    //see if user already has team, and if user has current_team
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
                        Some(users) => {
                            if users.name.to_lowercase() == c.0.user.to_lowercase() {
                                //see if user needs a new team, or a team in general
                                match users.active_team.unwrap_or_else(TeamWithColors::blank).name {
                                    None => {
                                        //check team exists
                                        match conn
                                            .run(move |connection| {
                                                TeamInfo::load(connection)
                                                    .iter()
                                                    .any(|e| e.id == team)
                                            })
                                            .await
                                        {
                                            true => {
                                                // check that team has territories
                                                match conn
                                                    .run(move |connection| {
                                                        CurrentStrength::load_id(team, connection)
                                                    })
                                                    .await
                                                {
                                                    Ok(strength) => {
                                                        if strength.territories > 0 {
                                                            match users
                                                                .team
                                                                .unwrap_or_else(
                                                                    TeamWithColors::blank,
                                                                )
                                                                .name
                                                            {
                                                                Some(_e) => {
                                                                    //merc!
                                                                    match conn.run(move |cn| update_user(
                                                                        false, c.0.id, team, cn,
                                                                    )).await {
                                                                        Ok(_e) => {
                                                                            std::result::Result::Ok(Json(
                                                                                String::from("Okay"),
                                                                            ))
                                                                        }
                                                                        Err(_e) => {
                                                                            std::result::Result::Err(
                                                                                Status::InternalServerError,
                                                                            )
                                                                        }
                                                                    }
                                                                }
                                                                None => {
                                                                    //new kid on the block
                                                                    match conn.run(move |cn| update_user(
                                                                        true, c.0.id, team, cn,
                                                                    )).await {
                                                                        Ok(_e) => {
                                                                            std::result::Result::Ok(Json(
                                                                                String::from("Okay"),
                                                                            ))
                                                                        }
                                                                        Err(_e) => {
                                                                            std::result::Result::Err(
                                                                                Status::InternalServerError,
                                                                            )
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        } else {
                                                            std::result::Result::Err(
                                                                Status::Forbidden,
                                                            )
                                                        }
                                                    }
                                                    Err(_e) => std::result::Result::Err(
                                                        Status::NotAcceptable,
                                                    ),
                                                }
                                            }
                                            false => {
                                                std::result::Result::Err(Status::NotAcceptable)
                                            }
                                        }
                                    }
                                    Some(_e) => {
                                        std::result::Result::Err(Status::InternalServerError)
                                    }
                                }
                            } else {
                                std::result::Result::Err(Status::Unauthorized)
                            }
                        }
                        None => std::result::Result::Err(Status::Unauthorized),
                    }
                }
                Err(_e) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

#[get("/my_move", rank = 1)]
//#[cfg(feature = "risk_security")]
pub async fn my_move(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    remote_addr: SocketAddr,
    config: &State<SysInfo>,
) -> Result<Json<String>, Status> {
    match conn.run(move |c| Latest::latest(c)).await {
        Ok(latest) => {
            //get cookie, verify it -> Claims (id, user, refresh_token)
            match cookies.get_private("jwt") {
                Some(cookie) => {
                    match Claims::interpret(
                        config.settings.cookie_key.as_bytes(),
                        cookie.value().to_string(),
                    ) {
                        Ok(c) => {
                            let _cinfo = ClientInfo {
                                claims: c.0.clone(),
                                ip: remote_addr.to_string(),
                            };
                            // get the territory the user has attacked
                            std::result::Result::Ok(Json(
                                conn.run(move |connection| {
                                    MoveInfo::get(latest.season, latest.day, c.0.id, connection)
                                })
                                .await
                                .territory
                                .unwrap_or_else(|| String::from("")),
                            ))
                        }
                        Err(_err) => std::result::Result::Err(Status::Unauthorized),
                    }
                }
                None => std::result::Result::Err(Status::Unauthorized),
            }
        }
        _ => std::result::Result::Err(Status::BadRequest),
    }
}

#[get("/move?<target>&<aon>", rank = 1)]
pub async fn make_move(
    target: i32,
    aon: Option<bool>,
    cookies: &CookieJar<'_>,
    conn: DbConn,
    remote_addr: SocketAddr,
    config: &State<SysInfo>,
) -> Result<Json<String>, Status> {
    match conn.run(move |c| Latest::latest(c)).await {
        Ok(latest) => {
            //get cookie, verify it -> Claims (id, user, refresh_token)
            match cookies.get_private("jwt") {
                Some(cookie) => {
                    match Claims::interpret(
                        config.settings.cookie_key.as_bytes(),
                        cookie.value().to_string(),
                    ) {
                        Ok(mut c) => {
                            let _cinfo = ClientInfo {
                                claims: c.0.clone(),
                                ip: remote_addr.to_string(),
                            };
                            // id, name Json(c.0.user)
                            //get user's team information, and whether they can make that move
                            let temp_pfix = c.0.clone();
                            let temp_ltst = Latest {
                                season: latest.season,
                                day: latest.day,
                            };
                            match conn
                                .run(move |connection| {
                                    handle_territory_info(
                                        &temp_pfix, target, temp_ltst, connection, aon,
                                    )
                                })
                                .await
                            {
                                Ok((user, multiplier)) => {
                                    //get user's current award information from CFBRisk
                                    let tmp_usname = c.0.user.clone();
                                    let awards = conn
                                        .run(move |connection| {
                                            get_cfb_points(tmp_usname, connection)
                                        })
                                        .await;
                                    //get user's current information from Reddit to ensure they still exist
                                    c.0.user.push_str(&awards.to_string());
                                    //at this point we know the user is authorized to make the action, so let's go ahead and make it
                                    let user_stats = Stats {
                                        totalTurns: user.3.unwrap_or(0),
                                        gameTurns: user.4.unwrap_or(0),
                                        mvps: user.5.unwrap_or(0),
                                        streak: user.6.unwrap_or(0),
                                        awards: awards as i32,
                                    };
                                    let user_ratings = Ratings::load(&user_stats);
                                    let user_weight: f64 = match user_ratings.overall {
                                        1 => 1.0,
                                        2 => 2.0,
                                        3 => 3.0,
                                        4 => 4.0,
                                        5 => 5.0,
                                        _ => 1.0,
                                    };
                                    let user_power: f64 = multiplier * user_weight as f64;
                                    let mut merc: bool = false;
                                    if user.0 != user.8 {
                                        merc = true;
                                    }
                                    match conn
                                        .run(move |connection| {
                                            insert_turn(
                                                &user,
                                                user_ratings,
                                                &latest,
                                                target,
                                                multiplier,
                                                user_weight,
                                                user_power,
                                                merc,
                                                connection,
                                            )
                                        })
                                        .await
                                    {
                                        Ok(_ok) => {
                                            //now we go update the user
                                            match conn
                                                .run(move |connection| {
                                                    UpdateUser::do_update(
                                                        UpdateUser {
                                                            id: user.1,
                                                            overall: user_power as i32,
                                                            turns: user_stats.totalTurns,
                                                            game_turns: user_stats.gameTurns,
                                                            mvps: user_stats.mvps,
                                                            streak: user_stats.streak,
                                                            awards: user_stats.awards,
                                                        },
                                                        connection,
                                                    )
                                                })
                                                .await
                                            {
                                                Ok(_oka) => std::result::Result::Ok(Json(
                                                    String::from("Okay"),
                                                )),
                                                Err(_e) => std::result::Result::Err(Status::Found),
                                            }
                                        }
                                        Err(_e) => {
                                            dbg!(_e);
                                            std::result::Result::Err(Status::ImATeapot)
                                        }
                                    }
                                }
                                Err(_e) => {
                                    dbg!(_e);
                                    std::result::Result::Err(Status::ImATeapot)
                                }
                            }
                        }
                        Err(_err) => std::result::Result::Err(Status::Unauthorized),
                    }
                }
                None => std::result::Result::Err(Status::Unauthorized),
            }
        }
        _ => std::result::Result::Err(Status::BadRequest),
    }
}

#[get("/polls", rank = 1)]
//#[cfg(feature = "risk_security")]
pub async fn get_polls(conn: DbConn) -> Result<Json<Vec<Poll>>, Status> {
    match conn.run(move |connection| Latest::latest(connection)).await {
        Ok(latest) => match conn
            .run(move |c| Poll::get(latest.season, latest.day, c))
            .await
        {
            Ok(polls) => std::result::Result::Ok(Json(polls)),
            Err(_E) => std::result::Result::Err(Status::InternalServerError),
        },
        Err(_E) => std::result::Result::Err(Status::InternalServerError),
    }
}

#[get("/poll/respond?<poll>&<response>", rank = 1)]
pub async fn submit_poll(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
    poll: i32,
    response: bool,
) -> Result<Json<bool>, Status> {
    // get user id
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(
                config.settings.cookie_key.as_bytes(),
                cookie.value().to_string(),
            ) {
                Ok(c) => {
                    // id, name Json(c.0.user)
                    match conn
                        .run(move |connection| {
                            PollResponse::upsert(
                                PollResponse {
                                    id: -1,
                                    poll,
                                    user_id: c.0.id,
                                    response,
                                },
                                connection,
                            )
                        })
                        .await
                    {
                        Ok(inner) => match inner {
                            1 => std::result::Result::Ok(Json(true)),
                            _ => std::result::Result::Err(Status::InternalServerError),
                        },
                        Err(_E) => std::result::Result::Err(Status::InternalServerError),
                    }
                }
                Err(_err) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

#[get("/poll/response?<poll>", rank = 1)]
pub async fn view_response(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
    poll: i32,
) -> Result<Json<Vec<PollResponse>>, Status> {
    // get user id
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(
                config.settings.cookie_key.as_bytes(),
                cookie.value().to_string(),
            ) {
                Ok(c) => {
                    // id, name Json(c.0.user)
                    match conn
                        .run(move |connection| PollResponse::get(poll, c.0.id, connection))
                        .await
                    {
                        Ok(responses) => std::result::Result::Ok(Json(responses)),
                        Err(_E) => std::result::Result::Err(Status::InternalServerError),
                    }
                }
                Err(_err) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

pub fn handleregionalownership(
    latest: &Latest,
    team: i32,
    conn: &PgConnection,
) -> QueryResult<i64> {
    use diesel::dsl::count;
    region_ownership::table
        .filter(region_ownership::season.eq(latest.season))
        .filter(region_ownership::day.eq(latest.day))
        .filter(region_ownership::owner_count.eq(1_i64))
        .filter(region_ownership::owners.contains(vec![team]))
        .select(count(region_ownership::owners))
        .first(conn)
}

pub fn handle_territory_info(
    c: &Claims,
    target: i32,
    latest: Latest,
    conn: &PgConnection,
    aon: Option<bool>,
) -> Result<
    (
        (
            i32,
            i32,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            i32,
        ),
        f64,
    ),
    String,
> {
    //get user now_playing team
    match users::table
        .filter(users::id.eq(c.id))
        .select((
            users::playing_for,
            users::id,
            users::overall,
            users::turns,
            users::game_turns,
            users::mvps,
            users::streak,
            users::awards,
            users::current_team,
        ))
        .first::<(
            i32,
            i32,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            i32,
        )>(conn)
    {
        Ok(team_id) => match get_adjacent_territory_owners(target, &latest, conn) {
            Ok(adjacent_territory_owners) => {
                match adjacent_territory_owners
                    .iter()
                    .position(|&x| x.0 == team_id.0)
                {
                    Some(_tuple_of_territory) => {
                        let pos = adjacent_territory_owners
                            .iter()
                            .position(|&x| x.1 == target);
                        match adjacent_territory_owners
                            .iter()
                            .position(|&x| x.0 != team_id.0)
                        {
                            Some(_npos) => {
                                if team_id.0 != 0 {
                                    let mut regional_multiplier =
                                        2 * handleregionalownership(&latest, team_id.0, conn)
                                            .unwrap_or(0);
                                    if regional_multiplier == 0 {
                                        regional_multiplier = 1;
                                    }
                                    let mut aon_multiplier: i32 = 1;
                                    if aon == Some(true)
                                        && get_territory_number(team_id.0, &latest, conn) == 1
                                    {
                                        let mut rng = thread_rng();
                                        aon_multiplier = 5 * rng.gen_range(0..2);
                                    }
                                    if adjacent_territory_owners[pos.unwrap()].0 == team_id.0 {
                                        Ok((
                                            team_id,
                                            1.5 * regional_multiplier as f64
                                                * f64::from(aon_multiplier),
                                        ))
                                    } else {
                                        Ok((
                                            team_id,
                                            1.0 * regional_multiplier as f64
                                                * f64::from(aon_multiplier),
                                        ))
                                    }
                                } else {
                                    let mut rng = thread_rng();
                                    let n: i32 = rng.gen_range(4..6);
                                    Ok((team_id, f64::from(n / 4)))
                                }
                            }
                            None => Err("You own all the surrounding territories".to_string()),
                        }
                    }
                    None => Err("You don't own that territory or an adjacent one".to_string()),
                }
            }
            Err(_er) => Err("You don't own that territory or an adjacent one".to_string()),
        },
        Err(_e) => Err("You don't own that territory or an adjacent one".to_string()),
    }
}

pub fn get_adjacent_territory_owners(
    target: i32,
    latest: &Latest,
    conn: &PgConnection,
) -> Result<Vec<(i32, i32)>, Error> {
    territory_adjacency::table
        .filter(territory_adjacency::territory_id.eq(target))
        .filter(territory_ownership::season.eq(latest.season))
        .filter(territory_ownership::day.eq(latest.day))
        .inner_join(
            territory_ownership::table
                .on(territory_ownership::territory_id.eq(territory_adjacency::adjacent_id)),
        )
        .select((
            territory_ownership::owner_id,
            territory_ownership::territory_id,
        ))
        .load::<(i32, i32)>(conn)
}

pub fn get_territory_number(team: i32, latest: &Latest, conn: &PgConnection) -> i32 {
    use diesel::dsl::count;
    territory_ownership::table
        .filter(territory_ownership::season.eq(latest.season))
        .filter(territory_ownership::day.eq(latest.day))
        .filter(territory_ownership::owner_id.eq(team))
        .select(count(territory_ownership::owner_id))
        .first(conn)
        .unwrap_or(0) as i32
}

pub fn get_cfb_points(name: String, conn: &PgConnection) -> i64 {
    match cfbr_stats::table
        .filter(cfbr_stats::player.eq(CiString::from(name)))
        .select(cfbr_stats::stars)
        .first(conn)
        .unwrap_or(1)
    {
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        _ => 1,
    }
}

pub fn insert_turn(
    user: &(
        i32,
        i32,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        i32,
    ),
    user_ratings: Ratings,
    latest: &Latest,
    target: i32,
    multiplier: f64,
    user_weight: f64,
    user_power: f64,
    merc: bool,
    conn: &PgConnection,
) -> QueryResult<usize> {
    diesel::insert_into(new_turns::table)
        .values((
            new_turns::user_id.eq(user.1),
            new_turns::season.eq(latest.season),
            new_turns::day.eq(latest.day),
            new_turns::territory.eq(target),
            new_turns::mvp.eq(false),
            new_turns::power.eq(user_power),
            new_turns::multiplier.eq(multiplier),
            new_turns::weight.eq(user_weight),
            new_turns::stars.eq(user_ratings.overall),
            new_turns::team.eq(user.0),
            new_turns::alt_score.eq(0),
            new_turns::merc.eq(merc),
        ))
        .on_conflict((new_turns::user_id, new_turns::season, new_turns::day))
        .do_update()
        .set((
            new_turns::territory.eq(target),
            new_turns::power.eq(user_power),
            new_turns::multiplier.eq(multiplier),
        ))
        .execute(conn)
}

pub fn update_user(new: bool, user: i32, team: i32, conn: &PgConnection) -> QueryResult<usize> {
    match new {
        true => diesel::update(users::table)
            .filter(users::id.eq(user))
            .set((users::current_team.eq(team), users::playing_for.eq(team)))
            .execute(conn),
        false => diesel::update(users::table)
            .filter(users::id.eq(user))
            .set(users::playing_for.eq(team))
            .execute(conn),
    }
}
