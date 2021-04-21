use crate::schema::{continuation_polls, continuation_responses, new_turns, territories};
use diesel::prelude::*;
use jsonwebtoken::errors::Error;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: i32,
    pub user: String,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientInfo {
    pub claims: Claims,
    pub ip: String,
}

#[derive(Serialize, Deserialize)]
pub struct Move {
    pub attack: Option<i32>,
    pub defend: Option<i32>,
    /* = "risk_security")]  pub information: DebuggingInformation, */
}

#[derive(Serialize, Deserialize)]
pub struct MoveInfo {
    pub territory: Option<String>,
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct Poll {
    pub id: i32,
    pub season: i32,
    pub day: i32,
    pub question: String,
    pub incrment: i32,
}

#[derive(Serialize, Deserialize, Queryable)]
pub struct PollResponse {
    pub id: i32,
    pub poll: i32,
    pub user_id: i32,
    pub response: bool,
}

impl Claims {
    pub fn put(key: &[u8], user_claims: Claims) -> Result<String, Error> {
        encode(&Header::default(), &user_claims, &EncodingKey::from_secret(key))
    }

    pub fn interpret(key: &[u8], token: String) -> Result<(Claims, Header), String> {
        let validation = Validation {
            ..Validation::default()
        };
        match decode::<Claims>(&token, &DecodingKey::from_secret(key), &validation) {
            Ok(c) => Ok((c.claims, c.header)),
            Err(err) => Err(err.to_string()),
        }
    }
}

impl MoveInfo {
    pub fn get(season: i32, day: i32, user_id: i32, conn: &PgConnection) -> MoveInfo {
        let r = new_turns::table
            .filter(new_turns::user_id.eq(user_id))
            .filter(new_turns::season.eq(season))
            .filter(new_turns::day.eq(day))
            .inner_join(territories::table.on(new_turns::territory.eq(territories::id)))
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
    pub fn get(
        season: i32,
        day: i32,
        conn: &PgConnection,
    ) -> Result<Vec<Poll>, diesel::result::Error> {
        continuation_polls::table
            .filter(continuation_polls::season.eq(season))
            .filter(continuation_polls::day.ge(day))
            .load::<Poll>(conn)
    }
}

impl PollResponse {
    pub fn get(
        poll_id: i32,
        user_id: i32,
        conn: &PgConnection,
    ) -> Result<Vec<PollResponse>, diesel::result::Error> {
        continuation_responses::table
            .filter(continuation_responses::poll_id.eq(poll_id))
            .filter(continuation_responses::user_id.eq(user_id))
            .load::<PollResponse>(conn)
    }

    pub fn upsert(response: PollResponse, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(continuation_responses::table)
            .values((
                continuation_responses::poll_id.eq(response.poll),
                continuation_responses::user_id.eq(response.user_id),
                continuation_responses::response.eq(response.response),
            ))
            .on_conflict((continuation_responses::poll_id, continuation_responses::user_id))
            .do_update()
            .set(continuation_responses::response.eq(response.response))
            .execute(conn)
    }
}
