/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::AuthProvider;

/// Login cog for permitting those with the `LoginAs` permission to login as a given user
struct LoginAs {
    foreign_id: String,
    foreign_name: String,
}

impl AuthProvider for LoginAs {
    fn platform(&self) -> String {
        String::from("loginas")
    }
    fn foreign_id(&self) -> String {
        self.foreign_id.clone()
    }
    fn foreign_name(&self) -> Option<String> {
        Some(self.foreign_name.clone())
    }
}

//#[protect("LoginAs")]
#[get("/loginas/<user_id>")]
pub(crate) async fn callback(
    token: TokenResponse<Reddit>,
    cookies: &CookieJar<'_>,
    cip: Cip,
    ua: UA,
    conn: DbConn,
    config: &State<SysInfo>,
    user_id: Uuid,
) -> Result<Redirect, crate::Error> {
    // Verify that the specified exists
    InternalPlayer::get(user_id);
    // Create a new `Session`
    todo!();
}
