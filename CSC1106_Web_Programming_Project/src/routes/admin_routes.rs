use core::error;

use actix_session::Session; // Handles session cookies for authentication
use actix_web::{web, HttpResponse, Responder}; // Framework core for web handling
use tera::{Context, Tera}; // Templating engine for HTML rendering
use sqlx::Row; // Helper for extracting data from SQL query rows
use crate::db::DbPool; // Database connection pool type
use crate::services::{admin_service, audit_service, auth_service}; // Custom business logic services
use crate::models::admin::{AdminUserRegisterForm, UpdateRoleForm, UserRow, AdminUserUpdateForm}; // Shared data models

// Helper function to verify if the session user has "admin" privileges
fn require_admin(session: &Session) -> Option<i32> {
    let user_id = session.get::<i32>("user_id").unwrap_or(None)?;
    let role = session.get::<String>("role").unwrap_or(None)?;
    if role == "admin" { Some(user_id) } else { None }
}

// Router configuration mapping URLs to handler functions
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/admin/dashboard", web::get().to(admin_dashboard))
       .route("/admin/users", web::get().to(admin_dashboard))
       .route("/admin/users/update-role", web::post().to(update_role))
       .route("/admin/users/update", web::post().to(update_user))
       .route("/admin/register", web::get().to(admin_register_page))
       .route("/admin/register", web::post().to(admin_register_user))
       .route("/admin/users/delete/{id}", web::post().to(delete_user_handler));
}

// Helper function to prepare dashboard stats and user lists for the template
async fn load_admin_context(pool: &DbPool, admin_id: i32) -> Result<(Context, Vec<UserRow>), String> {
    // Retrieve admin details for the UI
    let admin_info = sqlx::query(
        "SELECT first_name, last_name FROM users WHERE id = $1"
    )
    .bind(admin_id)
    .fetch_one(pool)
    .await
    .map_err(|_| "Failed to load admin info.".to_string())?;

    let first_name: String = admin_info.get("first_name");
    let last_name: String = admin_info.get("last_name");
    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('A'), // placeholder 'A' if first name is empty
        last_name.chars().next().unwrap_or('D'), // placeholder 'D' if last name is empty
    );
    // Get all users and filter out the current admin
    let users = admin_service::get_all_users(pool).await?;
    let user_rows: Vec<UserRow> = users.into_iter()
        .filter(|u| u.id != admin_id)
        .map(|u| UserRow {
            id: u.id,
            username: u.username,
            first_name: u.first_name,
            last_name: u.last_name,
            email: u.email,
            phone_number: u.phone_number,
            role: u.role,
        })
        .collect();

    // Calculate dashboard statistics
    let total_users = user_rows.len();
    let total_staff = user_rows.iter().filter(|u| u.role == "staff").count();
    let total_customers = user_rows.iter().filter(|u| u.role == "customer").count();

    // Populate Tera template context
    let mut context = Context::new();
    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("total_users", &total_users);
    context.insert("total_staff", &total_staff);
    context.insert("total_customers", &total_customers);
    context.insert("users", &user_rows);

    Ok((context, user_rows))
}

// Main admin landing page handler
pub async fn admin_dashboard(pool: web::Data<DbPool>, tmpl: web::Data<Tera>, session: Session, 
    query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    // Authentication guard
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    // Load dashboard context and render
    let (mut context, _) = match load_admin_context(&pool, admin_id).await {
        Ok(result) => result,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    // Check for query parameters to display messages (e.g., after registration)
    if query.get("registered").map(|v| v == "1").unwrap_or(false) {
        context.insert("message", "User registered successfully.");
    }

    if let Some(error) = query.get("error") {
        context.insert("error", &error.replace('+', " ")); // Decode error message from query param
    }

    let rendered = tmpl.render("admin_dashboard.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

// Handles POST requests to change a user's role
pub async fn update_role(pool: web::Data<DbPool>, tmpl: web::Data<Tera>, session: Session, form: web::Form<UpdateRoleForm>) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    let form_data = form.into_inner();
    let result = admin_service::update_user_role(&pool, form_data.user_id, &form_data.new_role).await;

    // Log the event for audit purposes
    let action = format!("Admin {} changed user {} role to {}", admin_id, form_data.user_id, form_data.new_role);
    let _ = audit_service::log_action(&pool, Some(admin_id), &action).await;

    match result {
        Ok(_) => HttpResponse::Found().append_header(("Location", "/admin/dashboard")).finish(),
        Err(e) => {
            // Re-render with error if update fails
            let (mut context, _) = match load_admin_context(&pool, admin_id).await {
                Ok(result) => result,
                Err(err) => return HttpResponse::InternalServerError().body(err),
            };
            context.insert("error", &e);
            let rendered = tmpl.render("admin_dashboard.html", &context).unwrap();
            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
    }
}

// Serves the registration page for Admin to create new users (staff or customers)
pub async fn admin_register_page(tmpl: web::Data<Tera>, session: Session) -> impl Responder {
    if require_admin(&session).is_none() {
        return HttpResponse::Found().append_header(("Location", "/dashboard")).finish();
    }
    let context = Context::new();
    let rendered = tmpl.render("admin_register_user.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)

}

// Handles user registration form submission
pub async fn admin_register_user(pool: web::Data<DbPool>, session: Session, form: web::Form<AdminUserRegisterForm>) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish(),
    };

    let form_data = form.into_inner();

    // Validate role inputs
    if form_data.role != "customer" && form_data.role != "staff" {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/dashboard?error=Invalid+role+selected"))
            .finish();
    }

    // Securely hash the password
    let password_hash = match auth_service::hash_password(&form_data.password) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("Location", "/admin/dashboard?error=Failed+to+hash+password"))
                .finish();
        }
    };

    // Execute service-level registration
    match admin_service::register_new_user(&pool, &form_data, &password_hash).await {
        Ok(_) => {
            let action = format!("Admin {} registered user {}", admin_id, form_data.username);
            let _ = audit_service::log_action(&pool, Some(admin_id), &action).await;
            // Redirect back to dashboard with success flag, modal closes naturally
            HttpResponse::Found()
                .append_header(("Location", "/admin/dashboard?registered=1"))
                .finish()
        }
        Err(e) => {
            // Redirect back to dashboard with error message as query param
            let encoded = e.replace(' ', "+");
            HttpResponse::Found()
                .append_header(("Location", format!("/admin/dashboard?error={}", encoded)))
                .finish()
        }
    }
}

// Handles profile updates (e.g., name, email) for an existing user
pub async fn update_user(pool: web::Data<DbPool>, tmpl: web::Data<Tera>, session: Session, form: web::Form<AdminUserUpdateForm>) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    let form_data = form.into_inner();
    let result = admin_service::update_user_details(&pool, &form_data).await;

    // Log update action
    let action = format!("Admin {} updated user {} profile", admin_id, form_data.user_id);
    let _ = audit_service::log_action(&pool, Some(admin_id), &action).await;

    match result {
        Ok(_) => HttpResponse::Found().append_header(("Location", "/admin/dashboard")).finish(),
        Err(e) => {
            let (mut context, _) = match load_admin_context(&pool, admin_id).await {
                Ok(result) => result,
                Err(err) => return HttpResponse::InternalServerError().body(err),
            };
            context.insert("error", &format!("Update failed: {}", e));
            let rendered = tmpl.render("admin_dashboard.html", &context).unwrap();
            HttpResponse::Ok().content_type("text/html").body(rendered)
        }
    }
}

// Deletes a user record by their unique ID
pub async fn delete_user_handler(pool: web::Data<DbPool>, session: Session, path: web::Path<i32>) -> impl Responder {
    if require_admin(&session).is_none() {
        return HttpResponse::Found().append_header(("Location", "/dashboard")).finish();
    }

    let user_id = path.into_inner();
    let _ = admin_service::delete_user(&pool, user_id).await;
    let _ = audit_service::log_action(&pool, None, &format!("Admin deleted user {}", user_id)).await;

    HttpResponse::Found().append_header(("Location", "/admin/dashboard")).finish()
}