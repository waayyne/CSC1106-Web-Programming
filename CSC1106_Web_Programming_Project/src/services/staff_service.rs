use sqlx::Row;
use crate::db::DbPool;
use crate::models::staff::CustInfo;
use rust_decimal::Decimal;


pub async fn get_all_customers(pool: &DbPool) -> Result<Vec<CustInfo>, sqlx::Error> {
    let rows = sqlx::query( 
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


pub async fn get_customers_filter(pool: &DbPool, filter: &str) -> Result<Vec<CustInfo>, sqlx::Error> {
    let filter_pattern = format!("%{}%", filter); 
    let rows = sqlx::query(
        "SELECT u.id, u.username, u.first_name, u.last_name, u.email, u.phone_number,
              COALESCE(ba.balance, 0.00) as balance
        FROM users u
        LEFT JOIN bank_accounts ba ON u.id = ba.user_id
        WHERE u.role = 'customer' AND 
              (u.username ILIKE $1 OR
               u.first_name ILIKE $1 OR
               u.last_name ILIKE $1 OR
               u.email ILIKE $1 OR
               u.phone_number ILIKE $1)
        ORDER BY u.id" 
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
