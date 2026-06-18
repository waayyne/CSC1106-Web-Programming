use serde::Serialize;


#[derive(Serialize)]
pub struct CustomerOverview {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub account_number: String,
    pub balance: String,
}


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
