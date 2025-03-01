use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use lettre_email::EmailBuilder;

#[tokio::main]
async fn send_mail({from: &str, to &str, subject: &str, body: &str}) {
    let email: Message = EmailBuilder::new()
        .from(from)
        .to(to)
        .subject(subject)
        .body(body)
        .build()
        .unwrap()
        .into();

    let creds = Credentials::new("your_email@example.com".to_string(), "your_password".to_string());

    let mailer: SmtpTransport = SmtpTransport::relay("smtp.example.com")
        .unwrap()
        .credentials(creds)
        .build();

    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
