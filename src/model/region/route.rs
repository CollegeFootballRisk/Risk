/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::db::DbConn;
use crate::error::Error;
use crate::model::Region;
use rocket::serde::json::Json;

/// # Territory Ownership
/// Gives territory ownership information
#[openapi(tag = "Regions", ignore = "conn")]
#[get("/regions")]
pub(crate) async fn regions(conn: DbConn) -> Result<Json<Vec<Region>>, Error> {
    Ok(Json(conn.run(Region::load).await?))
}
