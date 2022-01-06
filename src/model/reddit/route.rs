/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::{Claims, RedditUserInfo, UpsertableUser};
use crate::{db::DbConn, model::User, sys::SysInfo};
use chrono::Utc;
use diesel_citext::types::CiString;
use reqwest::header::{AUTHORIZATION, USER_AGENT};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_oauth2::{OAuth2, TokenResponse};
use time::Duration;

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
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Redirect, Status> {
    let userinfo: Result<RedditUserInfo, _> = match reqwest::Client::builder().build() {
        Ok(rclient) => {
            match rclient
                .get("https://oauth.reddit.com/api/v1/me")
                .header(AUTHORIZATION, format!("Bearer {}", token.access_token()))
                .header(USER_AGENT, "AggieRiskLocal - Dev Edition")
                .send()
                .await
            {
                Ok(text) => text.json().await,
                Err(_) => {
                    return std::result::Result::Err(Status::BadRequest);
                }
            }
        }
        Err(_) => {
            return std::result::Result::Err(Status::BadRequest);
        }
    };
    dbg!(&userinfo);
    match userinfo {
        Ok(user_info) => {
            let new_user = UpsertableUser {
                uname: CiString::from(user_info.name.clone()),
                platform: CiString::from("reddit"),
            };
            match conn.run(move |c| UpsertableUser::upsert(new_user, c)).await {
                Ok(_n) => {
                    let name = user_info.name.clone();
                    match conn
                        .run(move |c| User::load(name, "reddit".to_string(), c))
                        .await
                    {
                        Ok(user) => {
                            let datetime = Utc::now();
                            let timestamp: usize = 604_800 + datetime.timestamp() as usize;
                            //dbg!(&token);
                            let new_claims = Claims {
                                id: user.id,
                                user: user.uname.to_string(),
                                token: Some(token.refresh_token().unwrap().to_string()),
                                refresh_token: Some(token.access_token().to_string()),
                                exp: timestamp,
                            };
                            cookies.add_private(
                                Cookie::build("username", user_info.name)
                                    .same_site(SameSite::Lax)
                                    .domain(config.settings.base_url.clone())
                                    .path("/")
                                    .max_age(Duration::hours(168))
                                    .finish(),
                            );
                            match Claims::put(config.settings.cookie_key.as_bytes(), new_claims) {
                                Ok(s) => {
                                    cookies.add_private(
                                        Cookie::build("jwt", s)
                                            .same_site(SameSite::Lax)
                                            .domain(config.settings.base_url.clone())
                                            .path("/")
                                            .max_age(Duration::hours(168))
                                            .finish(),
                                    );
                                    std::result::Result::Ok(Redirect::to("/"))
                                }
                                _ => std::result::Result::Err(Status::NotAcceptable),
                            }
                        }
                        Err(_e) => {
                            dbg!(_e);
                            std::result::Result::Err(Status::BadRequest)
                        }
                    }
                }
                Err(_ex) => {
                    dbg!(_ex);
                    std::result::Result::Err(Status::BadRequest)
                }
            }
        }
        _ => std::result::Result::Err(Status::Gone),
    }
}
