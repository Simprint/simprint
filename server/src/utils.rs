mod config;
mod extractor;
mod jwt;
mod password;
mod responses;
mod secret;
pub mod storage;

pub use config::*;
pub use extractor::*;
pub use jwt::*;
pub use password::*;
pub use responses::*;
pub use secret::*;
pub use storage::*;

/// 向指定邮箱发送验证码
pub fn send_email(
    smtp_username: &str,
    smtp_password: &str,
    smtp_server: &str,
    to: &str,
    title: &str,
    body: &str,
) -> std::result::Result<bool, anyhow::Error> {
    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    let email = Message::builder()
        .from(smtp_username.parse().map_err(|_| anyhow::anyhow!("from email is failed."))?)
        .to(to.parse().map_err(|_| anyhow::anyhow!("receive email is failed."))?)
        .subject(title)
        .header(ContentType::TEXT_HTML)
        .body(String::from(body))?;
    let creds = Credentials::new(smtp_username.to_string(), smtp_password.to_string());
    let mailer = SmtpTransport::relay(smtp_server).unwrap().credentials(creds).build();

    match mailer.send(&email) {
        Ok(_) => Ok(true),
        Err(_e) => Err(anyhow::anyhow!("email send failed.")),
    }
}

/// 随机生成6位的数字编码
pub fn random_six_number_code() -> String {
    let mut rng = rand::rng();
    let random_number: i32 = rand::Rng::random_range(&mut rng, 100000..1000000);
    format!("{}", random_number)
}
