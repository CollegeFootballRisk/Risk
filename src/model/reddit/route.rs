use crate::model::{Claims, RedditUserInfo, UpsertableUser};
use hyper::{
    header::{Authorization, Bearer, UserAgent},
    net::HttpsConnector,
    Client,
};

use crate::{db::DbConn, model::User};
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_oauth2::{OAuth2, TokenResponse};
use serde_json::{self};
use std::io::Read;
/*use chrono::prelude::*;
use chrono::Duration;*/
use chrono::Utc;
use diesel_citext::types::CiString;
use time::Duration;

#[get("/reddit")]
pub fn reddit_login(oauth2: OAuth2<RedditUserInfo>, cookies: &CookieJar<'_>) -> Redirect {
    oauth2.get_redirect_extras(&mut cookies, &["identity"], &[("duration", "permanent")]).unwrap()
}

#[get("/logout")]
pub fn reddit_logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    /*let token: String = cookies
        .get_private("jwt")
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or_else(|| "".to_string());
    let https = HttpsConnector::new(hyper_sync_rustls::TlsClient::new());
    let client = Client::with_connector(https);
    let _response = client
        .get("https://www.reddit.com/api/v1/revoke_token")
        .header(Authorization(Bearer {
            token,
        }))
        .header(UserAgent("AggieRiskLocal - Dev Edition".into()))
        .send()
        .context("failed to send request to API");*/
    cookies.remove_private(Cookie::named("jwt"));
    cookies.remove_private(Cookie::named("username"));
    Flash::success(Redirect::to("/"), "Successfully logged out.")
    //TODO: Implement a deletion call to reddit.
}

#[get("/reddit")]
pub fn reddit_callback(
    token: TokenResponse<RedditUserInfo>,
    cookies: &CookieJar<'_>,
    conn: DbConn,
    key: State<String>,
) -> Result<Redirect, Status> {
    match getRedditUserInfo(&token) {
        Ok(user_info) => {
            let new_user = UpsertableUser {
                uname: CiString::from(user_info.name.clone()),
                platform: CiString::from("reddit"),
            };
            match UpsertableUser::upsert(new_user, &conn) {
                Ok(_n) => {
                    match User::load(user_info.name.clone(), "reddit".to_string(), &conn) {
                        Ok(user) => {
                            dotenv::from_filename("../.env").ok();
                            let datetime = Utc::now();
                            let timestamp: usize = 604800 + datetime.timestamp() as usize;
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
                                    .domain(dotenv::var("uri").unwrap_or_default())
                                    .path("/")
                                    .max_age(Duration::hours(168))
                                    .finish(),
                            );
                            match Claims::put(&key.as_bytes(), new_claims) {
                                Ok(s) => {
                                    cookies.add_private(
                                        Cookie::build("jwt", s)
                                            .same_site(SameSite::Lax)
                                            .domain(dotenv::var("uri").unwrap_or_default())
                                            .path("/")
                                            .max_age(Duration::hours(168))
                                            .finish(),
                                    );
                                    std::result::Result::Ok(Redirect::to("/"))
                                }
                                _ => std::result::Result::Err(Status::NotAcceptable),
                            }
                        }
                        Err(_e) => std::result::Result::Err(Status::BadRequest),
                    }
                }
                Err(_ex) => std::result::Result::Err(Status::BadRequest),
            }
        }
        _ => std::result::Result::Err(Status::Gone),
    }
}

fn getRedditUserInfo(token: &TokenResponse<RedditUserInfo>) -> Result<RedditUserInfo, String> {
    let https = HttpsConnector::new(hyper_rustls::TlsClient::new());
    let client = Client::with_connector(https);
    match client
        .get("https://oauth.reddit.com/api/v1/me")
        .header(Authorization(Bearer {
            token: token.access_token().to_string(),
        }))
        .header(UserAgent("AggieRiskLocal - Dev Edition".into()))
        .send()
    {
        Ok(response) => {
            match serde_json::from_reader(response.take(2 * 1024 * 1024)) {
                Ok(send) => Ok(send),
                Err(_e) => Err("Error in getting user data #2".to_string()),
            }
        }
        Err(_e) => Err("Error in getting user data #1".to_string()),
    }
}
