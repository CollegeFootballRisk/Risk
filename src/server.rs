/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(proc_macro_hygiene, decl_macro)]
// TODO: Remove the clippy lints.
#![allow(
    non_snake_case,
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::self_named_constructors
)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_okapi;

mod catchers;
pub mod db;
//pub mod limits;
mod error;
mod hardcode;
mod model;
mod schema;

pub use error::Error;
#[cfg(feature = "risk_security")]
#[rustfmt::skip]
mod security;
use crate::db::DbConn;
//use crate::limits::RateLimitGuard;
use crate::model::{auth, player, region, stats, sys, team, territory, turn};
use rocket::fs::{relative, FileServer};
//use rocket_governor::rocket_governor_catcher;
use rocket_oauth2::OAuth2;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};

#[rocket::launch]
fn rocket() -> _ {
    let mut global_info_private = sys::SysInfo::default();

    // The paths on the / endpoint. Defined up here for cleanliness
    let root_paths = routes![
        hardcode::js_api_leaderboard,
        hardcode::js_api_territory,
        hardcode::js_api_territories,
        hardcode::js_api_team,
        hardcode::js_api_map,
        hardcode::js_api_odds,
        hardcode::js_api_odds_2,
        hardcode::js_api_odds_1,
        hardcode::js_api_team_players,
        hardcode::js_api_player,
        hardcode::robots,
        hardcode::favicon,
        hardcode::global_css,
    ];

    // The paths on the /api endpoint. Defined up here for cleanliness
    let api_paths = openapi_get_routes![
        player::route::player,
        player::route::search,
        player::route::player_full,
        player::route::mercs,
        player::route::players,
        player::route::player_multifetch,
        region::route::regions,
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
        sys::route::sysinfo,
    ];

    // The paths on the /auth endpoint. Defined up here for cleanliness
    let auth_paths = routes![
        auth::route::make_move,
        auth::route::my_move,
        auth::route::join_team,
        auth::route::view_response,
        auth::route::submit_poll,
        auth::route::get_polls,
        auth::route::me,
    ];

    /*
        We attach all the fairings, even if not required, those fairings must therefore be compiled
        However, we won't actually append the non-specified routes so they are in effect disabled.
    */
    let mut saturn_v = rocket::build()
        .attach(DbConn::fairing())
        //        .attach(rocket_governor::LimitHeaderGen::default())
        .register(
            "/",
            catchers![catchers::not_found, catchers::internal_error], // Add rocket_governer_catcher here
        )
        .mount("/api", api_paths)
        .mount("/", FileServer::from(relative!("static")).rank(2))
        .mount("/", root_paths)
        .mount("/auth", auth_paths)
        .mount(
            "/docs/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../api/openapi.json".to_owned(),
                ..Default::default()
            }),
        );

    global_info_private.settings = saturn_v
        .figment()
        .extract_inner("risk")
        .expect("Cookie key not set; aborting!");
    saturn_v = saturn_v.manage(global_info_private);

    // Attach Discord routes
    #[cfg(feature = "risk_discord")]
    {
        use crate::model::discord;
        saturn_v = saturn_v.attach(OAuth2::<discord::DiscordUserInfo>::fairing("discord"));
        saturn_v = saturn_v.mount("/login", routes![discord::route::login]);
        saturn_v = saturn_v.mount("/auth", routes![discord::route::callback]);
    }

    // Attach Reddit routes
    #[cfg(feature = "risk_reddit")]
    {
        use crate::model::reddit;
        saturn_v = saturn_v.attach(OAuth2::<reddit::RedditUserInfo>::fairing("reddit"));
        saturn_v = saturn_v.mount("/login", routes![reddit::route::login]);
        saturn_v = saturn_v.mount(
            "/auth",
            routes![reddit::route::callback, reddit::route::logout],
        );
    }

    // Attach Captcha routes
    #[cfg(feature = "risk_captcha")]
    {
        use crate::model::captchasvc;
        saturn_v = saturn_v.mount("/auth", routes![captchasvc::route::captchaServe]);
    }

    // Attach Security routes
    #[cfg(feature = "risk_security")]
    {
        saturn_v = saturn_v.mount("/", crate::security::route::routes());
    }

    saturn_v
}

/* use serde_derive::Deserialize;
use std::fs;
use std::path::Path;
use xdg::BaseDirectories;
//use rocket::config::{Config, Environment};

#[derive(Deserialize, Debug)]
struct Config {
    name: String,
    base_uri: Option<u16>,
    port: i32,
    keys: Keys,
    postgres_string: String,
    log: Option<String>,
    workers: Option<i32>,
    keep_alive: Option<i32>,
    timeout: Option<i32>,
    tls: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Keys {
    reddit: Option<OAuthCredentials>,
    discord: Option<OAuthCredentials>,
    groupme: Option<OAuthCredentials>,
}

#[derive(Deserialize, Debug)]
struct OAuthCredentials {
    client_id: String,
    client_secret: String,
    auth_uri: Option<String>,
    token_uri: Option<String>,
}

fn getConfig() -> Result<(), std::io::Error> {
    let path = BaseDirectories::with_prefix("rust-risk")?;
    let config_filename = path.place_config_file("config.toml")?;
    if Path::new(&config_filename).exists() {
        let contents = fs::read_to_string(config_filename)?;
        let config: Config = toml::from_str(&contents)?;
        dbg!(config);
        Ok(())
    } else {
        /*let config = Config::default();
        let toml = toml::to_string(&config).unwrap();
        let mut file = File::create(&config_filename)?;
        file.write_all(&toml.as_bytes())?;*/
        dbg!("No config file!");
        Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"))
    }
}
*/
