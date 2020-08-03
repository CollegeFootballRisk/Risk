use rocket_contrib::json::Json;
#[derive(Serialize, Deserialize)]
pub struct Httperror {
    pub status: i32,
}

#[catch(404)]
pub fn not_found() -> Json<Httperror> {
    Json(Httperror { status: 404 })
}

#[catch(500)]
pub fn internal_error() -> Json<Httperror> {
    Json(Httperror { status: 500 })
}
