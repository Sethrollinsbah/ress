use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserData {
    pub email: Vec<String>,
    pub name: String,
    pub status: u16,
}


