/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rand::distributions::Alphanumeric;
use rand::Rng;
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct SysInfo {
    pub version: String,
    pub discord: bool,
    pub reddit: bool,
    pub groupme: bool,
    pub image: bool,
    pub captcha: bool,
    pub settings: SysSettings,
}
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct SysSettings {
    pub name: String,
    pub base_url: String,
    pub cookie_key: String,
}

impl Default for SysInfo {
    fn default() -> SysInfo {
        SysInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            discord: false,
            reddit: true,
            groupme: false,
            image: false,
            captcha: false,
            settings: SysSettings::default(),
        }
    }
}

impl Default for SysSettings {
    fn default() -> SysSettings {
        SysSettings {
            name: String::from("AggieRisk Local"),
            base_url: String::from("http://localhost:8000"),
            cookie_key: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(24)
                .map(char::from)
                .collect(),
        }
    }
}
