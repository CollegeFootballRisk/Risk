#[derive(Deserialize)]
pub struct RedditUserInfo {
    #[serde(default)]
    pub name: String,
}
