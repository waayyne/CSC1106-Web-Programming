use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterForm {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
    pub confirm_password: String,
    #[serde(rename = "cf-turnstile-response", default)]
    pub turnstile_response: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub identifier: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ForgotPasswordForm {
    pub email: String,
    #[serde(rename = "cf-turnstile-response", default)]
    pub turnstile_response: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordQuery {
    pub token: String,
}

#[derive(Deserialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub password: String,
    pub confirm_password: String,
}

#[derive(Deserialize)]
pub struct VerifyEmailQuery {
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct VerifyEmailForm {
    pub email: String,
    pub otp: String,
    #[serde(rename = "cf-turnstile-response", default)]
    pub turnstile_response: String,
}

#[derive(Deserialize)]
pub struct ResendVerificationOtpForm {
    pub email: String,
    #[serde(rename = "cf-turnstile-response", default)]
    pub turnstile_response: String,
}
