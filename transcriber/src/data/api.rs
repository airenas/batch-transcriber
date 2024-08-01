use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ASRMessage {
    pub id: String,
    pub file: String,
    pub base_dir: String,
}

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ResultMessage {
    pub id: String,
    pub finished: bool,
    pub file: String,
    pub base_dir: String,
    pub error: Option<String>,
}
