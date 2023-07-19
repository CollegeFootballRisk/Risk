use okapi::openapi3::Responses;
use rocket::http::Status;
use rocket::response::{self, Responder};
use rocket::Request;
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner, OpenApiError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    //#[response(status = 400, content_type = "json")]
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
    Diesel {
        #[from]
        source: diesel::result::Error,
    },
    #[error("Unwrap Error")]
    Yeet {},
    #[error("NotFound Error")]
    NotFound {},

    #[error("Unauthorized Error")]
    Unauthorized {},

    #[error("Bad Request")]
    BadRequest {},

    #[error("Internal Server Error")]
    InternalServerError {},

    #[error("Utf8Error")]
    FromUtf8Error {
        #[from]
        source: std::string::FromUtf8Error,
    },

    #[error("Teapot")]
    Teapot,
}

pub type Result<T> = std::result::Result<T, crate::Error>;

pub trait MapRre<T> {
    fn map_rre(self) -> Result<T>;
}

impl<T, E> MapRre<T> for std::result::Result<T, E>
where
    Error: From<E>,
{
    fn map_rre(self) -> Result<T> {
        self.map_err(crate::Error::from)
    }
}
/*

impl<T> MapRre<T> for crate::error::Result<T> {
    fn map_rre(self) -> Result<T> {
        self.map_err(crate::Error::from)
    }
}*/

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'o> {
        // log `self` to your favored error tracker, e.g.
        // sentry::capture_error(&self);

        match self {
            Error::Unauthorized {} => Status::Unauthorized.respond_to(req),
            Error::NotFound {} => Status::NotFound.respond_to(req),
            Error::BadRequest {} => Status::BadRequest.respond_to(req),
            Error::InternalServerError {} => Status::InternalServerError.respond_to(req),
            Error::FromUtf8Error { .. } => Status::BadRequest.respond_to(req),
            Error::Teapot => Status::ImATeapot.respond_to(req),
            _ => Status::InternalServerError.respond_to(req),
        }
    }
}

impl Error {
    pub fn unauthorized<T>() -> std::result::Result<T, Error> {
        std::result::Result::Err(Error::Unauthorized {})
    }

    pub fn not_found<T>() -> std::result::Result<T, Error> {
        std::result::Result::Err(Error::NotFound {})
    }
}

impl OpenApiResponderInner for Error {
    fn responses(
        _generator: &mut OpenApiGenerator,
    ) -> std::result::Result<Responses, OpenApiError> {
        Ok(Default::default())
    }
}
