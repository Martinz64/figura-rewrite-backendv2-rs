use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Stage1LoginParams {
    pub username: String
}
impl Default for Stage1LoginParams {
    fn default() -> Self {
        Self {
            username: "".to_string(),
        }
    }
}

#[derive(Deserialize)]
#[serde(default)]
pub struct Stage2LoginParams {
    pub id: String
}
impl Default for Stage2LoginParams {
    fn default() -> Self {
        Self {
            id: "".to_string(),
        }
    }
}

pub struct UserInfo {
    pub uuid: uuid::Uuid,
    pub username: String
}