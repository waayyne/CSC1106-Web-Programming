use serde::Deserialize;

#[derive(Deserialize)]
pub struct AtmForm {
    pub find_by: String,
    pub account_identifier: String,
    pub transaction_type: String,
    pub amount: f64,
}