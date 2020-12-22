/*use anyhow::{Context, Error};
use hyper::{
    header::{Authorization, Bearer, UserAgent},
    net::HttpsConnector,
    Client,
};*/
//use rocket_oauth2::{OAuth2, TokenResponse};
//use rocket::response::{Debug, Redirect};
//hyper::header::{Authorization, Bearer, UserAgent},
use crate::model::{
    Claims, ClientInfo, CurrentStrength, Latest, MoveInfo, PlayerWithTurnsAndAdditionalTeam, Poll,
    PollResponse, Ratings, Stats, TeamInfo, TeamWithColors, UpdateUser,
};
use crate::schema::{new_turns, region_ownership, territory_adjacency, territory_ownership, users};
use diesel::prelude::*;
use diesel::result::Error;
use rocket::http::{Cookies, Status};
use rocket::State;
use std::net::SocketAddr;
extern crate rand;
use crate::db::DbConn;
use hyper::{net::HttpsConnector, Client};
use rand::{thread_rng, Rng};
use rocket_contrib::json::Json;
use std::io::Read;

#[cfg(feature = "risk_security")]
use crate::security::*;

#[get("/join?<team>",rank=1)]
pub fn join_team(
    team: i32,
    mut cookies: Cookies,
    conn: DbConn,
    key: State<String>,
) -> Result<Json<String>, Status> {
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                Ok(c) => {
                    //see if user already has team, and if user has current_team
                    let users = PlayerWithTurnsAndAdditionalTeam::load(
                        vec![c.0.user.clone()],
                        false,
                        &conn,
                    );
                    match users{
                        Some(users)=> {
                            if users.name.to_lowercase() == c.0.user.to_lowercase() {
                                //see if user needs a new team, or a team in general
                                match users.active_team.unwrap_or_else(TeamWithColors::blank).name {
                                    None => {
                                        //check team exists
                                        match TeamInfo::load(&conn).iter().any(|e| e.id == team) {
                                            true => {
                                                // check that team has territories
                                                match CurrentStrength::load_id(team, &conn) {
                                                    Ok(strength) => {
                                                        if strength.territories > 0 {
                                                            match users
                                                                .team
                                                                .unwrap_or_else(TeamWithColors::blank)
                                                                .name
                                                            {
                                                                Some(_e) => {
                                                                    //merc!
                                                                    match update_user(
                                                                        false, c.0.id, team, &conn,
                                                                    ) {
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
                                                                    match update_user(
                                                                        true, c.0.id, team, &conn,
                                                                    ) {
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
                                                            std::result::Result::Err(Status::Forbidden)
                                                        }
                                                    }
                                                    Err(_e) => {
                                                        std::result::Result::Err(Status::NotAcceptable)
                                                    }
                                                }
                                            }
                                            false => std::result::Result::Err(Status::NotAcceptable),
                                        }
                                    }
                                    Some(_e) => {
                                        dbg!(_e);
                                        std::result::Result::Err(Status::InternalServerError)
                                    }
                                }
                            } else {
                                std::result::Result::Err(Status::Unauthorized)
                            }
                        }
                        None => std::result::Result::Err(Status::Unauthorized)
                    }
                }
                Err(_e) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

#[get("/my_move",rank=1)]
//#[cfg(feature = "risk_security")]
pub fn my_move(
    mut cookies: Cookies,
    conn: DbConn,
    remote_addr: SocketAddr,
    key: State<String>,
) -> Result<Json<String>, Status> {
    match Latest::latest(&conn) {
        Ok(latest) => {
            //get cookie, verify it -> Claims (id, user, refresh_token)
            match cookies.get_private("jwt") {
                Some(cookie) => {
                    match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                        Ok(c) => {
                            let _cinfo = ClientInfo {
                                claims: c.0.clone(),
                                ip: remote_addr.to_string(),
                            };
                            // get the territory the user has attacked
                            std::result::Result::Ok(Json(
                                MoveInfo::get(latest.season, latest.day, c.0.id, &conn)
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

#[get("/move?<target>&<aon>",rank=1)]
//#[cfg(feature = "risk_security")]
pub fn make_move(
    target: i32,
    aon: Option<bool>,
    mut cookies: Cookies,
    conn: DbConn,
    remote_addr: SocketAddr,
    key: State<String>,
) -> Result<Json<String>, Status> {
    match Latest::latest(&conn) {
        Ok(latest) => {
            //get cookie, verify it -> Claims (id, user, refresh_token)
            match cookies.get_private("jwt") {
                Some(cookie) => {
                    match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                        Ok(mut c) => {
                            let _cinfo = ClientInfo {
                                claims: c.0.clone(),
                                ip: remote_addr.to_string(),
                            };
                            // id, name Json(c.0.user)
                            //get user's team information, and whether they can make that move
                            match handle_territory_info(
                                &c.0,
                                target,
                                Latest {
                                    season: latest.season,
                                    day: latest.day,
                                },
                                &conn, aon
                            ) {
                                Ok((user, multiplier)) => {
                                    //get user's current award information from CFBRisk
                                    let awards = get_cfb_points(c.0.user.clone());
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
                                    let user_weight: f32 = match user_ratings.overall {
                                        1 => 1.0,
                                        2 => 2.0,
                                        3 => 3.0,
                                        4 => 4.0,
                                        5 => 5.0,
                                        _ => 1.0,
                                    };
                                    let user_power: f32 = multiplier * user_weight as f32;
                                    let mut merc: bool = false;
                                    if user.0 != user.8 {
                                        merc = true;
                                    }
                                    match insert_turn(
                                        &user,
                                        user_ratings,
                                        &latest,
                                        target,
                                        multiplier,
                                        user_weight,
                                        user_power,
                                        merc,
                                        &conn,
                                    ) {
                                        Ok(_ok) => {
                                            //now we go update the user
                                            match UpdateUser::do_update(
                                                UpdateUser {
                                                    id: user.1,
                                                    overall: user_power as i32,
                                                    turns: user_stats.totalTurns,
                                                    game_turns: user_stats.gameTurns,
                                                    mvps: user_stats.mvps,
                                                    streak: user_stats.streak,
                                                    awards: user_stats.awards,
                                                },
                                                &conn,
                                            ) {
                                                Ok(_oka) => {
                                                    std::result::Result::Ok(Json(String::from(
                                                        "Okay",
                                                    )))
                                                }
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

#[get("/polls",rank=1)]
//#[cfg(feature = "risk_security")]
pub fn get_polls(conn: DbConn) -> Result<Json<Vec<Poll>>, Status> {
    match Latest::latest(&conn) {
        Ok(latest) => {
            match Poll::get(latest.season, latest.day, &conn) {
                Ok(polls) => std::result::Result::Ok(Json(polls)),
                Err(_E) => std::result::Result::Err(Status::InternalServerError),
            }
        }
        Err(_E) => std::result::Result::Err(Status::InternalServerError),
    }
}

#[get("/poll/respond?<poll>&<response>",rank=1)]
//#[cfg(feature = "risk_security")]
pub fn submit_poll(
    mut cookies: Cookies,
    conn: DbConn,
    key: State<String>,
    poll: i32,
    response: bool,
) -> Result<Json<bool>, Status> {
    // get user id
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                Ok(c) => {
                    // id, name Json(c.0.user)
                    match PollResponse::upsert(
                        PollResponse {
                            id: -1,
                            poll,
                            user_id: c.0.id,
                            response,
                        },
                        &conn,
                    ) {
                        Ok(inner) => {
                            match inner {
                                1 => std::result::Result::Ok(Json(true)),
                                _ => std::result::Result::Err(Status::InternalServerError),
                            }
                        }
                        Err(_E) => std::result::Result::Err(Status::InternalServerError),
                    }
                }
                Err(_err) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

#[get("/poll/response?<poll>",rank=1)]
//#[cfg(feature = "risk_security")]
pub fn view_response(
    mut cookies: Cookies,
    conn: DbConn,
    key: State<String>,
    poll: i32,
) -> Result<Json<Vec<PollResponse>>, Status> {
    // get user id
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                Ok(c) => {
                    // id, name Json(c.0.user)
                    match PollResponse::get(poll, c.0.id, &conn) {
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

pub fn handleregionalownership(latest: &Latest, team: i32, conn: &PgConnection) -> QueryResult<i64>{
    use diesel::dsl::count;
    region_ownership::table
    .filter(region_ownership::season.eq(latest.season))
    .filter(region_ownership::day.eq(latest.day))
    .filter(region_ownership::owner_count.eq(1 as i64))
    .filter(region_ownership::owners.contains(vec![team]))
    .select(count(region_ownership::owners))
    .first(conn)
}

pub fn handle_territory_info(
    c: &Claims,
    target: i32,
    latest: Latest,
    conn: &PgConnection, aon: Option<bool>
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
        f32,
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
        Ok(team_id) => {
            match get_adjacent_territory_owners(target, &latest, &conn) {
                Ok(adjacent_territory_owners) => {
                    match adjacent_territory_owners.iter().position(|&x| x.0 == team_id.0) {
                        Some(_tuple_of_territory) => {
                            let pos = adjacent_territory_owners.iter().position(|&x| x.1 == target);
                            match adjacent_territory_owners.iter().position(|&x| x.0 != team_id.0){
                                Some(_npos) => {
                                    if team_id.0 != 0 {
                                        let mut regional_multiplier = 2 * handleregionalownership(&latest, team_id.0, &conn).unwrap_or(0);
                                        if regional_multiplier == 0 {
                                            regional_multiplier = 1;
                                        }
                                        let mut aon_multiplier: i32 = 1;
                                        if aon == Some(true) && get_territory_number(team_id.0, &latest, &conn) == 1{
                                            let mut rng = thread_rng();
                                            aon_multiplier = 5 * rng.gen_range(0, 2);
                                        }
                                        if adjacent_territory_owners[pos.unwrap()].0 == team_id.0 {
                                            Ok((team_id, 1.5 * regional_multiplier as f32 * aon_multiplier as f32))
                                        } else {
                                            Ok((team_id, 1.0 * regional_multiplier as f32 * aon_multiplier as f32))
                                        }
                                    } else {
                                        let mut rng = thread_rng();
                                        let n: i32 = rng.gen_range(4, 6);
                                        Ok((team_id, (n / 4) as f32))
                                    }
                                }
                                None => Err("You own all the surrounding territories".to_string()),
                            }
                        }
                        None => Err("You don't own that territory or an adjacent one".to_string()),
                    }
                }
                Err(_er) => Err("You don't own that territory or an adjacent one".to_string()),
            }
        }
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
        .select((territory_ownership::owner_id, territory_ownership::territory_id))
        .load::<(i32, i32)>(conn)
}

pub fn get_territory_number(team: i32, latest: &Latest, conn: &PgConnection) -> i32{
    use diesel::dsl::count;
    territory_ownership::table
    .filter(territory_ownership::season.eq(latest.season))
    .filter(territory_ownership::day.eq(latest.day))
    .filter(territory_ownership::owner_id.eq(team))
    .select(count(territory_ownership::owner_id))
    .first(conn).unwrap_or(0) as i32
}

pub fn get_cfb_points(name: String) -> i64 {
    /*let https = HttpsConnector::new(hyper_sync_rustls::TlsClient::new());
    let client = Client::with_connector(https);
    let mut url = "https://collegefootballrisk.com/api/player?player=".to_owned();
    url.push_str(&name);
    let mut res = client.get(&url).send().unwrap();

    let mut body = String::new();
    match res.read_to_string(&mut body) {
        Ok(_ok) => {
            match serde_json::from_str(&body[0..]) {
                Ok(v) => {
                    let v: serde_json::Value = v;
                    v["ratings"]["overall"].as_i64().unwrap_or(1)
                }
                _ => 1,
            }
        }
        Err(_e) => 1,
    }*/
    5
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
    multiplier: f32,
    user_weight: f32,
    user_power: f32,
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
        true => {
            diesel::update(users::table)
                .filter(users::id.eq(user))
                .set((users::current_team.eq(team), users::playing_for.eq(team)))
                .execute(conn)
        }
        false => diesel::update(users::table).filter(users::id.eq(user)).set(users::playing_for.eq(team)).execute(conn),
    }
}
