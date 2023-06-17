/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::captchas;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
#[derive(Deserialize, Insertable, Queryable)]
#[diesel(table_name = captchas)]
pub struct Captchas {
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) creation: NaiveDateTime,
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
    pub fn insert(insert_captcha: Captchas, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(captchas::table)
            .values(&insert_captcha)
            .execute(conn)
    }

    pub fn check(
        title: String,
        content: String,
        conn: &mut PgConnection,
    ) -> Result<bool, diesel::result::Error> {
        let true_content = captchas::table
            .filter(captchas::title.eq(title))
            .select((captchas::title, captchas::content, captchas::creation))
            .first::<Captchas>(conn)?;
        true_content.delete(conn)?;
        Ok(
            true_content.creation.timestamp() - Utc::now().naive_utc().timestamp() < 600
                && content == true_content.content,
        )
    }

    pub fn delete(&self, conn: &mut PgConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(captchas::table)
            .filter(captchas::title.eq(&self.title))
            .filter(captchas::content.eq(&self.content))
            .execute(conn)
    }
}
