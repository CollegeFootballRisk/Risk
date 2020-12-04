use crate::schema::{new_turns, territories};
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
    #[cfg(feature = "risk_security")]
    pub information: DebuggingInformation,
}

#[derive(Serialize, Deserialize)]
pub struct MoveInfo {
    pub territory: Option<String>,
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
