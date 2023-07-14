/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rocket::time::Duration;
use crate::db::DbConn;
use crate::model::model::COOKIE_DURATION;
use crate::model::{auth::{Cip, UA, InternalUser, Session}, AuthProvider};
use crate::sys::SysInfo;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use rocket_oauth2::{OAuth2, TokenResponse};

#[derive(Serialize, Debug)]
pub struct Discord {
    foreign_id: String,
    foreign_name: Option<String>,
}

impl AuthProvider for Discord {
    fn platform(&self) -> String {
        String::from("discord")
    }
    fn foreign_id(&self) -> String {
        self.foreign_id.clone()
    }
    fn foreign_name(&self) -> Option<String> {
        self.foreign_name.clone()
    }
}

#[get("/discord")]
pub(crate) fn login(oauth2: OAuth2<Discord>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2
        .get_redirect_extras(
            cookies,
            &["email", "identify"],
            &[("duration", "permanent")],
        )
        .unwrap()
}

#[get("/discord")]
pub(crate) async fn callback(
    token: TokenResponse<Discord>,
    cookies: &CookieJar<'_>,
    cip: Cip,
    ua: UA,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<Session>, crate::Error> {
    let user_info: serde_json::Value = reqwest::Client::builder()
        .build()
        .map_err(|_| crate::Error::InternalServerError{})?
        .get("https://discord.com/api/v10/users/@me")
        .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
        .header(
            USER_AGENT,
            config
                .discord_config
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
        
    let login_data = Discord {
        foreign_id: user_info
            .get("id")
            .ok_or(crate::Error::InternalServerError{})?
            .as_str()
            .ok_or(crate::Error::InternalServerError{})?
            .to_owned(),
        foreign_name: Some(
            user_info
                .get("username")
                .ok_or(crate::Error::InternalServerError{})?
                .as_str()
                .ok_or(crate::Error::InternalServerError{})?
                .to_owned(),
        ),
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
