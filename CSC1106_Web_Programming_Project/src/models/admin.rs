use serde::{Deserialize, Serialize};

//contains data structures for admin-related forms and views

#[derive(Deserialize)]
pub struct AdminUserRegisterForm { //struct for admin user registration form data
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)]
 pub struct AdminUserUpdateForm { //struct for admin user update form data
    pub user_id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
}

#[derive(Deserialize)]
pub struct UpdateRoleForm { //struct for only updating user role form data
    pub user_id: i32,
    pub new_role: String,
}

#[derive(Serialize)]
pub struct UserRow { //struct for representing user data in admin user management page
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
    pub role: String,
}