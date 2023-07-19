/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::db::DbConn;
use crate::sys::SysInfo;
use rocket::form::{Form, FromForm};
use rocket::http::CookieJar;
use rocket::serde::json::Json;
use rocket::State;

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusWrapper {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum EitherPorS {
    //PlayerWithTurnsAndAdditionalTeam(Box<PlayerWithTurnsAndAdditionalTeam>),
    StatusWrapper(StatusWrapper),
    String(std::string::String),
}

/// #Me
/// Retrieves all information about currently logged-in player. Should not be accessed by any
/// scraping programs.
#[openapi(skip)]
#[get("/me")]
pub(crate) async fn me(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<impl serde::ser::Serialize>, crate::Error> {
    return Ok(Json(EitherPorS::StatusWrapper(StatusWrapper {
        code: 4000,
        message: "Unauthenticated".to_owned(),
    })));
}

/// Playernames must be
#[derive(FromForm)]
pub struct Playername<'r> {
    #[field(validate = len(5..64))]
    username: &'r str,
}

#[openapi(skip)]
#[post("/me/username", data = "<username>")]
pub(crate) async fn update_username(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
    username: Form<Playername<'_>>,
) -> Result<Json<impl serde::ser::Serialize>, crate::Error> {
    return Ok(Json(EitherPorS::StatusWrapper(StatusWrapper {
        code: 4000,
        message: "Unauthenticated".to_owned(),
    })));
}
