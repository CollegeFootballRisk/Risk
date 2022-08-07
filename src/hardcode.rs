/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rocket::fs::NamedFile;

#[get("/robots.txt")]
pub(crate) async fn robots() -> String {
    "User-agent: *
    Disallow:
    Disallow: /auth/"
        .to_string()
}

//Favicon
#[get("/favicon.png", rank = 0)]
pub(crate) async fn favicon() -> NamedFile {
    NamedFile::open("static/favicon.png").await.ok().unwrap()
}

// CSS
#[get("/global.css", rank = 0)]
pub(crate) async fn global_css() -> NamedFile {
    NamedFile::open("static/global.css").await.ok().unwrap()
}

// These are JS Routes
#[get("/<_data>", rank = 1)]
pub(crate) async fn js_api_leaderboard(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/territory/<_data>", rank = 1)]
pub(crate) async fn js_api_territory(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/team/<_data>", rank = 1)]
pub(crate) async fn js_api_team(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/team/<_data>/players", rank = 1)]
pub(crate) async fn js_api_team_players(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/player/<_data>", rank = 1)]
pub(crate) async fn js_api_player(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/map/<_data>/<_data2>", rank = 1)]
pub(crate) async fn js_api_map(_data: Option<String>, _data2: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/territory/<_territory>/<_data>/<_data2>", rank = 1)]
pub(crate) async fn js_api_territories(
    _data: Option<String>,
    _territory: Option<String>,
    _data2: Option<String>,
) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}
