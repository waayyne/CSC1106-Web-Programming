use sqlx::{Row};
use crate::db::DbPool;
use crate::models::admin::{AdminUserRegisterForm, AdminUserUpdateForm};


/// Struct to represent user information for service-level operations
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone_number: String,
    pub role: String,
}

// Fetch all users from the database for display as well as displays them
pub async fn get_all_users(pool: &DbPool) -> Result<Vec<UserInfo>, String> {
    let rows = sqlx::query(
        "SELECT id, username, first_name, last_name, email, phone_number, role 
         FROM users 
         ORDER BY id ASC"
    )
    .fetch_all(pool).await
    .map_err(|e| format!("Failed to fetch users: {}", e))?;

    let users = rows.into_iter().map(|row| UserInfo {
        id: row.get("id"),
        username: row.get("username"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        email: row.get("email"),
        phone_number: row.get("phone_number"),
        role: row.get("role"),
    }).collect();

    Ok(users)
}

// Update a user's role
pub async fn update_user_role(pool: &DbPool, target_user_id: i32, new_role: &str) -> Result<(), String> {
    if new_role != "staff" && new_role != "customer" {
        return Err("Invalid role. Must be 'staff' or 'customer'.".to_string());
    }

    sqlx::query("UPDATE users SET role = $1 WHERE id = $2")
        .bind(new_role)
        .bind(target_user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to update role: {}", e))?;

    Ok(())
}

// Register a new user and create a bank account for them (used by admin registration can decide if staff or customer)
pub async fn register_new_user(
    pool: &DbPool,
    form: &AdminUserRegisterForm,
    password_hash: &str,
) -> Result<i32, String> {

    // Checks for existing username, email, or phone number first
    let existing = sqlx::query(
        "SELECT 
            (SELECT COUNT(*) FROM users WHERE username = $1) as username_count,
            (SELECT COUNT(*) FROM users WHERE email = $2) as email_count,
            (SELECT COUNT(*) FROM users WHERE phone_number = $3) as phone_count"
    )
    .bind(&form.username)
    .bind(&form.email)
    .bind(&form.phone_number)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Validation check failed: {}", e))?;

    // Gets counts of existing username, email, and phone number to check for duplicates
    let username_count: i64 = existing.get("username_count"); 
    let email_count: i64 = existing.get("email_count");
    let phone_count: i64 = existing.get("phone_count");

    // Checks if the username, email or phone no. exists and returns an error.
    if username_count > 0 {
        return Err("Username already exists.".to_string());
    }
    if email_count > 0 {
        return Err("Email already in use.".to_string());
    }
    if phone_count > 0 {
        return Err("Phone number already in use.".to_string());
    }

    let full_name = format!("{} {}", form.first_name.trim(), form.last_name.trim());

    let row = sqlx::query(
        "INSERT INTO users (username, first_name, last_name, name, email, password_hash, phone_number, role)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id"
    )
    .bind(&form.username)
    .bind(&form.first_name)
    .bind(&form.last_name)
    .bind(&full_name)
    .bind(&form.email)
    .bind(password_hash)
    .bind(&form.phone_number)
    .bind(&form.role)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to register user: {}", e))?;

    let user_id: i32 = row.get("id");

    let account_number = format!("RB{}", chrono::Utc::now().timestamp_millis());
    sqlx::query(
        "INSERT INTO bank_accounts (user_id, account_number, balance) VALUES ($1, $2, $3)"
    )
    .bind(user_id)
    .bind(account_number)
    .bind(rust_decimal::Decimal::new(0, 0))
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to create bank account: {}", e))?;

    Ok(user_id)
}

// Delete a user by ID (used when admin deletes a user)
pub async fn delete_user(pool: &DbPool, user_id: i32) -> Result<(), String> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|e| format!("Failed to cleanup orphaned user: {}", e))?;
    Ok(())
}


pub async fn update_user_details(pool: &DbPool, form: &AdminUserUpdateForm) -> Result<(), String> {
    sqlx::query(
        "UPDATE users 
         SET username = $1, first_name = $2, last_name = $3, email = $4, phone_number = $5
         WHERE id = $6"
    )
    .bind(&form.username)
    .bind(&form.first_name)
    .bind(&form.last_name)
    .bind(&form.email)
    .bind(&form.phone_number)
    .bind(form.user_id)
    .execute(pool)
    .await
    .map_err(|e| format!("Failed to update user details: {}", e))?;

    Ok(())
}
    