use crate::schema::users;
use diesel::prelude::*;
use diesel_citext::types::CiString;
use schemars::JsonSchema;

#[derive(Insertable, Queryable, Serialize, Deserialize, JsonSchema, AsChangeset)]
#[table_name = "users"]
pub struct UpsertableUser {
    pub uname: CiString,
    pub platform: CiString,
}

impl UpsertableUser {
    pub fn upsert(user: UpsertableUser, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(users::table)
            .values((users::uname.eq(&user.uname), users::platform.eq(user.platform)))
            .on_conflict((users::uname, users::platform))
            .do_update()
            .set(users::uname.eq(&user.uname))
            .execute(conn)
    }
}

#[derive(Queryable, Identifiable)]
#[table_name = "users"]
pub struct UpdateUser {
    pub id: i32,
    pub overall: i32,
    pub turns: i32,
    pub game_turns: i32,
    pub mvps: i32,
    pub streak: i32,
    pub awards: i32,
}

impl UpdateUser {
    pub fn do_update(user: UpdateUser, conn: &PgConnection) -> QueryResult<usize> {
        diesel::update(users::table)
            .filter(users::id.eq(user.id))
            .set((
                users::overall.eq(user.overall),
                users::turns.eq(user.turns),
                users::game_turns.eq(user.game_turns),
                users::mvps.eq(user.mvps),
                users::streak.eq(user.streak),
                users::awards.eq(user.awards),
            ))
            .execute(conn)
    }
}
