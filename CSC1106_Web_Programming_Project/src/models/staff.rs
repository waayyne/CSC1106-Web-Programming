use serde::Serialize;
use rust_decimal::Decimal;

// Struct to represent user information for service-level operations
#[derive(Serialize)]
pub struct CustInfo {
    pub id: i32,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub balance: String,
}
