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

impl Claims {
    pub fn put(key: &[u8], user_claims: Claims) -> Result<String, Error> {
        encode(
            &Header::default(),
            &user_claims,
            &EncodingKey::from_secret(key),
        )
    }

    pub fn interpret(key: &[u8], token: String) -> Result<(Claims, Header), String> {
        let validation = Validation {
            ..Validation::default()
        };
        match decode::<Claims>(&token, &DecodingKey::from_secret(key), &validation) {
            Ok(c) => Ok((c.claims, c.header)),
            Err(err) => Err(err.to_string()), /*match *err.kind() {
                                                  ErrorKind::InvalidToken => Err(String::from("Invalid Token")), // Example on how to handle a specific error
                                                  ErrorKind::InvalidIssuer => Err(String::from("Invalid Issuer")), // Example on how to handle a specific error
                                                  _ => Err(String::from("Error")),
                                              },*/
        }
    }
}
