#![feature(proc_macro_hygiene, decl_macro)]
#![allow(non_snake_case)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

use rocket_contrib::serve::StaticFiles;
mod model;
use crate::model::{auth, captchasvc, player, reddit, stats, team, territory, turn, Latest};
use rocket::http::Cookies;
use rocket::request::{self, FromRequest, Request};
use rocket::response::NamedFile;
use rocket::{routes, Outcome};
mod catchers;
mod db;
mod schema;
use rocket_oauth2::OAuth2;
use std::{fs, thread, time};
#[cfg(feature = "risk_security")]
mod security;

struct User {
    pub username: String,
}

impl<'a, 'r> FromRequest<'a, 'r> for User {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<User, ()> {
        let mut cookies = request.guard::<Cookies<'_>>().expect("request cookies");
        if let Some(cookie) = cookies.get_private("username") {
            return Outcome::Success(User {
                username: cookie.value().to_string(),
            });
        }

        Outcome::Forward(())
    }
}

fn main() {
    let _child = thread::spawn(move || {
        start();
    });
    dbg!("test");
    let ten_millis = time::Duration::from_millis(1000 * 15 * 60);
    let metadata = fs::metadata("../.env").unwrap();
    if let Ok(time) = metadata.modified() {
        let mut last_tv_sec = time;
        loop {
            // only check once per 15 minutes, unless it's circa 10 pm, iwc we check every 5s until update
            thread::sleep(ten_millis);
            let metadata = fs::metadata("../.env").unwrap();
            if let Ok(time) = metadata.modified() {
                if last_tv_sec < time {
                    println!("{:?}", time);
                    last_tv_sec = time;
                }
            }
        }
    }
}

#[get("/robots.txt")]
fn robots() -> String {
    "User-agent: *
    Disallow:
    Disallow: /auth/"
        .to_string()
}

// These are JS Routes
#[get("/<_data>", rank = 1)]
fn js_api_leaderboard(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/territory/<_data>", rank = 1)]
fn js_api_territory(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/team/<_data>", rank = 1)]
fn js_api_team(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/team/<_data>/players", rank = 1)]
fn js_api_team_players(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/player/<_data>", rank = 1)]
fn js_api_player(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/map/<_data>/<_data2>", rank = 1)]
fn js_api_map(_data: Option<String>, _data2: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/territory/<_territory>/<_data>/<_data2>", rank = 1)]
fn js_api_territories(
    _data: Option<String>,
    _territory: Option<String>,
    _data2: Option<String>,
) -> NamedFile {
    NamedFile::open("static/index.html").ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

fn start() {
    dotenv::from_filename("../.env").ok();
    let key = dotenv::var("SECRET").unwrap();
    let latest = Latest {
        season: dotenv::var("season").unwrap().parse::<i32>().unwrap(),
        day: dotenv::var("day").unwrap().parse::<i32>().unwrap(),
    };

    #[cfg(not(feature = "risk_security"))]
    rocket::ignite()
        .manage(db::init_pool())
        .manage(key)
        .manage(latest)
        .attach(OAuth2::<reddit::RedditUserInfo>::fairing("reddit"))
        .register(catchers![catchers::not_found, catchers::internal_error])
        .mount("/api", routes![
            player::route::player,
            player::route::me,
            player::route::players,
            player::route::player_multifetch,
            turn::route::turns,
            turn::route::all_turns,
            turn::route::rolllog,
            team::route::teams,
            team::route::teamplayersbymoves,
            territory::route::territories,
            territory::route::territoryhistory,
            territory::route::territory_turn,
            stats::route::heat,
            stats::route::stathistory,
            stats::route::currentstrength,
            stats::route::leaderboard,
            stats::route::odds,
        ])
        .mount("/auth", routes![
            reddit::route::reddit_callback,
            reddit::route::reddit_logout,
            captchasvc::route::captchaServe,
            auth::route::make_move,
            auth::route::my_move,
            auth::route::join_team,
            auth::route::view_response,
            auth::route::submit_poll,
            auth::route::get_polls,
        ])
        .mount("/login", routes![reddit::route::reddit_login])
        .mount("/", StaticFiles::from("static").rank(2))
        .mount("/", routes![
            js_api_leaderboard,
            js_api_territory,
            js_api_territories,
            js_api_team,
            js_api_map,
            js_api_team_players,
            js_api_player,
            robots
        ])
        .launch();

    #[cfg(feature = "risk_security")]
    rocket::ignite()
        .manage(db::init_pool())
        .manage(key)
        .manage(latest)
        .attach(OAuth2::<reddit::RedditUserInfo>::fairing("reddit"))
        .register(catchers![catchers::not_found, catchers::internal_error])
        .mount("/api", routes![
            player::route::player,
            player::route::me,
            player::route::players,
            player::route::player_multifetch,
            turn::route::turns,
            turn::route::all_turns,
            turn::route::rolllog,
            team::route::teams,
            team::route::teamplayersbymoves,
            territory::route::territories,
            territory::route::territoryhistory,
            territory::route::territory_turn,
            stats::route::heat,
            stats::route::stathistory,
            stats::route::currentstrength,
            stats::route::leaderboard,
            stats::route::odds,
        ])
        .mount("/", routes![security::route::one, security::route::two, security::route::three])
        .mount("/auth", routes![
            reddit::route::reddit_callback,
            reddit::route::reddit_logout,
            captchasvc::route::captchaServe,
            auth::route::make_move,
            auth::route::my_move,
            auth::route::join_team,
            auth::route::view_response,
            auth::route::submit_poll,
            auth::route::get_polls,
        ])
        .mount("/login", routes![reddit::route::reddit_login])
        .mount("/", StaticFiles::from("static").rank(2))
        .mount("/", routes![
            js_api_leaderboard,
            js_api_territory,
            js_api_territories,
            js_api_team,
            js_api_map,
            js_api_team_players,
            js_api_player,
            robots
        ])
        .launch();
}
