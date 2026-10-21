use better_auth::plugins::EmailOtpType;

const HTML_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/email/auth_otp.html"
));
const TEXT_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/email/auth_otp.txt"
));
const SPOT_LOGO_URL: &str = "https://ik.imagekit.io/to7rbsdcz/spot.png";

pub struct RenderedEmail {
    pub subject: String,
    pub html: String,
    pub text: String,
}

pub fn render_otp(kind: EmailOtpType, otp: &str) -> RenderedEmail {
    let content = content(kind);
    let subject = format!("{} · Spot", content.subject);
    let html = HTML_TEMPLATE
        .replace("{{ preheader }}", &escape_html(content.preheader))
        .replace("{{ eyebrow }}", &escape_html(content.eyebrow))
        .replace("{{ heading }}", &escape_html(content.heading))
        .replace("{{ instruction }}", &escape_html(content.instruction))
        .replace("{{ logo_url }}", SPOT_LOGO_URL)
        .replace("{{ code }}", &escape_html(otp));
    let text = TEXT_TEMPLATE
        .replace("{{ heading }}", content.heading)
        .replace("{{ instruction }}", content.instruction)
        .replace("{{ code }}", otp);

    RenderedEmail {
        subject,
        html,
        text,
    }
}

struct EmailContent {
    subject: &'static str,
    preheader: &'static str,
    eyebrow: &'static str,
    heading: &'static str,
    instruction: &'static str,
}

fn content(kind: EmailOtpType) -> EmailContent {
    match kind {
        EmailOtpType::SignIn => EmailContent {
            subject: "Your sign-in code",
            preheader: "Use this one-time code to get back to your training.",
            eyebrow: "Secure sign in",
            heading: "Let’s get you back to training.",
            instruction: "Enter this one-time code in Spot to sign in.",
        },
        EmailOtpType::EmailVerification => EmailContent {
            subject: "Verify your email",
            preheader: "One quick check and your Spot email is verified.",
            eyebrow: "Email verification",
            heading: "One quick check.",
            instruction: "Enter this code in Spot to verify your email address.",
        },
        EmailOtpType::ForgetPassword => EmailContent {
            subject: "Reset your password",
            preheader: "Use this one-time code to reset your Spot password.",
            eyebrow: "Password reset",
            heading: "Reset, then keep moving.",
            instruction: "Enter this code in Spot to choose a new password.",
        },
        EmailOtpType::ChangeEmail => EmailContent {
            subject: "Confirm your new email",
            preheader: "Use this one-time code to confirm your new Spot email.",
            eyebrow: "Email change",
            heading: "Confirm the change.",
            instruction: "Enter this code in Spot to confirm your new email address.",
        },
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use better_auth::plugins::EmailOtpType;

    use super::render_otp;

    #[test]
    fn renders_branded_html_and_plain_text() {
        let email = render_otp(EmailOtpType::SignIn, "123456");

        assert_eq!(email.subject, "Your sign-in code · Spot");
        assert!(email.html.contains("Let’s get you back to training."));
        assert!(email.html.contains("123456"));
        assert!(email.text.contains("123456"));
        assert!(email.html.contains("#063DB2"));
        assert!(email.html.contains("Never train lost."));
        assert!(
            email
                .html
                .contains("https://ik.imagekit.io/to7rbsdcz/spot.png")
        );
        assert!(email.text.contains("Never train lost."));
        assert!(!email.html.contains("#d9ff57"));
        assert!(!email.html.contains("{{"));
        assert!(!email.text.contains("{{"));
    }

    #[test]
    fn escapes_dynamic_html_values() {
        let email = render_otp(EmailOtpType::EmailVerification, "<123&456>");

        assert!(email.html.contains("&lt;123&amp;456&gt;"));
        assert!(!email.html.contains("<123&456>"));
        assert!(email.text.contains("<123&456>"));
    }
}
