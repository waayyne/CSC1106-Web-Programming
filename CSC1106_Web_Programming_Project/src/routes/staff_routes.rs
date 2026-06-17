use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use sqlx::Row;
use tera::{Context, Tera};
use rust_decimal::Decimal;

use crate::db::DbPool;
use crate::models::staff;
use crate::models::staff::CustInfo;
use crate::models::staff::CustomerOverview;
use crate::services::staff_service;
use crate::services::transaction_service;
use crate::routes::transaction_routes::TxQuery;

// Helper function to check if the current user is staff (or admin) and return their user ID
fn require_staff(session: &Session) -> Option<i32> {
    let user_id = session.get::<i32>("user_id").unwrap_or(None)?;
    let role = session.get::<String>("role").unwrap_or(None)?;
    if role == "staff" { Some(user_id) } else { None }
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
            .append_header(("Location", "/admin/dashboard"))
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
    let role = session.get::<String>("role").unwrap_or(None).unwrap_or_default();

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
    context.insert("role", &role);
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
            .append_header(("Location", "/admin/dashboard"))
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
    let role = session.get::<String>("role").unwrap_or(None).unwrap_or_default();

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
    context.insert("role", &role);
    context.insert("table_view", &customers);

    if let Some(q) = search_query { // If statement for search query, so that the search box can retain the search term after searching
        context.insert("query", &q);
    }

    let rendered = tmpl.render("staff_table_view.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}


// Handler for the list of customers to select from
pub async fn staff_transaction_users_page(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return HttpResponse::Found()
            .append_header(("Location", "/admin/dashboard"))
            .finish(),
    };

    // get staff's own info for topbar
    let staff_info = sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(staff_id)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();

    let first_name: String = staff_info.get("first_name");
    let last_name: String = staff_info.get("last_name");
    let initials = format!("{}{}", first_name.chars().next().unwrap_or('S'), last_name.chars().next().unwrap_or('T'));
    let role = session.get::<String>("role").unwrap_or(None).unwrap_or_default();

    // Fetch all customers and their primary bank accounts
    let records = match sqlx::query(
        r#"
        SELECT u.id, u.first_name, u.last_name, u.email, b.account_number, b.balance 
        FROM users u 
        JOIN bank_accounts b ON u.id = b.user_id 
        WHERE u.role = 'customer'
        ORDER BY u.last_name ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await {
        Ok(rows) => rows,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    let mut customers = Vec::new();
    for r in records {
        let first: String = r.get("first_name");
        let last: String = r.get("last_name");
        let balance: Decimal = r.get("balance");
        customers.push(CustomerOverview {
            id: r.get("id"),
            name: format!("{} {}", first, last),
            email: r.get("email"),
            account_number: r.get("account_number"),
            balance: format!("{:.2}", balance),
        });
    }

    let mut context = Context::new();
    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("role", &role);
    context.insert("customers", &customers);

    let rendered = tmpl.render("staff_transaction_users.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

// Handler for viewing a specific customer's transactions
pub async fn staff_view_customer_transactions(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
    path: web::Path<i32>,
    query: web::Query<TxQuery>,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return HttpResponse::Found()
            .append_header(("Location", "/admin/dashboard"))
            .finish(),
    };

    // get staff's own info for topbar
    let staff_info = sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(staff_id).fetch_one(pool.get_ref()).await.unwrap();
        
    let first_name: String = staff_info.get("first_name");
    let last_name: String = staff_info.get("last_name");
    let initials = format!("{}{}", first_name.chars().next().unwrap_or('S'), last_name.chars().next().unwrap_or('T'));
    let role = session.get::<String>("role").unwrap_or(None).unwrap_or_default();

    let target_user_id = path.into_inner();
    let page = query.page.unwrap_or(1).max(1) as i64;
    let per_page = query.per_page.unwrap_or(10).max(1) as i64;

    // Fetch the target user's transactions using the existing service
    let (transactions, total_count) = transaction_service::fetch_transactions(
        &pool, target_user_id, page, per_page, query.start_date.clone(),
        query.end_date.clone(), query.tx_type.clone(), query.q.clone(),
    ).await.unwrap_or((Vec::new(), 0));

    let cash_flow = transaction_service::get_cash_flow_summary(&pool, target_user_id)
        .await.unwrap();

    let target_user = sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(target_user_id).fetch_one(pool.get_ref()).await.unwrap();
    let target_first: String = target_user.get("first_name");
    let target_last: String = target_user.get("last_name");

    let total_pages = if total_count == 0 { 1 } else { (total_count + per_page - 1) / per_page };

    let mut context = Context::new();
    context.insert("first_name", &first_name);
    context.insert("last_name", &last_name);
    context.insert("initials", &initials);
    context.insert("role", &role);

    context.insert("target_user_id", &target_user_id);
    context.insert("customer_name", &format!("{} {}", target_first, target_last));
    context.insert("transactions", &transactions);
    context.insert("page", &page);
    context.insert("total_pages", &total_pages);
    context.insert("query", &query.into_inner());
    context.insert("net_flow", &format!("{:.2}", cash_flow.net_flow));
    context.insert("total_in", &format!("{:.2}", cash_flow.total_in));
    context.insert("total_out", &format!("{:.2}", cash_flow.total_out));

    let rendered = tmpl.render("staff_transaction_history.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
}


// Configuration block linking URLs to your handlers
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/staff/dashboard", web::get().to(staff_dashboard))
       .route("/staff/table_view", web::get().to(staff_table_view))
       .route("/staff/transactions", web::get().to(staff_transaction_users_page))
       .route("/staff/transactions/{id}", web::get().to(staff_view_customer_transactions));
}