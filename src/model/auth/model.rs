/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::User;
use crate::schema::{authentication_method, permission, role, user, user_role};
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
#[diesel(check_for_backend(diesel::pg::Pg))]
/// An `AuthenticationMethod` denotes a method for logging in for a User, e.g.
/// Reddit, Discord, etc.
pub(crate) struct AuthenticationMethod {
    pub(crate) id: Uuid,
    pub(crate) user_id: Uuid,
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
}

impl AuthenticationMethod {
    pub(crate) fn new(
        _user_id: Uuid,
        _platform: String,
        _foreign_id: String,
        _foreign_name: Option<String>,
        _createdby: Uuid,
        conn: &mut PgConnection,
    ) -> Result<Self, crate::Error> {
        let _createdby: Uuid = _createdby;
        use crate::schema::authentication_method::dsl::*;
        Ok(insert_into(authentication_method)
            .values((
                user_id.eq(_user_id),
                platform.eq(_platform),
                foreign_id.eq(_foreign_id),
                foreign_name.eq(_foreign_name),
                createdby.eq(_createdby),
                updatedby.eq(_createdby)
            ))
            .get_result(conn)?)
    }

    /// Retrieve an `AuthenticationMethod` using the unique pair (`platform`, `foreign_id`)
    pub(crate) fn get(
        _platform: String,
        _foreign_id: String,
        conn: &mut PgConnection,
    ) -> Result<Self, crate::Error> {
        use crate::schema::authentication_method::dsl::*;
        Ok(authentication_method
            .filter(platform.eq(_platform))
            .filter(foreign_id.eq(_foreign_id))
            .select(AuthenticationMethod::as_select())
            .get_result(conn)?)
    }

    pub(crate) fn get_by_id(_id: Uuid, conn: &mut PgConnection) -> Result<Self, crate::Error> {
        use crate::schema::authentication_method::dsl::*;
        Ok(authentication_method
            .filter(id.eq(_id))
            .select(AuthenticationMethod::as_select())
            .get_result(conn)?)
    }

    pub(crate) fn id(&self) -> Uuid {
        self.id
    }

    pub(crate) fn user_id(&self) -> Uuid {
        self.user_id
    }
}

#[derive(Queryable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = session)]
#[diesel(check_for_backend(diesel::pg::Pg))]
/// A `Session` denote an active instance of a `User` (i.e. a logged-in user)
pub(crate) struct Session {
    #[diesel(deserialize_as = Uuid)]
    pub(crate) id: Option<Uuid>,
    pub(crate) user_id: Uuid,
    pub(crate) authentication_method_id: Uuid,
    pub(crate) is_active: bool,
    pub(crate) user_agent: String,
    pub(crate) ip_address: IpNetwork,
    #[diesel(deserialize_as = NaiveDateTime)]
    pub(crate) created: Option<NaiveDateTime>,
    pub(crate) expires: Option<NaiveDateTime>,
}

impl Session {
    /// Create a new `Session` by insertion
    ///
    /// # Fields
    /// _user_id: the Uuid of the `User`
    /// _authentication_method_id: the Uuid of the `AuthenticationMethod` a user has
    /// _expires: optional `NaiveDateTime` for the token to expire.
    pub(crate) fn new(
        _user_id: Uuid,
        _authentication_method_id: Uuid,
        _expires: Option<NaiveDateTime>,
        _user_agent: String,
        _ip_address: Option<IpNetwork>,
        conn: &mut PgConnection,
    ) -> Result<Self, crate::Error> {
        use crate::schema::session::dsl::*;
        // Ok(..) coerces this into a Session
        Ok(insert_into(session)
            .values((
                user_id.eq(_user_id),
                authentication_method_id.eq(_authentication_method_id),
                expires.eq(_expires),
                user_agent.eq(_user_agent),
                ip_address.eq(_ip_address.unwrap_or("1.1.1.1".parse::<IpNetwork>().unwrap())),
            ))
            .get_result(conn)?)
    }

    pub(crate) fn put(&self, key: &[u8]) -> Result<String, crate::Error> {
        encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(key),
        ).map_err(|_| crate::Error::InternalServerError{})
    }

    pub(crate) fn interpret(key: &[u8], token: String) -> Result<(Session, Header), String> {
        let validation = Validation::default();
        match decode::<Session>(&token, &DecodingKey::from_secret(key), &validation) {
            Ok(c) => Ok((c.claims, c.header)),
            Err(err) => Err(err.to_string()),
        }
    }

    pub(crate) fn from_private_cookie(
        cookies: &CookieJar<'_>,
        config: &State<SysInfo>,
    ) -> Result<(Session, Header), crate::Error> {
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
    /// The `Uuid` of the `User` who created the `Permission`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `User` who most recently updated `Permission`
    pub(crate) updatedby: Uuid,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = role)]
#[diesel(belongs_to(UserRole))]
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
    /// The `Uuid` of the `User` who created the `Role`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `User` who most recently updated `Role`
    pub(crate) updatedby: Uuid,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = user_role)]
#[diesel(belongs_to(InternalUser))]
#[diesel(belongs_to(Role))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct UserRole {
    // The `Uuid` for the `UserRole`
    pub(crate) id: Uuid,
    // The `Uuid` for the `User` having the `UserRole`
    pub(crate) user_id: Uuid,
    // The `id` for the `Role` had by the `User`
    pub(crate) role_id: i32,
    /// The timestamp of the creation of the `UserRole`
    pub(crate) created: NaiveDateTime,
    /// The timestamp of the last update to `UserRole`
    pub(crate) updated: NaiveDateTime,
    /// The `Uuid` of the `User` who created the `UserRole`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `User` who most recently updated the `UserRole`
    pub(crate) updatedby: Uuid,
}

#[derive(Queryable, Identifiable, Selectable, Debug, Serialize, Deserialize, Clone)]
#[diesel(table_name = user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
// A simpler version of a `User` for internal representation ONLY
pub(crate) struct InternalUser {
    // The `Uuid` for the `User`
    pub(crate) id: Uuid,
    /// The id of the `Team` to which the `User` is primarily aligned (may be dead)
    pub(crate) main_team: Option<i32>,
    /// The id of the `Team` to which the `User` is currently aligned (will not be dead)
    pub(crate) playing_for: Option<i32>,
    /// Whether the user is an alt
    pub(crate) is_alt: bool,
    /// Whether the user must captcha or not, not displayed to end user
    pub(crate) must_captcha: bool,
    /// The timestamp of the creation of the `InternalUser`
    pub(crate) created: NaiveDateTime,
    /// The timestamp of the last update to `InternalUser`
    pub(crate) updated: NaiveDateTime,
    /// The `Uuid` of the `InternalUser` who most created the `InternalUser`
    pub(crate) createdby: Uuid,
    /// The `Uuid` of the `InternalUser` who most recently updated `InternalUser`
    pub(crate) updatedby: Uuid,
}

pub trait AuthProvider {
    fn platform(&self) -> String;
    fn foreign_id(&self) -> String;
    fn foreign_name(&self) -> Option<String>;
}

impl InternalUser {
    pub fn login_user(
        login: impl AuthProvider,
        user_agent: UA,
        ip_address: Cip,
        conn: &mut PgConnection,
    ) -> Result<Session, crate::Error> {
        // Get AuthenticationMethod
        let authentication_method =
            match AuthenticationMethod::get(login.platform(), login.foreign_id(), conn) {
                Ok(x) => x,
                Error => {
                    let user = InternalUser::new(
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
                        user.id(),
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
            authentication_method.user_id(),
            authentication_method.id(),
            Some((Utc::now() + Duration::seconds(COOKIE_DURATION)).naive_utc()),
            user_agent.into(),
            ip_address.into(),
            conn,
        )
    }

    pub fn new(_name: String, conn: &mut PgConnection) -> Result<Self, crate::Error> {
        use crate::schema::user::dsl::*;
        Ok(insert_into(user)
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

impl From<User> for InternalUser {
    fn from(w: User) -> InternalUser {
        InternalUser {
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
            request.headers().get("User-Agent").collect::<String>(),
        )))
    }
}

impl From<UA> for String {
    fn from(w: UA) -> String {
        w.0.unwrap_or("unknown".to_string())
    }
}