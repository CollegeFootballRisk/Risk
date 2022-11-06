/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// EventMapping, Event defined not here but in schema.rs
use crate::schema::{event, event::dsl::*, Flavor};
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

#[derive(Insertable, Serialize, Queryable, Debug, Deserialize, JsonSchema)]
#[table_name = "event"]
pub struct Event {
    pub id: Uuid,
    pub flavor: Flavor,
    pub time: chrono::NaiveDateTime,
    pub payload: Value,
}

impl Event {
    pub(crate) fn load_notifications(
        limit: Option<i64>,
        offset: Option<i64>,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, diesel::result::Error> {
        event::table
            .filter(event::flavor.eq(Flavor::Notification))
            .order_by(event::time.desc())
            .limit(limit.unwrap_or(10))
            .offset(offset.unwrap_or(0))
            .load::<Event>(conn)
    }

    pub(crate) fn load_change(
        limit: Option<i64>,
        offset: Option<i64>,
        conn: &PgConnection,
    ) -> Result<Vec<Event>, diesel::result::Error> {
        event::table
            .filter(event::flavor.eq(Flavor::ChangeTeam))
            .order_by(event::time.desc())
            .limit(limit.unwrap_or(10))
            .offset(offset.unwrap_or(0))
            .load::<Event>(conn)
    }

    // Insert an event
    pub(crate) fn insert(&self, conn: &PgConnection) -> Result<usize, diesel::result::Error> {
        diesel::insert_into(event).values(self).execute(conn)
    }
}
