/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#[derive(Deserialize, Debug)]
pub struct DiscordUserInfo {
    #[serde(default)]
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

impl DiscordUserInfo {
    pub fn name(&self) -> String {
        self.username.clone() + &String::from("#") + &self.discriminator
    }
}
