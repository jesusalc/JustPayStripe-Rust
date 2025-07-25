#[tokio::test]
async fn test_env_load_and_client() {
    let client = justpaystripe::StripeClient::new();
    assert!(client.api_key.starts_with("sk_"));
}
