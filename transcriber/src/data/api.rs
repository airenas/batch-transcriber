use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Deserialize)]
pub struct ASRMessage {
    pub id : String,
    pub file: String,
}
