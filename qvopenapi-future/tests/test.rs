use qvopenapi::AccountType;
use qvopenapi_future::{self, FutureQvOpenApiClient};
use qvopenapi_mock::{self, MockQvOpenApiClient};

#[tokio::test]
async fn test() {
    let client = FutureQvOpenApiClient::from(Box::new(MockQvOpenApiClient::new()));
    let connect_res = client.connect(0, AccountType::NAMUH, "id", "pw", "cert_pw").await.unwrap();
    assert_eq!(connect_res.account_count, 1);
}
