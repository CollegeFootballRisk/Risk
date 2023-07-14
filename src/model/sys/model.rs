/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rand::distributions::Alphanumeric;
use rand::Rng;
use rocket_oauth2::OAuthConfig;

// Since we're not using a state machine here
#[allow(clippy::struct_excessive_bools, unreachable_pub)]
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct SysInfo {
    pub(crate) version: String,
    pub(crate) discord: bool,
    pub(crate) reddit: bool,
    pub(crate) groupme: bool,
    pub(crate) image: bool,
    pub(crate) captcha: bool,
    #[serde(skip_serializing)]
    pub(crate) settings: SysSettings,
    #[serde(skip)]
    pub(crate) reddit_config: Option<OAuthConfig>,
    #[serde(skip)]
    pub(crate) discord_config: Option<OAuthConfig>,
}

#[allow(dead_code)]
pub(crate) struct AppSettings {
    // Time string for the next roll
    pub(crate) rolltime: String,
    // Map URL
    pub(crate) map: String,
    // ViewBox for the map:
    pub(crate) viewbox: String,
    // Website Title:
    pub(crate) webtitle: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub(crate) struct SysSettings {
    pub(crate) name: String,
    pub(crate) base_url: String,
    pub(crate) cookie_key: String,
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
            reddit_config: None,
            discord_config: None,
        }
    }
}

impl Default for SysSettings {
    fn default() -> SysSettings {
        SysSettings {
            name: String::from("RustRisk Local"),
            base_url: String::from("http://localhost:8000"),
            cookie_key: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(24)
                .map(char::from)
                .collect(),
        }
    }
}
