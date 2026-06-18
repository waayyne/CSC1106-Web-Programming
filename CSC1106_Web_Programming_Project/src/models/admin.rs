use serde::{Deserialize, Serialize};


#[derive(Deserialize)]
pub struct AdminUserRegisterForm { 
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)]
 pub struct AdminUserUpdateForm { 
    pub user_id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
}

#[derive(Deserialize)]
pub struct UpdateRoleForm { 
    pub user_id: i32,
    pub new_role: String,
}

#[derive(Serialize)]
pub struct UserRow { 
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
    pub role: String,
}