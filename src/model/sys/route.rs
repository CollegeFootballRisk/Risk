/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::catchers::Status;
use rocket::serde::json::Json;

/// # System Information
/// Gives some information about some configuration.
#[openapi(tag = "System")]
#[get("/sys/info")]
pub(crate) async fn sysinfo() -> Result<Json<PubSysInfo>, Status> {
    std::result::Result::Ok(Json(PubSysInfo::default()))
}

#[allow(clippy::struct_excessive_bools, unreachable_pub)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct PubSysInfo {
    pub(crate) version: String,
    pub(crate) discord: bool,
    pub(crate) reddit: bool,
    pub(crate) groupme: bool,
    pub(crate) chaos: bool,
    pub(crate) image: bool,
    pub(crate) captcha: bool,
}

impl Default for PubSysInfo {
    fn default() -> PubSysInfo {
        PubSysInfo {
            version: git_version::git_version!(
                fallback = option_env!("GIT_HASH").unwrap_or(env!("CARGO_PKG_VERSION"))
            )
            .to_string(),
            discord: cfg!(feature = "risk_discord"),
            reddit: cfg!(feature = "risk_reddit"),
            groupme: cfg!(feature = "risk_groupme"),
            image: cfg!(feature = "risk_image"),
            chaos: cfg!(feature = "chaos"),
            captcha: cfg!(feature = "risk_captcha"),
        }
    }
}
