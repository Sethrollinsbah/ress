use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub email: Vec<String>,
    pub name: String,
    pub status: u16,
}
