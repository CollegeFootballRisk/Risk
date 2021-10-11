/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::db::DbConn;
use crate::model::{Captchas, UserCaptcha};
use base64::encode;
use captcha::{gen, Difficulty};
use diesel::prelude::*;
use rocket::http::Status;
use rocket::serde::json::Json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[get("/captcha")]
pub async fn captchaServe(conn: DbConn) -> Result<Json<UserCaptcha>, Status> {
    let (_solution, png) = create_captcha(Difficulty::Easy).unwrap();
    let insert_captcha = Captchas {
        title: calculate_hash(&_solution).to_string()[0..7].to_string(),
        content: _solution[..].to_string(),
    };

    let result: QueryResult<usize> = conn.run(|c| Captchas::insert(insert_captcha, c)).await;
    let outlet = result.unwrap_or(0);
    match outlet {
        1 => {
            std::result::Result::Ok(Json(UserCaptcha {
                title: calculate_hash(&_solution).to_string(),
                content: encode(&png),
            }))
        }
        _ => std::result::Result::Err(Status::NotFound),
    }
    //encode(&png)
    //    Captcha::new()
    //        .add_chars(5)
    //        .apply_filter(Noise::new(0.4))
    //        .apply_filter(Wave::new(2.0, 20.0).horizontal())
    //        .apply_filter(Wave::new(2.0, 20.0).vertical())
    //        .view(220, 120)
    //        .apply_filter(Dots::new(15))
    //        .as_png();
}
pub fn create_captcha(d: Difficulty) -> Option<(String, Vec<u8>)> {
    gen(d).as_tuple()
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
