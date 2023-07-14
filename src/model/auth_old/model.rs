/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::{
    log, territory, turn, move_,
};
use crate::sys::SysInfo;
use diesel::prelude::*;
use jsonwebtoken::errors::Error;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rocket::http::CookieJar;
use rocket::State;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Claims {
    pub(crate) id: i32,
    pub(crate) user: String,
    pub(crate) token: Option<String>,
    pub(crate) refresh_token: Option<String>,
    pub(crate) exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ClientInfo {
    pub(crate) claims: Claims,
    pub(crate) ip: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Move {
    pub(crate) attack: Option<i32>,
    pub(crate) defend: Option<i32>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MoveInfo {
    pub(crate) territory: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Log {
    pub(crate) route: String,
    pub(crate) query: String,
    pub(crate) payload: String,
}

#[derive(Debug, Deserialize)]
pub struct MoveSub {
    pub target: i32,
    pub aon: Option<bool>,
    pub token: Option<String>,
    pub token_v2: Option<String>,
}

impl Claims {
    pub(crate) fn put(key: &[u8], user_claims: Claims) -> Result<String, Error> {
        encode(
            &Header::default(),
            &user_claims,
            &EncodingKey::from_secret(key),
        )
    }

    pub(crate) fn interpret(key: &[u8], token: String) -> Result<(Claims, Header), String> {
        let validation = Validation::default();
        match decode::<Claims>(&token, &DecodingKey::from_secret(key), &validation) {
            Ok(c) => Ok((c.claims, c.header)),
            Err(err) => Err(err.to_string()),
        }
    }

    pub(crate) fn from_private_cookie(
        cookies: &CookieJar<'_>,
        config: &State<SysInfo>,
    ) -> Result<(Claims, Header), crate::Error> {
        let cookie = cookies
            .get_private("jwt")
            .ok_or(crate::Error::Unauthorized {})?;
        Claims::interpret(
            config.settings.cookie_key.as_bytes(),
            cookie.value().to_string(),
        )
        .map_err(|_| crate::Error::BadRequest {})
    }
}

impl Log {
    pub(crate) fn begin(r: String, q: String) -> Log {
        Log {
            route: r,
            query: q,
            payload: String::new(),
        }
    }

    pub(crate) fn insert(&self, conn: &mut PgConnection) -> Result<(), crate::Error> {
        let route = self.route.clone();
        let query = self.query.clone();
        let payload = self.payload.clone();
        let err = diesel::insert_into(log::table)
            .values((
                log::route.eq(&route),
                log::query.eq(&query),
                log::payload.eq(&payload),
            ))
            .execute(conn);
        if let Ok(e) = err {
            if e > 0 {
                Ok(())
            } else {
                dbg!(&self);
                Err(crate::Error::InternalServerError {})
            }
        } else {
            dbg!(&self);
            Err(crate::Error::InternalServerError {})
        }
    }
}

impl MoveInfo {
    pub(crate) fn get(season: i32, day: i32, user_id: i32, conn: &mut PgConnection) -> MoveInfo {
        let r = move_::table
            .filter(move_::user_id.eq(user_id))
            .filter(turn::season.eq(season))
            .filter(turn::day.eq(day))
            .inner_join(turn::table.on(move_::turn_id.eq(turn::id)))
            .inner_join(territory::table.on(move_::territory.eq(territory::id)))
            .select(territory::name)
            .first(conn);
        MoveInfo {
            territory: match r {
                Ok(n) => Some(n),
                Err(_E) => None,
            },
        }
    }
}
