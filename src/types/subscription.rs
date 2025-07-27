use serde::{Deserialize,
    // Serialize
};

#[derive(Debug, Deserialize)]
pub struct StripeSubscription {
    pub id: String,
    pub status: String,
    pub current_period_start: u64,
    pub current_period_end: u64,
    pub cancel_at_period_end: bool,
    pub items: Items,
}

#[derive(Debug, Deserialize)]
pub struct Items {
    pub data: Vec<ItemData>,
}

#[derive(Debug, Deserialize)]
pub struct ItemData {
    pub price: Price,
}

#[derive(Debug, Deserialize)]
pub struct Price {
    pub id: String,
}
