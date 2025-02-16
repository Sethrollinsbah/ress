use axum::extract::Query;
use axum::response::IntoResponse;
use axum::Json;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;

#[derive(Serialize)]
pub struct EmailAddress {
    pub address: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct From {
    pub address: String,
}

#[derive(Serialize)]
pub struct To {
    pub email_address: EmailAddress,
}

#[derive(Serialize)]
pub struct EmailRequest {
    pub from: From,
    pub to: Vec<To>,
    pub subject: String,
    pub htmlbody: String,
}

#[derive(Debug, Deserialize)]
pub struct EmailResponse {
    pub success: bool,
    pub message: String,
}

pub async fn send_mail(
    domain: &str,
    recipient_email: &str,
    recipient_name: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let api_token = "Zoho-enczapikey wSsVR60n/BH5XK8pnDb8I7trkAtUBV32FER03QGg4iX1GqjLoMc8wRCcAwL1GfhJGWI7FjFD8L8vkE9T0zUPjt9+yg4DCSiF9mqRe1U4J3x17qnvhDzIXmVekRSAKosPwwtimWNkFMlu";
    let client = Client::new();

    let email_request = EmailRequest {
        from: From {
            address: "noreply@planetbun.com".to_string(),
        },
        to: vec![To {
            email_address: EmailAddress {
                address: recipient_email.to_string(),
                name: recipient_name.to_string(),
            },
        }],
        subject: subject.to_string(),
        htmlbody: html_body.to_string(),
    };

    // Serialize the `email_request` struct into a JSON string
    let json_body = serde_json::to_string(&email_request)?;

    let response = client
        .post("https://api.zeptomail.com/v1.1/email")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("Authorization", api_token)
        .body(json_body)
        .send()
        .await?;

    // Print the raw response body for debugging

    let raw_response = response.text().await?;
    println!("Raw API response: {}", raw_response);
    //if response.status().is_success() {
    //    let response_body: EmailResponse = response.json().await?;
    //    println!("Email sent successfully: {:?}", response_body);
    //} else {
    //    let error_body = response.text().await?;
    //    println!("Failed to send email: {}", error_body);
    //}

    Ok(())
}
#[derive(Deserialize)]
pub struct ParamsSendMail {
    domain: String,
    recipient_email: String,
    recipient_name: String,
    subject: String,
    html_body: String,
}

pub async fn send_mail_handler(Json(params): Json<ParamsSendMail>) -> impl IntoResponse {
    match send_mail(
        &params.domain,
        &params.recipient_email,
        &params.recipient_name,
        &params.subject,
        &params.html_body,
    )
    .await
    {
        Ok(_) => Json(json!({"status": "success"})),
        Err(e) => Json(json!({
            "status": "error",
            "message": e.to_string()
        })),
    }
}
