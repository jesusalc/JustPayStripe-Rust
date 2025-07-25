// use std::env;
use crate::stripe::Auth;
use dotenvy::dotenv;
use std::{
    // collections::{HashMap, HashSet},
    env as stdenv,
};

#[derive(Clone)]
pub struct StripeClient {
    pub api_key: String,
}

impl StripeClient {
    pub fn new() -> Self {
        dotenv().ok();
        let api_key = stdenv::var("STRIPE_SECRET_KEY").expect("STRIPE_SECRET_KEY not set in .env");
        Self { api_key }
    }
}

impl From<StripeClient> for Auth {
    fn from(client: StripeClient) -> Self {
        Auth {
            client: client.api_key.clone(),
            secret: client.api_key.clone(),
        }
    }
}
