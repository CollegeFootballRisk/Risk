/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
#[macro_use]
extern crate rocket;

use rust_risk::rocket_launcher::launcher;
#[launch]
fn rocket() -> _ {
    launcher()
}
