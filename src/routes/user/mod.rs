mod delete;
mod favorite_bookmark;
mod get_check;
mod modify;
mod reset;
mod verify;

use super::{build_err_resp, check_pass};
use crate::config::Config;
use lettre::{
    self,
    message::{header::ContentType, Mailbox},
    transport::smtp::{authentication::Credentials, response::Response},
    Message, SmtpTransport, Transport,
};
use tracing::warn;

pub use delete::{get_delete, post_delete, DeleteRequestBody};
pub use favorite_bookmark::{delete_bookmark, delete_favorite, put_bookmark, put_favorite};
pub use get_check::get_check;
pub use modify::{post_modify, ModifyRequestBody};
pub use reset::{get_reset, post_reset, ResetRequestBody};
pub use verify::{get_verify, post_verify};

pub use delete::{__path_get_delete, __path_post_delete};
pub use favorite_bookmark::{
    __path_delete_bookmark, __path_delete_favorite, __path_put_bookmark, __path_put_favorite,
};
pub use get_check::__path_get_check;
pub use modify::__path_post_modify;
pub use reset::{__path_get_reset, __path_post_reset};
pub use verify::{__path_get_verify, __path_post_verify};

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

    let host = env.smtp_host.as_ref().ok_or_else(|| {
        let err = "Invalid smtp host";
        warn!(err);
        err
    })?;

    let mailer = SmtpTransport::relay(host)
        .map_err(|_| "Failed to create mailer")?
        .credentials(creds)
        .build();

    mailer.send(&email).map_err(|e| e.to_string())
}
