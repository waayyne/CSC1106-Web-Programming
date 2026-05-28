use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use sqlx::Row;
use tera::{Context, Tera};

use crate::db::DbPool;

// Helper function to check if the current user is staff (or admin) and return their user ID
fn require_staff(session: &Session) -> Option<i32> {
    let user_id = session.get::<i32>("user_id").unwrap_or(None)?;
    let role = session.get::<String>("role").unwrap_or(None)?;
    if role == "staff" || role == "admin" { Some(user_id) } else { None }
}

// Handler for the staff dashboard page
pub async fn staff_dashboard(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish(),
    };

    // get staff's own info for topbar
    let staff_info = match sqlx::query(
        "SELECT first_name, last_name FROM users WHERE id = $1"
    )
    .bind(staff_id)
    .fetch_one(pool.get_ref())
    .await {
        Ok(row) => row,
        Err(_) => return HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish(),
    };

    let first_name: String = staff_info.get("first_name");
    let last_name: String = staff_info.get("last_name");
    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('S'),
        last_name.chars().next().unwrap_or('T'),
    );

    // count assigned customers, also only shows customers, not staff or admins
    let total_users: i64 = sqlx::query(
        "SELECT COUNT(*) as count FROM users WHERE role = 'customer'"
    )
    .fetch_one(pool.get_ref())
    .await
    .map(|row| row.get("count"))
    .unwrap_or(0);

    // count pending loans
    let pending_loans: i64 = sqlx::query(
        "SELECT COUNT(*) as count FROM loans WHERE status = 'pending'"
    )
    .fetch_one(pool.get_ref())
    .await
    .map(|row| row.get("count"))
    .unwrap_or(0);

    let mut context = Context::new();
    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("total_users", &total_users);
    context.insert("pending_loans", &pending_loans);

    let rendered = tmpl.render("staff_dashboard.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/staff/dashboard", web::get().to(staff_dashboard));
}