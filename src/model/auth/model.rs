/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::error::{MapRre, Result};
use crate::model::player::Player;
use crate::schema::{authentication_method, permission, player, player_role, role};
use crate::sys::SysInfo;
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::{insert_into, prelude::*};
use ipnetwork::IpNetwork;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use rocket::http::CookieJar;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::State;
use uuid::{uuid, Uuid};
// Use System ID
const SYSTEM_UUID: Uuid = uuid!("a147b32b-6779-462c-b20b-5f5bef4702fa");
// One month
pub const COOKIE_DURATION: i64 = 2592000;

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = authentication_method)]
#[diesel(belongs_to(Player))]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// An `AuthenticationMethod` denotes a method for logging in for a Player, e.g.
/// Reddit, Discord, etc.
pub(crate) struct AuthenticationMethod {
    pub(crate) id: Uuid,
    pub(crate) player_id: Uuid,
    // Limited to 10 char
    pub(crate) platform: String,
    // Limited to 256 char
    pub(crate) foreign_id: String,
    // Limited to 128 char
    pub(crate) foreign_name: Option<String>,
    pub(crate) created: NaiveDateTime,
    pub(crate) updated: NaiveDateTime,
    pub(crate) createdby: Uuid,
    pub(crate) updatedby: Uuid,
    pub(crate) published: bool,
}

impl AuthenticationMethod {
    pub(crate) fn new(
        _player_id: Uuid,
        _platform: String,
        _foreign_id: String,
        _foreign_name: Option<String>,
        _createdby: Uuid,
        conn: &mut PgConnection,
    ) -> Result<Self> {
        let _createdby: Uuid = _createdby;
        use crate::schema::authentication_method::dsl::*;
        Ok(insert_into(authentication_method)
            .values((
                player_id.eq(_player_id),
                platform.eq(_platform),
                foreign_id.eq(_foreign_id),
                foreign_name.eq(_foreign_name),
                createdby.eq(_createdby),
                updatedby.eq(_createdby),
            ))
            .get_result(conn)?)
    }

    /// Retrieve an `AuthenticationMethod` using the unique pair (`platform`, `foreign_id`)
    pub(crate) fn get(
        _platform: String,
        _foreign_id: String,
        conn: &mut PgConnection,
    ) -> Result<Self> {
        use crate::schema::authentication_method::dsl::*;
        Ok(authentication_method
            .filter(platform.eq(_platform))
            .filter(foreign_id.eq(_foreign_id))
            .select(AuthenticationMethod::as_select())
            .get_result(conn)?)
    }

    pub(crate) fn get_by_id(_id: Uuid, conn: &mut PgConnection) -> Result<Self> {
        use crate::schema::authentication_method::dsl::*;
        Ok(authentication_method
            .filter(id.eq(_id))
            .select(AuthenticationMethod::as_select())
            .get_result(conn)?)
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn player_id(&self) -> Uuid {
        self.player_id
    }
}

#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = session)]
#[diesel(belongs_to(Player))]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// A `Session` denotes an active instance of a `Player` (i.e. a logged-in player)
pub(crate) struct Session {
    #[diesel(deserialize_as = Uuid)]
    pub(crate) id: Option<Uuid>,
    pub(crate) player_id: Uuid,
    pub(crate) authentication_method_id: Uuid,
    pub(crate) is_active: bool,
    pub(crate) player_agent: String,
    pub(crate) ip_address: IpNetwork,
    #[diesel(deserialize_as = NaiveDateTime)]
    pub(crate) created: Option<NaiveDateTime>,
    pub(crate) expires: Option<NaiveDateTime>,
}

impl Session {
    /// Create a new `Session` by insertion
    ///
    /// # Fields
    /// _player_id: the Uuid of the `Player`
    /// _authentication_method_id: the Uuid of the `AuthenticationMethod` a player has
    /// _expires: optional `NaiveDateTime` for the token to expire.
    pub(crate) fn new(
        _player_id: Uuid,
        _authentication_method_id: Uuid,
        _expires: Option<NaiveDateTime>,
        _player_agent: String,
        _ip_address: Option<IpNetwork>,
        conn: &mut PgConnection,
    ) -> Result<Self> {
        use crate::schema::session::dsl::*;
        // Ok(..) coerces this into a Session
        Ok(insert_into(session)
            .values((
                player_id.eq(_player_id),
                authentication_method_id.eq(_authentication_method_id),
                expires.eq(_expires),
                player_agent.eq(_player_agent),
                ip_address.eq(_ip_address.unwrap_or("1.1.1.1".parse::<IpNetwork>().unwrap())),
            ))
            .get_result(conn)?)
    }

    pub(crate) fn put(&self, key: &[u8]) -> Result<String> {
        encode(&Header::default(), &self, &EncodingKey::from_secret(key))
            .map_err(|_| crate::Error::InternalServerError {})
    }

    pub(crate) fn interpret(key: &[u8], token: String) -> std::result::Result<(Session, Header), String> {
        let validation = Validation::default();
        match decode::<Session>(&token, &DecodingKey::from_secret(key), &validation) {
            Ok(c) => Ok((c.claims, c.header)),
            Err(err) => Err(err.to_string()),
        }
    }

    pub(crate) fn from_private_cookie(
        cookies: &CookieJar<'_>,
        config: &State<SysInfo>,
    ) -> Result<(Session, Header)> {
        let cookie = cookies
            .get_private("jwt")
            .ok_or(crate::Error::Unauthorized {})?;
        Session::interpret(
            config.settings.cookie_key.as_bytes(),
            cookie.value().to_string(),
        )
        .map_err(|_| crate::Error::BadRequest {})
    }

    pub(crate) fn get_permissions() -> Permission {
        todo!()
    }

    pub(crate) fn check_permissions(permissions: Vec<String>) -> Permission {
        todo!()
    }
}

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = permission)]
#[diesel(belongs_to(Role))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Permission {
    // The `id` for the `Permission`
    pub(crate) id: i32,
    // The name of the `Role`
    pub(crate) name: String,
    /// The timestamp of the creation of the `Permission`
    pub(crate) created: NaiveDateTime,
    /// The timestamp of the last update to `Permisison`
    pub(crate) updated: NaiveDateTime,
    /// The `Uuid` of the `Player` who created the `Permission`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `Player` who most recently updated `Permission`
    pub(crate) updatedby: Uuid,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[diesel(table_name = role)]
#[diesel(belongs_to(PlayerRole))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct Role {
    // The `id` for the `Role`
    pub(crate) id: i32,
    // The name of the `Role`
    pub(crate) name: String,
    /// The timestamp of the creation of the `Role`
    pub(crate) created: NaiveDateTime,
    /// The timestamp of the last update to `Role`
    pub(crate) updated: NaiveDateTime,
    /// The `Uuid` of the `Player` who created the `Role`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `Player` who most recently updated `Role`
    pub(crate) updatedby: Uuid,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = player_role)]
#[diesel(belongs_to(InternalPlayer))]
#[diesel(belongs_to(Role))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct PlayerRole {
    // The `Uuid` for the `PlayerRole`
    pub(crate) id: Uuid,
    // The `Uuid` for the `Player` having the `PlayerRole`
    pub(crate) player_id: Uuid,
    // The `id` for the `Role` had by the `Player`
    pub(crate) role_id: i32,
    /// The timestamp of the creation of the `PlayerRole`
    pub(crate) created: NaiveDateTime,
    /// The timestamp of the last update to `PlayerRole`
    pub(crate) updated: NaiveDateTime,
    /// The `Uuid` of the `Player` who created the `PlayerRole`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `Player` who most recently updated the `PlayerRole`
    pub(crate) updatedby: Uuid,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = player)]
#[diesel(check_for_backend(diesel::pg::Pg))]
// A simpler version of a `Player` for internal representation ONLY
pub(crate) struct InternalPlayer {
    // The `Uuid` for the `Player`
    pub(crate) id: Uuid,
    /// The id of the `Team` to which the `Player` is primarily aligned (may be dead)
    pub(crate) main_team: Option<i32>,
    /// The id of the `Team` to which the `Player` is currently aligned (will not be dead)
    pub(crate) playing_for: Option<i32>,
    /// Whether the player is an alt
    pub(crate) is_alt: bool,
    /// Whether the player must captcha or not, not displayed to end player
    pub(crate) must_captcha: bool,
    /// The timestamp of the creation of the `InternalPlayer`
    pub(crate) created: NaiveDateTime,
    /// The timestamp of the last update to `InternalPlayer`
    pub(crate) updated: NaiveDateTime,
    /// The `Uuid` of the `InternalPlayer` who most created the `InternalPlayer`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `InternalPlayer` who most recently updated `InternalPlayer`
    pub(crate) updatedby: Uuid,
}

pub trait AuthProvider {
    fn platform(&self) -> String;
    fn foreign_id(&self) -> String;
    fn foreign_name(&self) -> Option<String>;
}

impl Role {
    pub fn by_player_id(player_id: Uuid, conn: &mut PgConnection) -> crate::error::Result<Vec<Role>> {
        use crate::schema::{role, player_role};
        role::table
        .inner_join(player_role::dsl::player_role)
        .filter(player_role::player_id.eq(player_id))
        .select(Self::as_select())
        .load(conn)
        .map_rre()
    }
}

impl InternalPlayer {
    pub fn login_player(
        login: impl AuthProvider,
        player_agent: UA,
        ip_address: Cip,
        conn: &mut PgConnection,
    ) -> Result<Session> {
        // Get AuthenticationMethod
        let authentication_method =
            match AuthenticationMethod::get(login.platform(), login.foreign_id(), conn) {
                Ok(x) => x,
                Err(_) => {
                    let player = InternalPlayer::new(
                        format!(
                            "{}_{}",
                            login.platform(),
                            login.foreign_name().unwrap_or(
                                rand::thread_rng()
                                    .sample_iter(&Alphanumeric)
                                    .take(7)
                                    .map(char::from)
                                    .collect()
                            )
                        ),
                        conn,
                    )?;
                    AuthenticationMethod::new(
                        player.id(),
                        login.platform(),
                        login.foreign_id(),
                        login.foreign_name(),
                        SYSTEM_UUID,
                        conn,
                    )?
                }
            };
        // Create Session
        Session::new(
            authentication_method.player_id(),
            authentication_method.id(),
            Some((Utc::now() + Duration::seconds(COOKIE_DURATION)).naive_utc()),
            player_agent.into(),
            ip_address.into(),
            conn,
        )
    }

    pub fn new(_name: String, conn: &mut PgConnection) -> Result<Self> {
        use crate::schema::player::dsl::*;
        Ok(insert_into(player)
            .values(name.eq(_name))
            .returning((
                id,
                main_team,
                playing_for,
                is_alt,
                must_captcha,
                created,
                updated,
                createdby,
                updatedby,
            ))
            .get_result(conn)?)
    }

    pub fn id(&self) -> Uuid {
        self.id
    }
}

impl From<Player> for InternalPlayer {
    fn from(w: Player) -> InternalPlayer {
        InternalPlayer {
            id: w.id,
            main_team: w.main_team,
            playing_for: w.playing_for,
            is_alt: w.is_alt,
            must_captcha: w.must_captcha,
            created: w.created,
            updated: w.updated,
            createdby: w.createdby,
            updatedby: w.updatedby,
        }
    }
}

pub(crate) struct Cip(pub Option<String>);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for Cip {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(Cip(Some(
            request
                .headers()
                .get("CF-Connecting-IP")
                .collect::<String>(),
        )))
    }
}

impl From<Cip> for Option<IpNetwork> {
    fn from(w: Cip) -> Option<IpNetwork> {
        use std::str::FromStr;
        if let Some(wx) = w.0 {
            IpNetwork::from_str(&wx).ok()
        } else {
            None
        }
    }
}

pub(crate) struct UA(pub Option<String>);

#[rocket::async_trait]
impl<'a> FromRequest<'a> for UA {
    type Error = ();

    async fn from_request(request: &'a Request<'_>) -> request::Outcome<Self, Self::Error> {
        Outcome::Success(UA(Some(
            request.headers().get("Player-Agent").collect::<String>(),
        )))
    }
}

impl From<UA> for String {
    fn from(w: UA) -> String {
        w.0.unwrap_or("unknown".to_string())
    }
}
