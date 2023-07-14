/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::user;
use chrono::NaiveDateTime;
use diesel::{prelude::*};
use uuid::{Uuid};

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// An `AuthenticationMethod` denotes a method for logging in for a User, e.g.
/// Reddit, Discord, etc.
pub(crate) struct User {
    // The `Uuid` for the `User`
    pub(crate) id: Uuid,
    /// The id of the `Team` to which the `User` is primarily aligned (may be dead)
    pub(crate) main_team: Option<i32>,
    /// The id of the `Team` to which the `User` is currently aligned (will not be dead)
    pub(crate) playing_for: Option<i32>,
    /// The overall rating (1-5) of `User`
    pub(crate) overall: i32,
    /// The number of `Move`s made by `User`
    pub(crate) turns: i32,
    /// The number of `Move`s made by `User` this game
    pub(crate) game_turns: i32,
    /// The number of `Move`s made by `User` for which they were MVP
    pub(crate) mvps: i32,
    /// The number of consecutive `Move`s made by `User`
    pub(crate) streak: i32,
    /// Whether the user is an alt
    pub(crate) is_alt: bool,
    /// Whether the user must captcha or not, not displayed to end user
    #[serde(skip_serializing)]
    pub(crate) must_captcha: bool,
    /// The timestamp of the creation of the `User`
    pub(crate) created: NaiveDateTime,
    /// The timestamp of the last update to `User`
    pub(crate) updated: NaiveDateTime,
    /// The `Uuid` of the `User` who most created the `User`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `User` who most recently updated `User`
    pub(crate) updatedby: Uuid,
}
