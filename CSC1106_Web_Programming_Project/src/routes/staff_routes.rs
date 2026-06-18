use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use rust_decimal::Decimal;
use sqlx::Row;
use tera::{Context, Tera};

use crate::db::DbPool;
use crate::models::staff::CustomerOverview;
use crate::routes::transaction_routes::TxQuery;
use crate::services::{staff_service, transaction_service};

struct StaffHeader {
    first_name: String,
    last_name: String,
    initials: String,
    role: String,
}

fn require_staff(session: &Session) -> Option<i32> {
    let user_id = session.get::<i32>("user_id").unwrap_or(None)?;
    let role = session.get::<String>("role").unwrap_or(None)?;

    if role == "staff" {
        Some(user_id)
    } else {
        None
    }
}

async fn load_staff_header(
    pool: &DbPool,
    session: &Session,
    staff_id: i32,
) -> Result<StaffHeader, HttpResponse> {
    let staff_lookup = sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(staff_id)
        .fetch_one(pool)
        .await;

    let staff_info = match staff_lookup {
        Ok(staff_info) => staff_info,
        Err(_) => {
            return Err(HttpResponse::Found()
                .append_header(("Location", "/login"))
                .finish());
        }
    };

    let first_name: String = staff_info.get("first_name");
    let last_name: String = staff_info.get("last_name");
    let initials = format!(
        "{}{}",
        first_name.chars().next().unwrap_or('S'),
        last_name.chars().next().unwrap_or('T')
    );
    let role = session
        .get::<String>("role")
        .unwrap_or(None)
        .unwrap_or_default();

    Ok(StaffHeader {
        first_name,
        last_name,
        initials,
        role,
    })
}

fn insert_staff_header(context: &mut Context, header: &StaffHeader) {
    context.insert("first_name", &header.first_name);
    context.insert("last_name", &header.last_name);
    context.insert("initials", &header.initials);
    context.insert("role", &header.role);
}

fn render_html(tmpl: &Tera, template: &str, context: &Context) -> HttpResponse {
    match tmpl.render(template, context) {
        Ok(rendered) => HttpResponse::Ok().content_type("text/html").body(rendered),
        Err(error) => {
            HttpResponse::InternalServerError().body(format!("Template error: {}", error))
        }
    }
}

fn staff_redirect() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/admin/dashboard"))
        .finish()
}

pub async fn staff_dashboard(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return staff_redirect(),
    };

    let header = match load_staff_header(&pool, &session, staff_id).await {
        Ok(header) => header,
        Err(response) => return response,
    };

    let total_users: i64 =
        sqlx::query("SELECT COUNT(*) as count FROM users WHERE role = 'customer'")
            .fetch_one(pool.get_ref())
            .await
            .map(|row| row.get("count"))
            .unwrap_or(0);

    let pending_loans: i64 =
        sqlx::query("SELECT COUNT(*) as count FROM loans WHERE status = 'pending'")
            .fetch_one(pool.get_ref())
            .await
            .map(|row| row.get("count"))
            .unwrap_or(0);

    let mut context = Context::new();
    insert_staff_header(&mut context, &header);
    context.insert("total_users", &total_users);
    context.insert("pending_loans", &pending_loans);

    render_html(&tmpl, "staff_dashboard.html", &context)
}

pub async fn staff_table_view(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
    params: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return staff_redirect(),
    };

    let header = match load_staff_header(&pool, &session, staff_id).await {
        Ok(header) => header,
        Err(response) => return response,
    };

    let search_query = params.get("query").cloned();
    let customers_result = match search_query.as_deref() {
        Some(query) if !query.is_empty() => staff_service::get_customers_filter(&pool, query).await,
        _ => staff_service::get_all_customers(&pool).await,
    };

    let customers = match customers_result {
        Ok(customers) => customers,
        Err(error) => return HttpResponse::InternalServerError().body(error.to_string()),
    };

    let mut context = Context::new();
    insert_staff_header(&mut context, &header);
    context.insert("table_view", &customers);

    if let Some(query) = search_query {
        context.insert("query", &query);
    }

    render_html(&tmpl, "staff_table_view.html", &context)
}

pub async fn staff_transaction_users_page(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return staff_redirect(),
    };

    let header = match load_staff_header(&pool, &session, staff_id).await {
        Ok(header) => header,
        Err(response) => return response,
    };

    let records = match sqlx::query(
        r#"
        SELECT u.id, u.first_name, u.last_name, u.email, b.account_number, b.balance
        FROM users u
        JOIN bank_accounts b ON u.id = b.user_id
        WHERE u.role = 'customer'
        ORDER BY u.last_name ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await
    {
        Ok(rows) => rows,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    let customers: Vec<CustomerOverview> = records
        .into_iter()
        .map(|row| {
            let first: String = row.get("first_name");
            let last: String = row.get("last_name");
            let balance: Decimal = row.get("balance");

            CustomerOverview {
                id: row.get("id"),
                name: format!("{} {}", first, last),
                email: row.get("email"),
                account_number: row.get("account_number"),
                balance: format!("{:.2}", balance),
            }
        })
        .collect();

    let mut context = Context::new();
    insert_staff_header(&mut context, &header);
    context.insert("customers", &customers);

    render_html(&tmpl, "staff_transaction_users.html", &context)
}

pub async fn staff_view_customer_transactions(
    pool: web::Data<DbPool>,
    tmpl: web::Data<Tera>,
    session: Session,
    path: web::Path<i32>,
    query: web::Query<TxQuery>,
) -> impl Responder {
    let staff_id = match require_staff(&session) {
        Some(id) => id,
        None => return staff_redirect(),
    };

    let header = match load_staff_header(&pool, &session, staff_id).await {
        Ok(header) => header,
        Err(response) => return response,
    };

    let target_user_id = path.into_inner();
    let page = query.page.unwrap_or(1).max(1) as i64;
    let per_page = query.per_page.unwrap_or(10).max(1) as i64;

    let (transactions, total_count) = transaction_service::fetch_transactions(
        &pool,
        target_user_id,
        page,
        per_page,
        query.start_date.clone(),
        query.end_date.clone(),
        query.tx_type.clone(),
        query.q.clone(),
    )
    .await
    .unwrap_or((Vec::new(), 0));

    let cash_flow = match transaction_service::get_cash_flow_summary(&pool, target_user_id).await {
        Ok(summary) => summary,
        Err(error) => return HttpResponse::InternalServerError().body(error),
    };

    let target_user = match sqlx::query("SELECT first_name, last_name FROM users WHERE id = $1")
        .bind(target_user_id)
        .fetch_one(pool.get_ref())
        .await
    {
        Ok(row) => row,
        Err(_) => return HttpResponse::NotFound().body("Customer not found"),
    };

    let target_first: String = target_user.get("first_name");
    let target_last: String = target_user.get("last_name");
    let total_pages = if total_count == 0 {
        1
    } else {
        (total_count + per_page - 1) / per_page
    };

    let mut context = Context::new();
    insert_staff_header(&mut context, &header);
    context.insert("target_user_id", &target_user_id);
    context.insert(
        "customer_name",
        &format!("{} {}", target_first, target_last),
    );
    context.insert("transactions", &transactions);
    context.insert("page", &page);
    context.insert("total_pages", &total_pages);
    context.insert("query", &query.into_inner());
    context.insert("net_flow", &format!("{:.2}", cash_flow.net_flow));
    context.insert("total_in", &format!("{:.2}", cash_flow.total_in));
    context.insert("total_out", &format!("{:.2}", cash_flow.total_out));

    render_html(&tmpl, "staff_transaction_history.html", &context)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route("/staff/dashboard", web::get().to(staff_dashboard))
        .route("/staff/table_view", web::get().to(staff_table_view))
        .route(
            "/staff/transactions",
            web::get().to(staff_transaction_users_page),
        )
        .route(
            "/staff/transactions/{id}",
            web::get().to(staff_view_customer_transactions),
        );
}
