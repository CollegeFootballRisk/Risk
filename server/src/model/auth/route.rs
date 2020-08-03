/*use anyhow::{Context, Error};
use hyper::{
    header::{Authorization, Bearer, UserAgent},
    net::HttpsConnector,
    Client,
};*/

use rocket::http::{Cookies, Status};
use std::net::SocketAddr;
//use rocket::response::{Debug, Redirect};
use rocket::State;
//use rocket_oauth2::{OAuth2, TokenResponse};
use crate::schema::*;
#[cfg(feature = "risk_security")]
use crate::security::*;
use crate::model::{Claims, Latest, Ratings, Stats, UpdateUser, ClientInfo, PlayerWithTurnsAndAdditionalTeam, TeamInfo};
use diesel::prelude::*;
use diesel::result::Error;
extern crate rand;
use crate::db::DbConn;
use hyper::{
    //header::{Authorization, Bearer, UserAgent},
    net::HttpsConnector,
    Client,
};
use rand::{thread_rng, Rng};
use rocket_contrib::json::Json;
use std::io::Read; //, model::User};
#[get("/join?<team>")]
    pub fn join_team(team: i32, cookies:Cookies, conn: DbConn, key: State<String>) -> Result<Json<String>, Status> {
        match cookies.get("jwt") {
            Some(cookie) => {
                match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                    Ok(c) => {
                        //see if user already has team, and if user has current_team
                        let users = PlayerWithTurnsAndAdditionalTeam::load(vec![c.0.user.clone()], false, &conn);
                        if users.name.to_lowercase() == c.0.user.to_lowercase() {
                            //see if user needs a new team, or a team in general
                            match users.active_team.unwrap().name {
                                None => {
                                    //check team exists
                                    match TeamInfo::load(&conn).iter().any(|e| e.id == team){
                                        true => {
                                            match users.team.unwrap().name {
                                                Some(_e) =>  {
                                                    //merc!
                                                    match update_user(false, c.0.id, team, &conn) {
                                                        Ok(_e) => std::result::Result::Ok(Json(String::from("Okay"))),
                                                        Err(_e) => std::result::Result::Err(Status::InternalServerError)
                                                    }
                                                }
                                                None => {
                                                    //new kid on the block
                                                    match update_user(true, c.0.id, team, &conn) {
                                                        Ok(_e) => std::result::Result::Ok(Json(String::from("Okay"))),
                                                        Err(_e) => std::result::Result::Err(Status::InternalServerError)
                                                    }
                                                }
                                            }
                                        },
                                        false => std::result::Result::Err(Status::NotAcceptable)
                                    }
                                },
                                Some(_e) =>  {dbg!(_e); std::result::Result::Err(Status::Conflict)}
                            }
                        } else {
                            std::result::Result::Err(Status::Unauthorized)
                        }
                    }
                    Err(_e) => std::result::Result::Err(Status::Unauthorized)
                }
            },
            None => std::result::Result::Err(Status::Unauthorized)
        }
    }

#[get("/move?<target>")]
//#[cfg(feature = "risk_security")]
pub fn make_move(
    target: i32,
    cookies: Cookies,
    conn: DbConn,
    remote_addr: SocketAddr,
    key: State<String>,
    latest: State<Latest>,
) -> Result<Json<ClientInfo>, Status> {
    //get cookie, verify it -> Claims (id, user, refresh_token)
    match cookies.get("jwt") {
        Some(cookie) => {
            match Claims::interpret(key.as_bytes(), cookie.value().to_string()) {
                Ok(mut c) => {
                    let cinfo = ClientInfo {
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
                        &conn,
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
                                3 => 6.0,
                                4 => 12.0,
                                5 => 24.0,
                                _ => 1.0,
                            };
                            let user_power: f32 = multiplier * user_weight as f32;
                            let mut merc: bool = false;
                            if user.0 != user.8{
                                merc = true;
                            }
                            match insert_turn(
                                &user,
                                user_ratings,
                                latest,
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
                                        Ok(_oka) => std::result::Result::Ok(Json(cinfo)),
                                        Err(_e) => std::result::Result::Err(Status::Found),
                                    }
                                }
                                Err(_e) => { dbg!(_e); std::result::Result::Err(Status::ImATeapot)},
                            }
                        }
                        Err(_e) => { dbg!(_e); std::result::Result::Err(Status::Gone)},
                    }
                }
                Err(_err) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

fn handle_territory_info(
    c: &Claims,
    target: i32,
    latest: Latest,
    conn: &PgConnection,
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
            users::current_team
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
        Ok(team_id) => match get_adjacent_territory_owners(target, latest, &conn) {
            Ok(adjacent_territory_owners) => {
                match adjacent_territory_owners
                    .iter()
                    .position(|&x| x.0 == team_id.0)
                {
                    Some(_tuple_of_territory) => {
                        let pos = adjacent_territory_owners
                            .iter()
                            .position(|&x| x.1 == target);
                        if team_id.0 != 0 {
                            if adjacent_territory_owners[pos.unwrap()].0 == team_id.0 {
                                Ok((team_id, 1.5))
                            } else {
                                Ok((team_id, 1.0))
                            }
                        } else {
                            let mut rng = thread_rng();
                            let n: i32 = rng.gen_range(4, 6);
                            Ok((team_id, (n / 4) as f32))
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

fn get_adjacent_territory_owners(
    target: i32,
    latest: Latest,
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

fn get_cfb_points(name: String) -> i64 {
    let https = HttpsConnector::new(hyper_sync_rustls::TlsClient::new());
    let client = Client::with_connector(https);
    let mut url = "https://collegefootballrisk.com/api/player?player=".to_owned();
    url.push_str(&name);
    let mut res = client.get(&url).send().unwrap();

    let mut body = String::new();
    match res.read_to_string(&mut body) {
        Ok(_ok) => match serde_json::from_str(&body[0..]) {
            Ok(v) => {
                let v: serde_json::Value = v;
                match v["ratings"]["overall"].as_i64() {
                    Some(number) => number,
                    None => 1,
                }
            }
            _ => 1,
        },
        Err(_e) => 1,
    }
}

fn insert_turn(
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
    latest: State<Latest>,
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
            new_turns::merc.eq(merc)
        ))
        .on_conflict((new_turns::user_id, new_turns::season, new_turns::day))
        .do_update()
        .set((new_turns::territory.eq(target),
        new_turns::power.eq(user_power),
        new_turns::multiplier.eq(multiplier)))
        .execute(conn)
}

fn  update_user(new: bool, user: i32, team: i32, conn: &PgConnection) -> QueryResult<usize>{
    match new{
        true => {
            diesel::update(users::table)
            .filter(users::id.eq(user))
            .set((
                users::current_team.eq(team),
                users::playing_for.eq(team)
            ))
            .execute(conn)
        }
        false => {
            diesel::update(users::table)
            .set(
                users::playing_for.eq(team)
            )
            .execute(conn)
        }
    }
}

/*fn get_owned_territories (c: &Claims, latest: State<Latest>, conn: &PgConnection) -> Result<Vec<(Option<i32>,i32)>,Error>{
    use diesel::prelude::*;
    territory_ownership::table
    .inner_join(users::table.on(territory_ownership::owner_id.eq(users::playing_for)))
    .filter(users::id.eq(c.id))
    .filter(territory_ownership::season.eq(latest.season))
    .filter(territory_ownership::day.eq(latest.day))
    .select((users::playing_for, territory_ownership::territory_id))
    .load::<(Option<i32>, i32)>(conn)
}*/
