/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::schema::{bans, users};
use diesel::prelude::*;

use schemars::JsonSchema;

pub trait UserId {
    fn id(&self) -> i32;
}

pub struct UserIdFast {
    pub id: i32,
}
impl UserId for UserIdFast {
    fn id(&self) -> i32 {
        self.id
    }
}

#[derive(Insertable, Queryable, Serialize, Deserialize, JsonSchema, AsChangeset)]
#[diesel(table_name = users)]
pub struct UpsertableUser {
    pub(crate) uname: String,
    pub(crate) platform: String,
}

impl UpsertableUser {
    pub fn upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(users::table)
            .values((
                users::uname.eq(&self.uname),
                users::platform.eq(&self.platform),
            ))
            .on_conflict((users::uname, users::platform))
            .do_update()
            .set(users::uname.eq(&self.uname))
            .execute(conn)
    }
    pub fn flag(uname: String, conn: &mut PgConnection) -> QueryResult<usize> {
        let stop_ban = bans::table
            .filter(bans::class.eq(2))
            .filter(bans::uname.eq(&uname.clone()))
            .count()
            .get_result::<i64>(conn)?;
        if stop_ban > 0 {
            return QueryResult::Ok(0);
        }
        diesel::update(users::table)
            .filter(users::uname.eq(&uname))
            .set(users::is_alt.eq(true))
            .execute(conn)
    }
}

#[derive(Queryable, Identifiable)]
#[diesel(table_name = users)]
pub struct UpdateUser {
    pub(crate) id: i32,
    pub(crate) overall: i32,
    pub(crate) turns: i32,
    pub(crate) game_turns: i32,
    pub(crate) mvps: i32,
    pub(crate) streak: i32,
    // pub(crate) awards: i32,
}

impl UpdateUser {
    pub fn do_update(user: UpdateUser, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::update(users::table)
            .filter(users::id.eq(user.id))
            .set((
                users::overall.eq(user.overall),
                users::turns.eq(user.turns),
                users::game_turns.eq(user.game_turns),
                users::mvps.eq(user.mvps),
                users::streak.eq(user.streak),
                // users::awards.eq(user.awards),
            ))
            .execute(conn)
    }
}
