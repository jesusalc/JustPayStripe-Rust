use dotenvy::dotenv;
// use justpaystripe::client::StripeClient;
// use std::env;

#[tokio::test]
async fn test_create_customer() {
    dotenv().ok();
    // let secret = env::var("STRIPE_SECRET").expect("Missing STRIPE_SECRET in .env");
    // let client = StripeClient::new();

    // let params = vec![
    //     ("email", "test@example.com"),
    //     ("description", "Test User"),
    //     ("metadata[userId]", "test-user-1"),
    // ];

    // let resp: serde_json::Value = client
    //     .post("https://api.stripe.com/v1/customers", params)
    //     .await
    //     .unwrap();

    // assert!(resp["id"].as_str().unwrap().starts_with("cus_"));
}
