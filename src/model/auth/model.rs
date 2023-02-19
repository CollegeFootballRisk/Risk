/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::{
    continuation_polls, continuation_responses, logs, territories, turninfo, turns,
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
    pub captcha_title: Option<String>,
    pub captcha_content: Option<String>,
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Queryable)]
pub(crate) struct Poll {
    pub(crate) id: i32,
    pub(crate) season: i32,
    pub(crate) day: i32,
    pub(crate) question: String,
    pub(crate) increment: i32,
}

#[derive(Serialize, Deserialize, Queryable)]
pub(crate) struct PollResponse {
    pub(crate) id: i32,
    pub(crate) poll: i32,
    pub(crate) user_id: i32,
    pub(crate) response: bool,
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

    pub(crate) fn insert(&self, conn: &PgConnection) -> Result<(), crate::Error> {
        let route = self.route.clone();
        let query = self.query.clone();
        let payload = self.payload.clone();
        let err = diesel::insert_into(logs::table)
            .values((
                logs::route.eq(&route),
                logs::query.eq(&query),
                logs::payload.eq(&payload),
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
    pub(crate) fn get(season: i32, day: i32, user_id: i32, conn: &PgConnection) -> MoveInfo {
        let r = turns::table
            .filter(turns::user_id.eq(user_id))
            .filter(turninfo::season.eq(season))
            .filter(turninfo::day.eq(day))
            .inner_join(turninfo::table.on(turns::turn_id.eq(turninfo::id)))
            .inner_join(territories::table.on(turns::territory.eq(territories::id)))
            .select(territories::name)
            .first(conn);
        MoveInfo {
            territory: match r {
                Ok(n) => Some(n),
                Err(_E) => None,
            },
        }
    }
}

impl Poll {
    pub(crate) fn get(
        season: i32,
        day: i32,
        conn: &PgConnection,
    ) -> Result<Vec<Poll>, diesel::result::Error> {
        continuation_polls::table
            .inner_join(turninfo::table.on(turninfo::id.eq(continuation_polls::turn_id)))
            .filter(turninfo::season.eq(season))
            .filter(turninfo::day.ge(day))
            .select((
                continuation_polls::id,
                turninfo::season,
                turninfo::day,
                continuation_polls::question,
                continuation_polls::incrment,
            ))
            .load::<Poll>(conn)
    }
}

impl PollResponse {
    pub(crate) fn get(
        poll_id: i32,
        user_id: i32,
        conn: &PgConnection,
    ) -> Result<Vec<PollResponse>, diesel::result::Error> {
        continuation_responses::table
            .filter(continuation_responses::poll_id.eq(poll_id))
            .filter(continuation_responses::user_id.eq(user_id))
            .load::<PollResponse>(conn)
    }

    pub(crate) fn upsert(response: PollResponse, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(continuation_responses::table)
            .values((
                continuation_responses::poll_id.eq(response.poll),
                continuation_responses::user_id.eq(response.user_id),
                continuation_responses::response.eq(response.response),
            ))
            .on_conflict((
                continuation_responses::poll_id,
                continuation_responses::user_id,
            ))
            .do_update()
            .set(continuation_responses::response.eq(response.response))
            .execute(conn)
    }
}
