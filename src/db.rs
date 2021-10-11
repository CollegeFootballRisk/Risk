/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rocket_sync_db_pools::{database, diesel};
#[database("postgres_global")]
pub(crate) struct DbConn(diesel::PgConnection);
