#[derive(Deserialize, Debug)]
pub struct DiscordUserInfo {
    #[serde(default)]
    pub id: String,
    pub username: String,
    pub discriminator: String,
}

impl DiscordUserInfo {
    pub fn name(&self) -> String {
        self.username.clone() + &String::from("#") + &self.discriminatoràà
    }
}
