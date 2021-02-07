#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SysInfo {
    pub name: String,
    pub base_url: String,
    pub version: String,
    pub discord: bool,
    pub reddit: bool,
    pub groupme: bool,
    pub image: bool,
    pub captcha: bool,
}

impl SysInfo {
    pub fn default() -> SysInfo{
        SysInfo {
            name: String::from("AggieRisk Local"),
            base_url: String::from("http://localhost:8000"),
            version: env!("CARGO_PKG_VERSION").to_string(),
            discord: false,
            reddit: true,
            groupme: false,
            image: false,
            captcha: false
        }
    }
}