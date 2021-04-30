/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod auth;
pub mod discord;
pub mod player;
pub mod ratings;
pub mod reddit;
pub mod stats;
pub mod sys;
pub mod team;
pub mod territory;
pub mod turn;
pub mod user;
pub use auth::*;
pub use player::*;
pub use ratings::*;
pub use stats::*;
pub use sys::*;
pub use team::*;
pub use territory::*;
pub use turn::*;
pub use user::*;
#[cfg(feature = "risk_captcha")]
pub mod captchasvc;
#[cfg(feature = "risk_captcha")]
pub use captchasvc::*;
#[cfg(feature = "risk_discord")]
pub use discord::*;
#[cfg(feature = "risk_reddit")]
pub use reddit::*;
