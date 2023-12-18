use crate::config::Config;
use lettre::{
    self,
    message::header::ContentType,
    transport::smtp::{authentication::Credentials, response::Response, Error},
    Message, SmtpTransport, Transport,
};

pub fn sendmail(
    env: &Config,
    receiver_name: &str,
    receiver_email: &str,
    subject: &str,
    content: &str,
) -> Result<Response, Error> {
    let email = Message::builder()
        .from(
            format!("{} <{}>", env.smtp_from_name, env.smtp_from_email)
                .parse()
                .unwrap(),
        )
        .to(format!("{} <{}>", receiver_name, receiver_email)
            .parse()
            .unwrap())
        .header(ContentType::TEXT_PLAIN)
        .subject(subject)
        .body(content.to_string())
        .unwrap();

    let creds = Credentials::new(env.smtp_username.to_string(), env.smtp_password.to_string());

    let mailer = SmtpTransport::relay(env.smtp_host.as_ref().unwrap())
        .unwrap()
        .credentials(creds)
        .build();

    mailer.send(&email)
}
