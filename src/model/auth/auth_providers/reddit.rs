/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rocket::time::Duration;
use crate::db::DbConn;
use crate::model::{auth::{Cip, UA, InternalUser, Session}, AuthProvider};
use crate::sys::SysInfo;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use rocket_oauth2::{OAuth2, TokenResponse};
use crate::model::model::COOKIE_DURATION;

#[derive(Serialize, Debug)]
pub struct Reddit {
    foreign_id: String,
    foreign_name: Option<String>,
}

impl AuthProvider for Reddit {
    fn platform(&self) -> String {
        String::from("reddit")
    }
    fn foreign_id(&self) -> String {
        self.foreign_id.clone()
    }
    fn foreign_name(&self) -> Option<String> {
        self.foreign_name.clone()
    }
}

#[get("/reddit")]
pub(crate) fn login(oauth2: OAuth2<Reddit>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect_extras(cookies, &["identity"], &[("duration", "permanent")])
        .unwrap()
}

#[get("/reddit")]
pub(crate) async fn callback(
    token: TokenResponse<Reddit>,
    cookies: &CookieJar<'_>,
    cip: Cip,
    ua: UA,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<Session>, crate::Error> {
    let user_info: serde_json::Value = reqwest::Client::builder()
        .build()
        .map_err(|_| crate::Error::InternalServerError{})?
        .get("https://oauth.reddit.com/api/v1/me")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .header(
            USER_AGENT,
            config
                .reddit_config
                .as_ref()
                .unwrap()
                .user_agent()
                .unwrap_or("RustRisk Local"),
        )
        .send()
        .await
        .map_err(|e| {
            dbg!(e);
            crate::Error::InternalServerError{}
        })?
        .json()
        .await
        .map_err(|e| {
            dbg!(e);
           crate::Error::InternalServerError{}
        })?;

    // This is a rather gross way of extracting the user's name
    let uname: String = String::from(
        user_info
            .get("name")
            .ok_or(crate::Error::InternalServerError{})?
            .as_str()
            .ok_or(crate::Error::InternalServerError{})?,
    );
     
    let login_data = Reddit {
        foreign_id: uname.clone(),
        foreign_name: Some(uname.clone())
    };

    let session = conn.run(|c| InternalUser::login_user(login_data, ua, cip, c)).await.map_err(|e| {dbg!(&e); e} )?;

    cookies.add_private(
        Cookie::build("jwt", session.put(config.settings.cookie_key.as_bytes())?)
            .same_site(SameSite::Lax)
            .domain(config.settings.base_url.clone())
            .path("/")
            .max_age(Duration::seconds(COOKIE_DURATION))
            .finish(),
    );
    
    std::result::Result::Ok(Json(session))
}
