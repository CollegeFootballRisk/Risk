/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use okapi::openapi3::Responses;
use rocket::serde::json::Json;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::response::OpenApiResponderInner;
use rocket_okapi::util::set_status_code;
use rocket_okapi::Result;

#[derive(Serialize, Deserialize)]
pub(crate) struct Httperror {
    pub(crate) status: i32,
}

#[catch(404)]
pub(crate) fn not_found() -> Json<Httperror> {
    Json(Httperror { status: 404 })
}

#[catch(401)]
pub(crate) fn not_authorized() -> Json<Httperror> {
    Json(Httperror { status: 401 })
}

#[catch(500)]
pub(crate) fn internal_error() -> Json<Httperror> {
    Json(Httperror { status: 500 })
}

#[derive(Debug)]
pub(crate) struct Status(pub(crate) rocket::http::Status);

impl OpenApiResponderInner for Status {
    fn responses(_gen: &mut OpenApiGenerator) -> Result<Responses> {
        let mut responses = Responses::default();
        set_status_code(&mut responses, 500)?;
        Ok(responses)
    }
}

use rocket::http::StatusClass;
use rocket::response;

impl<'r> response::Responder<'r, 'static> for Status {
    fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'static> {
        match self.0.class() {
            StatusClass::ClientError | StatusClass::ServerError => Err(self.0),
            StatusClass::Success if self.0.code < 206 => {
                response::Response::build().status(self.0).ok()
            }
            StatusClass::Informational if self.0.code == 100 => {
                response::Response::build().status(self.0).ok()
            }
            _ => {
                error_!("Invalid status used as responder: {}.", self.0);
                Err(rocket::http::Status::InternalServerError)
            }
        }
    }
}
