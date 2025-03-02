// src/services/auth_service.rs

use crate::models::database::user::{User, AuthMethod, VerificationStatus}; // Assuming you have a User model
use uuid::Uuid;
use chrono::{DateTime, Utc};

pub async fn register_user(
    name: String,
    email: String,
    password: String,
) -> Result<User, String> {
    // Implement user registration logic here
    // ...
    let new_user = User {
        id: Uuid::new_v4(),
        name,
        email,
        auth_method: AuthMethod::MagicLink,
        profile_picture: "https://bucket.planetbun.com/profile_picture/0",
        bio: None,
        location: None,
        website: None,
        created_at: Utc::now(),
        last_login: None,
        is_active: true,
        verification_status: VerificationStatus
    };

    Ok(new_user)
}
