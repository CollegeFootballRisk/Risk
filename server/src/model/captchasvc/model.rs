use crate::schema::*;
use diesel::prelude::*;
#[derive(Deserialize, Insertable)]
#[table_name = "captchas"]
pub struct Captchas<'a> {
    pub title: &'a str,
    pub content: &'a str,
}
#[derive(Serialize, Deserialize)]
pub struct UserCaptcha {
    pub title: String,
    pub content: String,
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

impl Captchas<'_> {
    pub fn insert(insert_captcha: Captchas, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(captchas::table).values(&insert_captcha).execute(conn)
    }

    pub fn delete(delete_captcha: Captchas, conn: &PgConnection) -> QueryResult<usize> {
        diesel::delete(captchas::table)
            .filter(captchas::title.eq(&delete_captcha.title[0..7]))
            .filter(captchas::content.eq(delete_captcha.content))
            .execute(conn)
    }
}
