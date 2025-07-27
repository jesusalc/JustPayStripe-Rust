use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CheckoutSession {
    pub customer: Option<String>,
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub mode: Option<String>,
    pub line_items: Option<Vec<LineItem>>,
    pub url: Option<String>, // for response
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LineItem {
    pub price: Option<String>,
    pub quantity: Option<u32>,
}
