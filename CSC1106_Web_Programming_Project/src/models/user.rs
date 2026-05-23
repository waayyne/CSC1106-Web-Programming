use serde::Deserialize;

#[derive(Deserialize)]
pub struct RegisterForm {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub identifier: String,
    pub password: String,
}