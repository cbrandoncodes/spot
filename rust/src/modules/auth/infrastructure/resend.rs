use std::time::Duration;

use async_trait::async_trait;
use better_auth::plugins::{EmailOtpMessage, SendEmailOtp};
use better_auth::{AuthError, AuthResult};
use reqwest::{Client, StatusCode};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;

use super::email_template::render_otp;

const RESEND_EMAILS_URL: &str = "https://api.resend.com/emails";

pub struct ResendOtpSender {
    client: Client,
    api_key: SecretString,
    from: String,
}

impl ResendOtpSender {
    pub fn new(api_key: SecretString, from_email: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            from: format!("Spot <{from_email}>"),
        }
    }
}

#[async_trait]
impl SendEmailOtp for ResendOtpSender {
    async fn send(&self, message: &EmailOtpMessage) -> AuthResult<()> {
        let otp = message.otp.typed()?;
        let email = render_otp(message.kind, otp);
        let body = SendEmailRequest {
            from: &self.from,
            to: [&message.email],
            subject: &email.subject,
            html: email.html,
            text: email.text,
        };

        let response = self
            .client
            .post(RESEND_EMAILS_URL)
            .bearer_auth(self.api_key.expose_secret())
            .timeout(Duration::from_secs(10))
            .json(&body)
            .send()
            .await
            .map_err(|error| {
                tracing::error!(error = %error, "Resend OTP request failed");
                delivery_error()
            })?;

        if !response.status().is_success() {
            log_rejection(response.status());
            return Err(delivery_error());
        }

        Ok(())
    }
}

#[derive(Serialize)]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: [&'a str; 1],
    subject: &'a str,
    html: String,
    text: String,
}

fn log_rejection(status: StatusCode) {
    tracing::error!(%status, "Resend rejected OTP email");
}

fn delivery_error() -> AuthError {
    AuthError::internal("Unable to deliver authentication email")
}
