use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum AuthMethod {
    Password,
    OAuth,
    ApiKey,
    MagicLink,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pending,
    Verified,
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub auth_method: AuthMethod,
    pub profile_picture: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub verification_status: VerificationStatus,
}

// fn main() {
//     let user = User {
//         id: Uuid::new_v4(),
//         name: "John Doe".to_string(),
//         email: "john.doe@example.com".to_string(),
//         auth_method: AuthMethod::OAuth,
//         profile_picture: Some("https://example.com/profile.jpg".to_string()),
//         bio: Some("A software developer.".to_string()),
//         location: Some("New York".to_string()),
//         website: Some("https://johndoe.com".to_string()),
//         created_at: Utc::now(),
//         last_login: Some(Utc::now()),
//         is_active: true,
//         verification_status: VerificationStatus::Verified,
//     };
//
//     println!("User: {:?}", user);
// }
