/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::db::DbConn;
use crate::model::Event;
use rocket::serde::json::Json;
/// # Lists users who have selected a new team in the Transfer Portal.
/// Returns a listing of users who have entered the transfer portal and selected a new team after their team was eliminated.
/// Also lists new users who have just joined a team.
/// Paginated in batches of `count` starting from the `offset`-ed most recent event. `count` is limited to <=100.
#[openapi(tag = "Events", ignore = "conn")]
#[get("/events/transfers?<count>&<offset>")]
pub(crate) async fn transfers(
    mut count: Option<i64>,
    offset: Option<i64>,
    conn: DbConn,
) -> Result<Json<Vec<Event>>, crate::Error> {
    if count > Some(100) {
        count = Some(100)
    };
    std::result::Result::Ok(Json(
        conn.run(move |cn| Event::load_change(count, offset, &cn))
            .await
            .map_err(|_| crate::Error::InternalServerError {})?,
    ))
}

/// # List of notifications
/// Returns list of wide-spread notifications intended for all users.
/// Paginated in batches of `count` starting from the `offset`-ed most recent event. `count` is limited to <=100.
#[openapi(tag = "Events", ignore = "conn")]
#[get("/events/notifications?<count>&<offset>")]
pub(crate) async fn notifications(
    mut count: Option<i64>,
    offset: Option<i64>,
    conn: DbConn,
) -> Result<Json<Vec<Event>>, crate::Error> {
    if count > Some(100) {
        count = Some(100)
    };
    std::result::Result::Ok(Json(
        conn.run(move |cn| Event::load_notifications(count, offset, &cn))
            .await
            .map_err(|_| crate::Error::InternalServerError {})?,
    ))
}
