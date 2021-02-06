#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SysInfo {
    pub name: String,
    pub base_url: String,
    pub version: String,
    pub discord: bool,
    pub reddit: bool,
    pub groupme: bool,
}