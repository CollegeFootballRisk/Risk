use okapi::openapi3::Responses;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::Request;
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner, OpenApiError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP Error {source:?}")]
    Reqwest {
        #[from]
        source: reqwest::Error,
    },
    #[error("SerdeJson Error {source:?}")]
    SerdeJson {
        #[from]
        source: serde_json::Error,
    },
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
    #[error("Unwrap Error")]
    Yeet {},
    #[error("NotFound Error")]
    NotFound {},

    #[error("Unauthorized Error")]
    Unauthorized {},
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        // log `self` to your favored error tracker, e.g.
        // sentry::capture_error(&self);

        match self {
            // in our simplistic example, we're happy to respond with the default 500 responder in all cases
            Error::Unauthorized {} => Status::Unauthorized.respond_to(req),
            Error::NotFound {} => Status::NotFound.respond_to(req),
            _ => Status::InternalServerError.respond_to(req),
        }
    }
}

impl Error {
    pub fn unauthorized<T>() -> Result<T, Error> {
        std::result::Result::Err(Error::Unauthorized {})
    }

    pub fn not_found<T>() -> Result<T, Error> {
        std::result::Result::Err(Error::NotFound {})
    }
}

impl OpenApiResponderInner for Error {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        Ok(Responses {
            ..Default::default()
        })
    }
}
