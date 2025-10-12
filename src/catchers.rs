/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rocket::serde::json::Json;

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
