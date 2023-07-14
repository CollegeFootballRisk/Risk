/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::schema::{ban, user};
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
#[diesel(table_name = user)]
pub struct UpsertableUser {
    pub(crate) uname: String,
    pub(crate) platform: String,
}

impl UpsertableUser {
    pub fn upsert(&self, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(user::table)
            .values((
                user::uname.eq(&self.uname),
                user::platform.eq(&self.platform),
            ))
            .on_conflict((user::uname, user::platform))
            .do_update()
            .set(user::uname.eq(&self.uname))
            .execute(conn)
    }
    pub fn flag(uname: String, conn: &mut PgConnection) -> QueryResult<usize> {
        let stop_ban = ban::table
            .filter(ban::class.eq(2))
            .filter(ban::uname.eq(&uname.clone()))
            .count()
            .get_result::<i64>(conn)?;
        if stop_ban > 0 {
            return QueryResult::Ok(0);
        }
        diesel::update(user::table)
            .filter(user::uname.eq(&uname))
            .set(user::is_alt.eq(true))
            .execute(conn)
    }
}

#[derive(Queryable, Identifiable)]
#[diesel(table_name = user)]
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
        diesel::update(user::table)
            .filter(user::id.eq(user.id))
            .set((
                user::overall.eq(user.overall),
                user::turns.eq(user.turns),
                user::game_turns.eq(user.game_turns),
                user::mvps.eq(user.mvps),
                user::streak.eq(user.streak),
                // user::awards.eq(user.awards),
            ))
            .execute(conn)
    }
}
