/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::diesel::Connection;
use crate::diesel::RunQueryDsl;
use crate::model::sys::route::SystemInformation;
use crate::rocket_launcher::launcher;
use diesel::PgConnection;
use rocket::http::ContentType;
use rocket::http::Status;
use rocket::local::blocking::Client;

struct TestContext {
    base_url: String,
    db_name: String,
}

impl TestContext {
    fn new(base_url: &str, db_name: &str) -> Self {
        // First, connect to postgres db to be able to create our test
        // database.
        let postgres_url = format!("{}/postgres", base_url);
        let mut conn =
            PgConnection::establish(&postgres_url).expect("Cannot connect to postgres database.");

        // Create a new database for the test
        let query = diesel::sql_query(format!("CREATE DATABASE {}", db_name).as_str());
        query
            .execute(&mut conn)
            .expect(format!("Could not create database {}", db_name).as_str());

        Self {
            base_url: base_url.to_string(),
            db_name: db_name.to_string(),
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let postgres_url = format!("{}/postgres", self.base_url);
        let mut conn =
            PgConnection::establish(&postgres_url).expect("Cannot connect to postgres database.");

        let disconnect_users = format!(
            "SELECT pg_terminate_backend(pid)
FROM pg_stat_activity
WHERE datname = '{}';",
            self.db_name
        );

        diesel::sql_query(disconnect_users.as_str())
            .execute(&mut conn)
            .unwrap();

        let query = diesel::sql_query(format!("DROP DATABASE {}", self.db_name).as_str());
        query
            .execute(&mut conn)
            .expect(&format!("Couldn't drop database {}", self.db_name));
    }
}

#[test]
fn try_database_create_and_drop() {
    let database_url: String = rocket::build()
        .figment()
        .extract_inner("databases.postgres_tests.url_base")
        .expect("Database root not set in configuration.");
    // Needs to be created first.
    let _ctx = TestContext::new(&database_url, "risktest00");
    // Do your test here
}

#[test]
fn test_system_api() {
    let client = Client::tracked(launcher()).expect("valid rocket instance");
    let response = client.get("/api/sys/info").dispatch();
    assert_eq!(response.status(), Status::Ok);
    assert_eq!(
        serde_json::from_str::<SystemInformation>(&response.into_string().unwrap()).unwrap(),
        SystemInformation::default()
    );
}

#[test]
fn test_username_changes() {
    let client = Client::tracked(launcher()).expect("valid rocket instance");
    let response_too_short = client
        .post("/auth/me/username")
        .header(ContentType::Form)
        .body("username=xxx")
        .dispatch();
    assert_eq!(response_too_short.status(), Status::UnprocessableEntity);

    let response_too_long = client.post("/auth/me/username").header(ContentType::Form).body("username=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").dispatch();
    assert_eq!(response_too_long.status(), Status::UnprocessableEntity);

    let response_okay = client
        .post("/auth/me/username")
        .header(ContentType::Form)
        .body("username=xxxxx")
        .dispatch();
    assert_eq!(response_okay.status(), Status::Ok);
}
