/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#[derive(Deserialize, Debug)]
pub(crate) struct DiscordUserInfo {
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) id: String,
    pub(crate) username: String,
    pub(crate) discriminator: String,
}

impl DiscordUserInfo {
    pub(crate) fn name(&self) -> String {
        self.username.clone() + &String::from("#") + &self.discriminator
    }
}
