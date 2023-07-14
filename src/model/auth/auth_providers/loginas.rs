/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::AuthProvider;

struct LoginAs {
    foreign_id: String,
    foreign_name: String,
}

impl AuthProvider for LoginAs {
    fn platform(&self) -> String {
        String::from("loginas")
    }
    fn foreign_id(&self) -> String {
        foreign_id.clone()
    }
    fn foreign_name(&self) -> Option<String> {
        Some(foreign_name.clone())
    }
}
