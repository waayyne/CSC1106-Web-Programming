use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use sqlx::Row;
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::models::staff;
use crate::services::staff_service;


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
            .append_header(("Location", "/login")) // Redirect to login if can't fetch staff info, which means the session is invalid
            .finish(),
    };

    // Displays staff's initials in the top right corner
    let first_name: String = staff_info.get("first_name");
    let last_name: String = staff_info.get("last_name");
    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('S'),
        last_name.chars().next().unwrap_or('T'),
    );

    // count assigned customers, also only shows customers
    let total_users: i64 = sqlx::query(
        "SELECT COUNT(*) as count FROM users WHERE role = 'customer'"
    )
    .fetch_one(pool.get_ref())
    .await
    .map(|row| row.get("count"))
    .unwrap_or(0);

    // count pending loans that need staff approval
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


// Handler for the staff table view page, which lists all customers and their details
pub async fn staff_table_view(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
    params: web::Query<std::collections::HashMap<String, String>>, // for search filter, e.g. ?search=john
) -> impl Responder {

    let search_query = params.get("query").cloned(); // Get the search query from the URL parameters, if any

    // Check if the id is staff if not redirect to dashboard
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return HttpResponse::Found()
            .append_header(("Location", "/dashboard"))
            .finish(),
    };

    // get staff's own info for topbar, this is beacuse the viewing table is going to be in another page.
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

    let customers_result = match search_query.as_ref(){
        Some(q) if !q.is_empty() => {
            staff_service::get_customers_filter(&pool, &q).await // If there's a search query, use the filtered function
    },
        _ => {staff_service::get_all_customers(&pool).await // Otherwise, get all customers
        }
    };

    let customers = match customers_result {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    let mut context = Context::new();
    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("table_view", &customers);

    if let Some(q) = search_query { // If statement for search query, so that the search box can retain the search term after searching
        context.insert("query", &q);
    }

    let rendered = tmpl.render("staff_table_view.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/staff/dashboard", web::get().to(staff_dashboard))
        .route("/staff/table_view", web::get().to(staff_table_view));
}