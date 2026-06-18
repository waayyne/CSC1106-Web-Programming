use actix_session::Session; 
use actix_web::{web, HttpResponse, Responder}; 
use tera::{Context, Tera}; 
use sqlx::Row; 
use crate::db::DbPool; 
use crate::models::audit_log::AuditLog;
use crate::services::{admin_service, audit_service, auth_service}; 
use crate::models::admin::{AdminUserRegisterForm, UpdateRoleForm, UserRow, AdminUserUpdateForm}; 
use crate::services::audit_service::AuditLogView;


fn require_admin(session: &Session) -> Option<i32> {
    let user_id = session.get::<i32>("user_id").unwrap_or(None)?;
    let role = session.get::<String>("role").unwrap_or(None)?;
    if role == "admin" { Some(user_id) } else { None }
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/admin/dashboard", web::get().to(admin_dashboard))
       .route("/admin/users", web::get().to(admin_dashboard))
       .route("/admin/users/update-role", web::post().to(update_role))
       .route("/admin/users/update", web::post().to(update_user))
       .route("/admin/register", web::post().to(admin_register_user))
       .route("/admin/users/delete/{id}", web::post().to(delete_user_handler))
       .route("/admin/logs", web::get().to(audit_logs_page));
}


async fn load_admin_context(pool: &DbPool, admin_id: i32) -> Result<(Context, Vec<UserRow>), String> {
    
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
        first_name.chars().next().unwrap_or('A'), 
        last_name.chars().next().unwrap_or('D'), 
    );
    
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

    
    let total_users = user_rows.len();
    let total_staff = user_rows.iter().filter(|u| u.role == "staff").count();
    let total_customers = user_rows.iter().filter(|u| u.role == "customer").count();

    
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


pub async fn admin_dashboard(pool: web::Data<DbPool>, tmpl: web::Data<Tera>, session: Session, 
    query: web::Query<std::collections::HashMap<String, String>>) -> impl Responder {
    
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    
    let (mut context, _) = match load_admin_context(&pool, admin_id).await {
        Ok(result) => result,
        Err(e) => return HttpResponse::InternalServerError().body(e),
    };

    
    if query.get("registered").map(|v| v == "1").unwrap_or(false) {
        context.insert("message", "User registered successfully.");
    }

    if let Some(error) = query.get("error") {
        context.insert("error", &error.replace('+', " ")); 
    }

    let rendered = tmpl.render("admin_dashboard.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}


pub async fn update_role(pool: web::Data<DbPool>, tmpl: web::Data<Tera>, session: Session, form: web::Form<UpdateRoleForm>) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    let form_data = form.into_inner();
    let result = admin_service::update_user_role(&pool, form_data.user_id, &form_data.new_role).await;

    match result {
        Ok(_) => {
            let action = format!("Admin {} changed user {} role to {}", admin_id, form_data.user_id, form_data.new_role);
            let _ = audit_service::log_action(&pool, Some(admin_id), &action).await;
            HttpResponse::Found().append_header(("Location", "/admin/dashboard")).finish()
        }
        Err(e) => {
            
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




pub async fn admin_register_user(pool: web::Data<DbPool>, session: Session, form: web::Form<AdminUserRegisterForm>) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish(),
    };

    let form_data = form.into_inner();

    
    if form_data.role != "customer" && form_data.role != "staff" {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/dashboard?error=Invalid+role+selected"))
            .finish();
    }

    if let Err(message) = auth_service::validate_password_complexity(&form_data.password) {
        let encoded = message.replace(' ', "+");
        return HttpResponse::Found()
            .append_header(("Location", format!("/admin/dashboard?error={}", encoded)))
            .finish();
    }

    
    let password_hash = match auth_service::hash_password(&form_data.password) {
        Ok(h) => h,
        Err(_) => {
            return HttpResponse::Found()
                .append_header(("Location", "/admin/dashboard?error=Failed+to+hash+password"))
                .finish();
        }
    };

    
    match admin_service::register_new_user(&pool, &form_data, &password_hash).await {
        Ok(_) => {
            let action = format!("Admin {} registered user {}", admin_id, form_data.username);
            let _ = audit_service::log_action(&pool, Some(admin_id), &action).await;
            
            HttpResponse::Found()
                .append_header(("Location", "/admin/dashboard?registered=1"))
                .finish()
        }
        Err(e) => {
            
            let encoded = e.replace(' ', "+");
            HttpResponse::Found()
                .append_header(("Location", format!("/admin/dashboard?error={}", encoded)))
                .finish()
        }
    }
}


pub async fn update_user(pool: web::Data<DbPool>, tmpl: web::Data<Tera>, session: Session, form: web::Form<AdminUserUpdateForm>) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    let form_data = form.into_inner();
    let result = admin_service::update_user_details(&pool, &form_data).await;

    match result {
        Ok(_) => {
            let action = format!("Admin {} updated user {} profile", admin_id, form_data.user_id);
            let _ = audit_service::log_action(&pool, Some(admin_id), &action).await;
            HttpResponse::Found().append_header(("Location", "/admin/dashboard")).finish()
        }
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


pub async fn delete_user_handler(pool: web::Data<DbPool>, session: Session, path: web::Path<i32>) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    let user_id = path.into_inner();
    if user_id == admin_id {
        return HttpResponse::Found()
            .append_header(("Location", "/admin/dashboard?error=You+cannot+delete+your+own+admin+account"))
            .finish();
    }

    if let Err(e) = admin_service::delete_user(&pool, user_id).await {
        let encoded = e.replace(' ', "+");
        return HttpResponse::Found()
            .append_header(("Location", format!("/admin/dashboard?error={}", encoded)))
            .finish();
    }

    let _ = audit_service::log_action(&pool, Some(admin_id), &format!("Admin deleted user {}", user_id)).await;

    HttpResponse::Found().append_header(("Location", "/admin/dashboard")).finish()
}

pub async fn audit_logs_page(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let admin_id = match require_admin(&session) {
        Some(id) => id,
        None => return HttpResponse::Found().append_header(("Location", "/dashboard")).finish(),
    };

    let admin_row = sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(admin_id)
        .fetch_one(pool.get_ref())
        .await;

    let (first_name, last_name) = match admin_row {
        Ok(row) => (
            row.get::<String, _>("first_name"),
            row.get::<String, _>("last_name"),
        ),
        Err(_) => ("Admin".to_string(), "".to_string()),
    };

    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('A'),
        last_name.chars().next().unwrap_or('D')
    );

    let rows = sqlx::query(
        "SELECT al.id, al.user_id,
                COALESCE(u.username, 'System') AS username,
                al.action,
                al.created_at
         FROM audit_logs al
         LEFT JOIN users u ON al.user_id = u.id
         ORDER BY al.created_at DESC
         LIMIT 300",
    )
    .fetch_all(pool.get_ref())
    .await
    .unwrap_or_default();

    let logs: Vec<AuditLogView> = rows
        .into_iter()
        .map(|row| AuditLogView {
            id: {
                let audit_log = AuditLog {
                    id: row.get("id"),
                    user_id: row.get("user_id"),
                    action: row.get("action"),
                    created_at: row.get("created_at"),
                };
                audit_log.id
            },
            user_id: row.get("user_id"),
            username: row.get("username"),
            action: row.get("action"),
            created_at: row
                .get::<chrono::NaiveDateTime, _>("created_at")
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        })
        .collect();

    let mut ctx = Context::new();
    ctx.insert("first_name", &first_name);
    ctx.insert("last_name", &last_name);
    ctx.insert("initials", &initials);
    ctx.insert("logs", &logs);

    let rendered = tmpl.render("audit_logs.html", &ctx).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}
