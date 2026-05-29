use sqlx::Row;
use crate::db::DbPool;
use crate::models::staff::CustInfo;
use rust_decimal::Decimal;


pub async fn get_all_customers(pool: &DbPool) -> Result<Vec<CustInfo>, sqlx::Error> {
    let rows = sqlx::query( // Query to only fetch customers, not staff or admins
        "SELECT u.id, u.username, u.first_name, u.last_name, u.email, u.phone_number,
              COALESCE(ba.balance, 0.00) as balance
        FROM users u
        LEFT JOIN bank_accounts ba ON u.id = ba.user_id
        WHERE u.role = 'customer'
        ORDER BY u.id"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        eprintln!("Error fetching customers: {}", e);
        e
    })?;

    // Map the database rows to our CustInfo struct dont need security checks here since this is only for staff and not exposed to customers
    Ok(rows.into_iter().map(|row| CustInfo {
        id: row.get("id"),
        username: row.get("username"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        email: row.get("email"),
        phone: row.get("phone_number"),
        balance: row.get::<Decimal, _>("balance").to_string(),    
    }).collect())

}

// New function to fetch customers based on a search filter
pub async fn get_customers_filter(pool: &DbPool, filter: &str) -> Result<Vec<CustInfo>, sqlx::Error> {
    let filter_pattern = format!("%{}%", filter); // Add wildcards for partial matching
    let rows = sqlx::query(
        "SELECT u.id, u.username, u.first_name, u.last_name, u.email, u.phone_number,
              COALESCE(ba.balance, 0.00) as balance
        FROM users u
        LEFT JOIN bank_accounts ba ON u.id = ba.user_id
        WHERE u.role = 'customer' AND 
              (u.username ILIKE $1 OR -- if search by username
               u.first_name ILIKE $1 OR -- if search by first name
               u.last_name ILIKE $1 OR  -- if search by last name
               u.email ILIKE $1 OR    -- if search by email
               u.phone_number ILIKE $1) -- if search by phone number
        ORDER BY u.id" // only fetch customers, not staff or admins
    )
    .bind(&filter_pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        eprintln!("Error fetching filtered customers: {}", e);
        e
    })?;

    Ok(rows.into_iter().map(|row| CustInfo {
        id: row.get("id"),
        username: row.get("username"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        email: row.get("email"),
        phone: row.get("phone_number"),
        balance: row.get::<Decimal, _>("balance").to_string(),    
    }).collect())
}