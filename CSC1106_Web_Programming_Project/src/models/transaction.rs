use serde::Deserialize;

#[derive(Deserialize)]
pub struct TransferForm {
    pub transfer_by: String,
    pub recipient_identifier: String,
    pub amount: f64,
    pub description: Option<String>,
}