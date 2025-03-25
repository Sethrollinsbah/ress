use axum::response::IntoResponse;
use axum::Json;
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    // pub success: bool,
    // pub message: String,
}

pub async fn send_mail(
    _domain: &str,
    recipient_email: &str,
    recipient_name: &str,
    subject: &str,
    html_body: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    // Retrieve API token, with error handling
    let api_token =
        std::env::var("ZOHO_KEY").map_err(|_| "ZOHO_KEY environment variable not set")?;

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
