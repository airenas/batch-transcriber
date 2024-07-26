use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize, Clone)]
pub struct ASRMessage {
    pub id: String,
    pub file: String,
    pub base_dir: String,
}
