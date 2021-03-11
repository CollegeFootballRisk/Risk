use rocket::response::NamedFile;

#[get("/robots.txt")]
pub async fn robots() -> String {
    "User-agent: *
    Disallow:
    Disallow: /auth/"
        .to_string()
}

// These are JS Routes
#[get("/<_data>", rank = 1)]
pub async fn js_api_leaderboard(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/territory/<_data>", rank = 1)]
pub async fn js_api_territory(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/team/<_data>", rank = 1)]
pub async fn js_api_team(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/team/<_data>/players", rank = 1)]
pub async fn js_api_team_players(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/player/<_data>", rank = 1)]
pub async fn js_api_player(_data: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/map/<_data>/<_data2>", rank = 1)]
pub async fn js_api_map(_data: Option<String>, _data2: Option<String>) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}

#[get("/territory/<_territory>/<_data>/<_data2>", rank = 1)]
pub async fn js_api_territories(
    _data: Option<String>,
    _territory: Option<String>,
    _data2: Option<String>,
) -> NamedFile {
    NamedFile::open("static/index.html").await.ok().unwrap()
    // We are assuming index.html exists. If it does not, uh oh!
}
