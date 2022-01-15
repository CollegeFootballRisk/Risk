/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::captchas;
use diesel::prelude::*;
#[derive(Deserialize, Insertable)]
#[table_name = "captchas"]
pub struct Captchas {
    pub(crate) title: String,
    pub(crate) content: String,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct UserCaptcha {
    pub(crate) title: String,
    pub(crate) content: String,
}

//pub enum CaptchaError {
//    InvalidParameters,
//    CaptchaGeneration,
//    Uuid,
//    ToJson,
//    Persist,
//    NotFound,
//    Unexpected,
//}

impl Captchas {
    pub fn insert(insert_captcha: Captchas, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(captchas::table)
            .values(&insert_captcha)
            .execute(conn)
    }

    pub fn delete(delete_captcha: Captchas, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(captchas::table)
            .filter(captchas::title.eq(&delete_captcha.title[0..7]))
            .filter(captchas::content.eq(delete_captcha.content))
            .execute(conn)
    }
}
