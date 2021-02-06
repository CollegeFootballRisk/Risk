#![feature(proc_macro_hygiene, decl_macro)]
#![allow(non_snake_case)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;

mod catchers;
mod db;
mod hardcode;
mod model;
mod schema;
#[cfg(feature = "risk_security")]
mod security;
use crate::model::{auth, discord, player, reddit, stats, sys, team, territory, turn, Latest};
use rocket_contrib::serve::StaticFiles;
use rocket_oauth2::OAuth2;
use rocket::config::{Config, Environment};

fn main() {
    let global_info_private = sys::SysInfo {
        name: String::from("AggieRisk"),
        base_url: String::from("Howdy"),
        version: env!("CARGO_PKG_VERSION").to_string(),
        discord: cfg!(feature = "risk_discord"),
        reddit: cfg!(feature = "risk_reddit"),
        groupme: cfg!(feature = "risk_groupme"),
    };
    dbg!(global_info_private);
    dotenv::from_filename("../.env").ok();
    let key = dotenv::var("SECRET").unwrap();
    let latest = Latest {
        season: dotenv::var("season").unwrap().parse::<i32>().unwrap(),
        day: dotenv::var("day").unwrap().parse::<i32>().unwrap(),
    };

    let mut root_paths = routes![
        hardcode::js_api_leaderboard,
        hardcode::js_api_territory,
        hardcode::js_api_territories,
        hardcode::js_api_team,
        hardcode::js_api_map,
        hardcode::js_api_team_players,
        hardcode::js_api_player,
        hardcode::robots
    ];

    let api_paths = routes![
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
    ];

    let mut auth_paths = routes![
        reddit::route::reddit_callback,
        reddit::route::reddit_logout,
        discord::route::discord_callback,
        auth::route::make_move,
        auth::route::my_move,
        auth::route::join_team,
        auth::route::view_response,
        auth::route::submit_poll,
        auth::route::get_polls,
    ];

    #[cfg(feature = "risk_captcha")]
    use crate::model::captchasvc;
    #[cfg(feature = "risk_captcha")]
    auth_paths.append(&mut routes![captchasvc::route::captchaServe]);
    #[cfg(feature = "risk_security")]
    root_paths.append(&mut crate::security::route::routes());

    rocket::ignite()
        .manage(db::init_pool())
        .manage(key)
        .manage(latest)
        .attach(OAuth2::<reddit::RedditUserInfo>::fairing("reddit"))
        .attach(OAuth2::<discord::DiscordUserInfo>::fairing("discord"))
        .register(catchers![catchers::not_found, catchers::internal_error])
        .mount("/api", api_paths)
        .mount("/auth", auth_paths)
        .mount("/login", routes![reddit::route::reddit_login, discord::route::discord_login ])
        .mount("/", StaticFiles::from("static").rank(2))
        .mount("/", root_paths)
        .launch();
}