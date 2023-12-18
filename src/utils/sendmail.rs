use crate::config::Config;
use lettre::{
    self,
    message::{header::ContentType, Mailbox},
    transport::smtp::{authentication::Credentials, response::Response},
    Message, SmtpTransport, Transport,
};

pub fn sendmail(
    env: &Config,
    receiver_name: &str,
    receiver_email: &str,
    subject: &str,
    content: &str,
) -> Result<Response, String> {
    let from: Mailbox = format!("{} <{}>", env.smtp_from_name, env.smtp_from_email)
        .parse()
        .map_err(|_| format!("Invalid from address: {}", env.smtp_from_email))?;

    let to: Mailbox = format!("{} <{}>", receiver_name, receiver_email)
        .parse()
        .map_err(|_| format!("Invalid to address: {}", receiver_email))?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .header(ContentType::TEXT_PLAIN)
        .subject(subject)
        .body(content.to_string())
        .map_err(|_| "Failed to build email")?;

    let creds = Credentials::new(env.smtp_username.to_string(), env.smtp_password.to_string());

    let host = env.smtp_host.as_ref().ok_or("Invalid smtp host")?;

    let mailer = SmtpTransport::relay(host)
        .map_err(|_| "Failed to create mailer")?
        .credentials(creds)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())
}
