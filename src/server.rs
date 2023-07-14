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
#[macro_use]
extern crate rr_proc_macros;

mod catchers;
pub mod db;
//pub mod limits;
mod error;
mod hardcode;
mod model;
mod schema;

use crate::db::DbConn;
pub use error::Error;
//use crate::limits::RateLimitGuard;
//use crate::model::{auth, player, region, stats, sys, team, territory, turn};
use crate::model::{sys};
use rocket::fs::FileServer;
//use rocket_governor::rocket_governor_catcher;
use rocket_oauth2::OAuth2;
use rocket_oauth2::OAuthConfig;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use rocket_recaptcha_v3::ReCaptcha;

#[openapi(skip)]
#[get("/my_move", rank = 0)]
//#[protect]
pub(crate) async fn my_move(
    cookies: &rocket::http::CookieJar<'_>,
    conn: DbConn,
) -> Result<rocket::serde::json::Json<i32>, crate::Error> {
    Ok(rocket::serde::json::Json(32))
}
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
        hardcode::js_api_visited,
        hardcode::js_api_odds,
        hardcode::js_api_odds_2,
        hardcode::js_api_odds_1,
        hardcode::js_api_team_players,
        hardcode::js_api_player,
        hardcode::robots,
        hardcode::favicon,
        hardcode::global_css,
        hardcode::error_ret,
    ];

    // The paths on the /api endpoint. Defined up here for cleanliness
    let api_paths = openapi_get_routes![
        my_move,
        /*      player::route::player,
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
        team::route::team_territories_visited_by_season,
        territory::route::territories,
        territory::route::territoryhistory,
        territory::route::territory_turn,
        stats::route::heat,
        stats::route::stathistory,
        stats::route::currentstrength,
        stats::route::leaderboard,
        stats::route::odds,*/
        sys::route::sysinfo,
    ];

    // The paths on the /auth endpoint. Defined up here for cleanliness
    let auth_paths = routes![
        //auth::route::make_move,
        //auth::route::my_move,
        //auth::route::join_team,
        //auth::route::view_response,
        //auth::route::submit_poll,
        //auth::route::get_polls,
        //auth::route::me,
    ];

    // Get Static Dir
    let mut static_dir = std::env::current_dir().expect("No path current");
    static_dir.push("static");

    /*
        We attach all the fairings, even if not required, those fairings must therefore be compiled
        However, we won't actually append the non-specified routes so they are in effect disabled.
    */
    let mut saturn_v = rocket::build()
        .attach(DbConn::fairing())
        //        .attach(rocket_governor::LimitHeaderGen::default())
        .register(
            "/",
            catchers![
                catchers::not_found,
                catchers::internal_error,
                catchers::not_authorized
            ], // Add rocket_governer_catcher here
        )
        .mount("/api", api_paths)
        .mount("/", FileServer::from(static_dir).rank(2))
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

    // Attach Discord routes
    #[cfg(feature = "discord")]
    {
    global_info_private.discord_config = Some(
        OAuthConfig::from_figment(saturn_v.figment(), "discord")
            .expect("No Discord Oauth available :()"),
    );
        use crate::model::auth::auth_providers::discord;
        saturn_v = saturn_v.attach(OAuth2::<discord::Discord>::fairing("discord"));
        saturn_v = saturn_v.mount("/login", routes![discord::login]);
        saturn_v = saturn_v.mount("/auth", routes![discord::callback]);
    }

    // Attach Reddit routes
    #[cfg(feature = "reddit")]
    {
            global_info_private.reddit_config = Some(
        OAuthConfig::from_figment(saturn_v.figment(), "reddit")
            .expect("No Reddit Oauth available :()"),
    );
        use crate::model::auth::auth_providers::reddit;
        saturn_v = saturn_v.attach(OAuth2::<reddit::Reddit>::fairing("reddit"));
        saturn_v = saturn_v.mount("/login", routes![reddit::login]);
        saturn_v = saturn_v.mount("/auth", routes![reddit::callback]);
    }

    saturn_v = saturn_v.manage(global_info_private);

    // Attach Captcha routes
    #[cfg(feature = "captcha")]
    {
        use crate::model::captchasvc;
        saturn_v = saturn_v.mount("/auth", routes![captchasvc::route::captchaServe]);
    }

    saturn_v = saturn_v.attach(ReCaptcha::fairing());
    saturn_v = saturn_v.attach(ReCaptcha::fairing_v2());

    saturn_v
}
