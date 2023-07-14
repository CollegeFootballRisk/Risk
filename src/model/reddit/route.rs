/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::{Claims, RedditUserInfo, UpsertableUser};
use crate::schema::{audit_log, ban};
use crate::{
    db::DbConn,
    model::{User, UserId},
    sys::SysInfo,
};
use chrono::Utc;

use reqwest::header::{AUTHORIZATION, USER_AGENT};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::response::{Flash, Redirect};
use rocket::time::Duration;
use rocket::State;
use rocket_oauth2::{OAuth2, TokenResponse};
use serde_json::value::Value;

#[get("/reddit")]
pub(crate) fn login(oauth2: OAuth2<RedditUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect_extras(cookies, &["identity"], &[("duration", "permanent")])
        .unwrap()
}

#[get("/logout")]
pub(crate) async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("jwt"));
    cookies.remove_private(Cookie::named("username"));
    Flash::success(Redirect::to("/"), "Successfully logged out.")
    //TODO: Implement a deletion call to reddit.
}

#[get("/reddit")]
pub(crate) async fn callback(
    token: TokenResponse<RedditUserInfo>,
    cookies: &CookieJar<'_>,
    cip: Cip,
    ua: UA,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Redirect, Status> {
    // Get user's information from Reddit
    let user_info: serde_json::Value = reqwest::Client::builder()
        .build()
        .map_err(|_| Status::BadRequest)?
        .get("https://oauth.reddit.com/api/v1/me")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .header(USER_AGENT, "AggieRiskLocal - Dev Edition")
        .send()
        .await
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?
        .json()
        .await
        .map_err(|e| {
            dbg!(e);
            Status::InternalServerError
        })?;

    // This is a rather gross way of extracting the user's name
    let uname: String = String::from(
        user_info
            .get("name")
            .ok_or(Status::BadRequest)?
            .as_str()
            .ok_or(Status::InternalServerError)?,
    );

    let uname_ban_chk = uname.clone();
    // We also want to ensure the user has a validated email with Reddit:
    if !user_info
        .get("has_verified_email")
        .unwrap_or_else(|| {
            dbg!("Error serializing user email check");
            &serde_json::json!(false)
        })
        .as_bool()
        .unwrap_or(false)
        && conn
            .run(move |c| {
                ban::table
                    .filter(ban::class.eq(3))
                    .filter(ban::uname.eq(&uname_ban_chk))
                    .count()
                    .get_result::<i64>(c)
            })
            .await
            .map_err(|_| Status::InternalServerError)?
            < 1
    {
        dbg!("User {} does not have valid email", uname);
        return std::result::Result::Ok(Redirect::to("/error/EmailError"));
    }

    // Build the `UpsertableUser` for querying the DB
    let new_user = UpsertableUser {
        uname: uname.clone(),
        platform: String::from("reddit"),
    };

    // Upsert the user
    conn.run(move |c| new_user.upsert(c))
        .await
        .map_err(|_| Status::InternalServerError)?;

    let uname_int = uname.clone();
    let uname_2_int = uname.clone();

    dbg!(user_info.get("is_suspended"));

    if user_info
        .get("is_suspended")
        .unwrap_or_else(|| {
            dbg!("Error: unable to know if user was suspended");
            &serde_json::json!(false)
        })
        .as_bool()
        .unwrap_or(false)
    {
        conn.run(move |c| UpsertableUser::flag(uname_2_int, c))
            .await
            .map_err(|e| {
                dbg!(e);
                Status::InternalServerError
            })?;
    }

    // We now retrieve the user from the database for `Cookie` creation
    // TODO: This query can in theory be removed.
    let user = conn
        .run(move |c| User::load(uname_int, "reddit".to_string(), c))
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Allow security to inform us whether the login should go through
    // i.e. is the user banned from the platform?
    audit_trail(&user, &user_info, &cip.0, &ua.0, 1, &conn).await?;

    // Cookie is valid for 30 Days
    // TODO: Pull this from Rocket settings...

    let datetime = Utc::now();
    let timestamp: usize = 2_592_000 + datetime.timestamp() as usize;

    let new_claims = Claims {
        id: user.id,
        user: user.uname.to_string(),
        token: Some(token.refresh_token().unwrap().to_string()),
        refresh_token: Some(token.access_token().to_string()),
        exp: timestamp,
    };

    // Now we build a private `Cookie` to return to the user
    // that contains the user's username (which is used in some low-sec processes)
    cookies.add_private(
        Cookie::build("username", uname)
            .same_site(SameSite::Lax)
            .domain(config.settings.base_url.clone())
            .path("/")
            .max_age(Duration::hours(720))
            .finish(),
    );

    // Now we build the private JWT `Cookie` to return to the user
    // that contains more information and is used in secure processes.
    match Claims::put(config.settings.cookie_key.as_bytes(), new_claims) {
        Ok(s) => {
            cookies.add_private(
                Cookie::build("jwt", s)
                    .same_site(SameSite::Lax)
                    .domain(config.settings.base_url.clone())
                    .path("/")
                    .max_age(Duration::hours(720))
                    .finish(),
            );
            std::result::Result::Ok(Redirect::to("/"))
        }
        _ => std::result::Result::Err(Status::InternalServerError),
    }
}

use diesel::prelude::*;
pub(crate) async fn audit_trail(
    user_information: &impl UserId,
    user_ext: &Value,
    cip_ext: &Option<String>,
    ua_ext: &Option<String>,
    event: i32,
    conn: &DbConn,
) -> Result<(), Status> {
    let user_id = user_information.id();
    let user_int = user_ext.clone();
    let cip_int: Option<String> = cip_ext.clone();
    let ua_int: Option<String> = ua_ext.clone();
    conn.run(move |connection| {
        diesel::insert_into(audit_log::table)
            .values((
                audit_log::user_id.eq(user_id),
                audit_log::event.eq(event),
                audit_log::data.eq(user_int),
                audit_log::cip.eq(cip_int),
                audit_log::ua.eq(ua_int),
            ))
            .execute(connection)
    })
    .await
    .map_err(|_| Status::InternalServerError)?;
    // Return Ok() for now
    Ok(())
}

pub(crate) struct Cip(pub Option<String>);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Cip {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(Cip(Some(
            request
                .headers()
                .get("CF-Connecting-IP")
                .collect::<String>(),
        )))
    }
}

pub(crate) struct UA(pub Option<String>);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for UA {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(UA(Some(
            request.headers().get("User-Agent").collect::<String>(),
        )))
    }
}
