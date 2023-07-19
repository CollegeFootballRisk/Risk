/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::catchers::Status;
use rocket::serde::json::Json;

/// # System Information
/// Information about the configuration of the backend.
#[openapi(tag = "System")]
#[get("/sys/info")]
pub(crate) async fn sysinfo() -> Result<Json<SystemInformation>, Status> {
    std::result::Result::Ok(Json(SystemInformation::default()))
}

#[allow(clippy::struct_excessive_bools, unreachable_pub)]
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug, PartialEq)]
/// # System Information
/// Information about the configuration of the backend.
pub struct SystemInformation {
    /// The git version of the backend at compile time. `-modified` suffix
    /// indicates changes were made post-pull.
    pub(crate) version: String,
    /// Whether login by Discord is enabled (true) or disabled (false).
    pub(crate) discord: bool,
    /// Whether login by Reddit is enabled (true) or disabled (false).
    pub(crate) reddit: bool,
    /// Whether login by GroupMe is enabled (true) or disabled (false).
    pub(crate) groupme: bool,
    /// Whether chaos
    pub(crate) chaos: bool,
    /// Whether the `image` feature (deprecated), that creates images for sharing at roll time, is enabled.
    pub(crate) image: bool,
    /// Whether the `captcha` feature (deprecated), that requires possible likely alts to enter a captcha, is enabled.
    pub(crate) captcha: bool,
}

impl Default for SystemInformation {
    fn default() -> SystemInformation {
        SystemInformation {
            version: git_version::git_version!(
                fallback = option_env!("GIT_HASH").unwrap_or(env!("CARGO_PKG_VERSION"))
            )
            .to_string(),
            discord: cfg!(feature = "discord"),
            reddit: cfg!(feature = "reddit"),
            groupme: cfg!(feature = "groupme"),
            image: cfg!(feature = "image"),
            chaos: cfg!(feature = "chaos"),
            captcha: cfg!(feature = "captcha"),
        }
    }
}
