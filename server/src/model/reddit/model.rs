#[derive(Deserialize,Debug)]
pub struct RedditUserInfo {
    #[serde(default)]
    pub name: String,
}
