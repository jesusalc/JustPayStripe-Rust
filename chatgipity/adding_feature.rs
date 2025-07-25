the following is an CODE: old stripe library, we need to upgrade it to a newer version.

according to ARTICLE: I am attaching, there is a proven way to use stripe, and it even has some typescript nextjs samples about it.

Can you analyze the CODE: and make detailed mermaid flow of current thing.

Then analyze how we can make a modernized version of this library

and make a bash script to deploy into rust cargo online to publish it

We need to create TESTS: we don't have any, we need to create USAGE: code that uses this library and proves its use and sample FRONT: that has front end calling it and making it the new library concepts work.

We need to automate use of TESTS: with sample stripe card 424242424242 and could read .env for STRIPE developer key.

Analize, think, make modern. Make useful.
Make new mermaid flowchar with new suggest CODE: FRONT: TESTS: USAGE: codes
Write the new codes CODE: FRONT: TESTS: USAGE:

use actix web not axum . zip a complete solution. Please check that all elements are being used. What hhapened to the older code PAyUP I had, It seems you have coded an entire new code, but left the other code unused. The idea was to enhance current code, not replace it. to enhance it. To Evolve it. Please review all answers and check that code is related and, reusable as a lib, also new example to run has to be in a different folder to setup and run for both front and back  exmaples how to setup and run .

CODE:



# file: src/lib.rs   // --- start
pub mod stripe;
# file: src/lib.rs   // --- end

# file: src/stripe.rs   // --- start
pub mod response;
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::convert::TryInto;
pub struct Auth {
    pub client: String,
    pub secret: String,
}
impl Auth {
    pub fn new(client: String, secret: String) -> Self {
        return Auth{client, secret};
    }
}
pub struct Balance {
    pub object: String,
    pub available: Vec<BalanceAvailable>,
    pub livemode: bool,
    pub pending: Vec<BalancePending>,
}
impl Balance {
    pub async fn async_get(creds: Auth) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/balance");
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/balance");
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
}
pub struct BalanceTransaction {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub available_on: i64,
    pub created: i64,
    pub currency: String,
    pub description: String,
    pub fee: i64,
    pub fee_details: Vec<FeeDetail>,
    pub net: i64,
    pub reporting_category: String,
    pub source: String,
    pub status: String,
    pub type_field: String,
}
impl BalanceTransaction {
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/balance_transactions/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/balance_transactions/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<BalanceTransactions, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/balance_transactions".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/balance_transactions?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<BalanceTransactions>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<BalanceTransactions, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/balance_transactions".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/balance_transactions?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<BalanceTransactions>()?;
        return Ok(json);
    }
}
pub struct Card {
    pub id: Option<String>,
    pub brand: Option<String>,
    pub last4: Option<String>,
    pub number: Option<String>,
    pub cvc: Option<String>,
    pub network: Option<String>,
    pub country: Option<String>,
    pub exp_month: Option<String>,
    pub exp_year: Option<String>,
    pub fingerprint: Option<String>,
}
impl Card {
    pub fn new() -> Self {
        return Card{
            id: None,
            brand: None,
            last4: None,
            number: None,
            cvc: None,
            network: None,
            country: None,
            exp_month: None,
            exp_year: None,
            fingerprint: None
        };
    }
}
pub struct Charge {
    pub id: Option<String>,
    pub object: Option<String>,
    pub amount: Option<String>,
    pub stripe_amount: Option<i64>,
    pub amount_captured: Option<i64>,
    pub amount_refunded: Option<i64>,
    pub balance_transaction: Option<String>,
    pub billing_details: Option<BillingDetails>,
    pub captured: Option<bool>,
    pub created: Option<i64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub disputed: Option<bool>,
    pub fraud_details: Option<FraudDetails>,
    pub livemode: Option<bool>,
    pub paid: Option<bool>,
    pub payment_method: Option<String>,
    pub payment_method_details: Option<PaymentMethodDetails>,
    pub receipt_url: Option<String>,
    pub refunded: Option<bool>,
    pub refunds: Option<Refunds>,
    pub status: Option<String>,
    pub customer: Option<String>,
    pub receipt_email: Option<String>,
    pub source: Option<String>,
    pub statement_descriptor: Option<String>,
    pub statement_descriptor_suffix: Option<String>,
}
impl Charge {
    pub fn new() -> Self {
        return Charge{
            id: None,
            object: None,
            amount: None,
            stripe_amount: None,
            amount_captured: None,
            amount_refunded: None,
            balance_transaction: None,
            billing_details: None,
            captured: None,
            created: None,
            currency: None,
            customer: None,
            description: None,
            disputed: None,
            fraud_details: None,
            livemode: None,
            paid: None,
            payment_method: None,
            payment_method_details: None,
            receipt_url: None,
            refunded: None,
            refunds: None,
            status: None,
            source: None,
            receipt_email: None,
            statement_descriptor: None,
            statement_descriptor_suffix: None
        };
    }
    pub async fn async_capture(&self, creds: Auth) ->  Result<Self, reqwest::Error>{
        let url = format!("https://api.stripe.com/v1/charges/{}/capture", self.id.clone().unwrap());
        let request = reqwest::Client::new().post(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_capture_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/charges/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/charges")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new().post(format!("https://api.stripe.com/v1/charges/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn capture(&self, creds: Auth) ->  Result<Self, reqwest::Error>{
        let url = format!("https://api.stripe.com/v1/charges/{}/capture", self.id.clone().unwrap());
        let request = reqwest::blocking::Client::new().post(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_capture_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/charges/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/charges")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post(format!("https://api.stripe.com/v1/charges/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<Charges, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/charges".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/charges?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Charges>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<Charges, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/charges".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/charges?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Charges>().await?;
        return Ok(json);
    }
    fn to_capture_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.receipt_email{
            Some(receipt_email) => params.push(("receipt_email", receipt_email.as_str())),
            None => {}
        }
        match &self.amount{
            Some(amount) => params.push(("amount", amount.as_str())),
            None => {}
        }
        match &self.statement_descriptor{
            Some(statement_descriptor) => params.push(("statement_descriptor", statement_descriptor.as_str())),
            None => {}
        }
        match &self.statement_descriptor_suffix{
            Some(statement_descriptor_suffix) => params.push(("statement_descriptor_suffix", statement_descriptor_suffix.as_str())),
            None => {}
        }
        return params;
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.customer{
            Some(customer) => params.push(("customer", customer.as_str())),
            None => {}
        }
        match &self.description{
            Some(description) => params.push(("description", description.as_str())),
            None => {}
        }
        match &self.receipt_email{
            Some(receipt_email) => params.push(("receipt_email", receipt_email.as_str())),
            None => {}
        }
        match &self.amount{
            Some(amount) => params.push(("amount", amount.as_str())),
            None => {}
        }
        match &self.currency{
            Some(currency) => params.push(("currency", currency.as_str())),
            None => {}
        }
        match &self.source{
            Some(source) => params.push(("source", source.as_str())),
            None => {}
        }
        match &self.statement_descriptor{
            Some(statement_descriptor) => params.push(("statement_descriptor", statement_descriptor.as_str())),
            None => {}
        }
        match &self.statement_descriptor_suffix{
            Some(statement_descriptor_suffix) => params.push(("statement_descriptor_suffix", statement_descriptor_suffix.as_str())),
            None => {}
        }
        return params;
    }
}
pub struct Customer {
    pub id: Option<String>,
    pub object: Option<String>,
    pub balance: Option<i64>,
    pub created: Option<i64>,
    pub currency: Option<String>,
    pub default_source: Option<String>,
    pub payment_method: Option<String>,
    pub delinquent: Option<bool>,
    pub description: Option<String>,
    pub email: Option<String>,
    pub invoice_prefix: Option<String>,
    pub livemode: Option<bool>,
    pub name: Option<String>,
    pub next_invoice_sequence: Option<i64>,
    pub phone: Option<String>,
    pub tax_exempt: Option<String>,
}
impl Customer {
    pub fn new() -> Self {
        return Customer{
            id: None,
            object: None,
            balance: None,
            created: None,
            currency: None,
            default_source: None,
            payment_method: None,
            delinquent: None,
            description: None,
            email: None,
            invoice_prefix: None,
            livemode: None,
            name: None,
            next_invoice_sequence: None,
            phone: None,
            tax_exempt: None
        };
    }
    pub async fn async_delete(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::Client::new().delete(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_invoices(creds: Auth, customer_id: String) -> Result<Vec<crate::stripe::response::Invoice>, reqwest::Error>{
        let mut objects: Vec<crate::stripe::response::Invoice> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::get_invoices_chunk(creds.clone(), customer_id.clone(), starting_after.clone())?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_payment_methods(creds: Auth, customer_id: String, method_type: String) -> Result<Vec<crate::stripe::response::PaymentMethod>, reqwest::Error>{
        let mut objects: Vec<crate::stripe::response::PaymentMethod> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::get_payment_methods_chunk_async(creds.clone(), customer_id.clone(), method_type.clone(), starting_after.clone()).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
        .post("https://api.stripe.com/v1/customers")
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .form(&self.to_params())
        .send().await;
        match request{
            Ok(req) => {
                let json = req.json::<Self>().await?;
                return Ok(json);
            },
            Err(err) => Err(err)
        }
    }
    pub async fn async_update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new().post(format!("https://api.stripe.com/v1/customers/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn delete(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::blocking::Client::new().delete(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(auth: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(auth.client.as_str(), Some(auth.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn invoices(creds: Auth, customer_id: String) -> Result<Vec<crate::stripe::response::Invoice>, reqwest::Error>{
        let mut objects: Vec<crate::stripe::response::Invoice> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::get_invoices_chunk(creds.clone(), customer_id.clone(), starting_after.clone())?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn payment_methods(creds: Auth, customer_id: String, method_type: String) -> Result<Vec<crate::stripe::response::PaymentMethod>, reqwest::Error>{
        let mut objects: Vec<crate::stripe::response::PaymentMethod> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::get_payment_methods_chunk(creds.clone(), customer_id.clone(), method_type.clone(), starting_after.clone())?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post("https://api.stripe.com/v1/customers")
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .form(&self.to_params())
        .send();
        match request{
            Ok(req) => {
                let json = req.json::<Self>()?;
                Ok(json)
            },
            Err(err) => Err(err)
        }
    }
    pub fn update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post(format!("https://api.stripe.com/v1/customers/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn get_invoices_chunk(creds: Auth, customer_id: String, starting_after: Option<String>) ->  Result<crate::stripe::response::Invoices, reqwest::Error>{
        let mut url = format!("https://api.stripe.com/v1/invoices?customer={}", customer_id);
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/invoices?customer={}&starting_after={}", customer_id, starting_after.unwrap())
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<crate::stripe::response::Invoices>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<Customers, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/customers".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/customers?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Customers>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<Customers, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/customers".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/customers?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Customers>().await?;
        return Ok(json);
    }
    fn get_payment_methods_chunk(creds: Auth, customer_id: String, method_type: String, starting_after: Option<String>) ->  Result<crate::stripe::response::PaymentMethods, reqwest::Error>{
        let mut url = format!("https://api.stripe.com/v1/customers/{}/payment_methods?type={}", customer_id, method_type);
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/customers/{}/payment_methods?type={}&starting_after={}", customer_id, method_type, starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<crate::stripe::response::PaymentMethods>()?;
        return Ok(json);
    }
    async fn get_payment_methods_chunk_async(creds: Auth, customer_id: String, method_type: String, starting_after: Option<String>) ->  Result<crate::stripe::response::PaymentMethods, reqwest::Error>{
        let mut url = format!("https://api.stripe.com/v1/customers/{}/payment_methods?type={}", customer_id, method_type);
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/customers/{}/payment_methods?type={}&starting_after={}", customer_id, method_type, starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<crate::stripe::response::PaymentMethods>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.payment_method{
            Some(payment_method) => params.push(("payment_method", payment_method.as_str())),
            None => {}
        }
        match &self.description{
            Some(description) => params.push(("description", description.as_str())),
            None => {}
        }
        match &self.email{
            Some(email) => params.push(("email", email.as_str())),
            None => {}
        }
        match &self.name{
            Some(name) => params.push(("name", name.as_str())),
            None => {}
        }
        match &self.phone{
            Some(phone) => params.push(("phone", phone.as_str())),
            None => {}
        }
        return params;
    }
}
pub struct Dispute {
    pub id: Option<String>,
    pub object: Option<String>,
    pub amount: Option<i64>,
    pub charge: Option<String>,
    pub created: Option<i64>,
    pub currency: Option<String>,
    pub evidence: Option<Evidence>,
    pub evidence_details: Option<EvidenceDetails>,
    pub is_charge_refundable: Option<bool>,
    pub livemode: Option<bool>,
    pub submit: Option<bool>,
    pub payment_intent: Option<String>,
    pub reason: Option<String>,
    pub status: Option<String>,
}
impl Dispute {
    pub fn new() -> Self {
        return Dispute{
            id: None,
            object: None,
            amount: None,
            charge: None,
            created: None,
            currency: None,
            evidence: None,
            evidence_details: None,
            is_charge_refundable: None,
            livemode: None,
            submit: None,
            payment_intent: None,
            reason: None,
            status: None
        };
    }
    pub async fn async_close(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new().post(format!("https://api.stripe.com/v1/disputes/{}/close", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/disputes/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new().post(format!("https://api.stripe.com/v1/disputes/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn close(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post(format!("https://api.stripe.com/v1/disputes/{}/close", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/disputes/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post(format!("https://api.stripe.com/v1/disputes/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<Disputes, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/disputes".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/disputes?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Disputes>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<Disputes, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/disputes".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/disputes?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Disputes>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.evidence{
            Some(evidence) => {
                match &evidence.access_activity_log{
                    Some(access_activity_log) => {
                        params.push(("evidence[access_activity_log]", access_activity_log.as_str()));
                    },
                    None => {}
                }
                match &evidence.billing_address{
                    Some(billing_address) => {
                        params.push(("evidence[billing_address]", billing_address.as_str()));
                    },
                    None => {}
                }
                match &evidence.cancellation_policy{
                    Some(cancellation_policy) => {
                        params.push(("evidence[cancellation_policy]", cancellation_policy.as_str()));
                    },
                    None => {}
                }
                match &evidence.cancellation_policy_disclosure{
                    Some(cancellation_policy_disclosure) => {
                        params.push(("evidence[cancellation_policy_disclosure]", cancellation_policy_disclosure.as_str()));
                    },
                    None => {}
                }
                match &evidence.cancellation_rebuttal{
                    Some(cancellation_rebuttal) => {
                        params.push(("evidence[cancellation_rebuttal]", cancellation_rebuttal.as_str()));
                    },
                    None => {}
                }
                match &evidence.customer_communication{
                    Some(customer_communication) => {
                        params.push(("evidence[customer_communication]", customer_communication.as_str()));
                    },
                    None => {}
                }
                match &evidence.customer_email_address{
                    Some(customer_email_address) => {
                        params.push(("evidence[customer_email_address]", customer_email_address.as_str()));
                    },
                    None => {}
                }
                match &evidence.customer_name{
                    Some(customer_name) => {
                        params.push(("evidence[customer_name]", customer_name.as_str()));
                    },
                    None => {}
                }
                match &evidence.customer_purchase_ip{
                    Some(customer_purchase_ip) => {
                        params.push(("evidence[customer_purchase_ip]", customer_purchase_ip.as_str()));
                    },
                    None => {}
                }
                match &evidence.customer_signature{
                    Some(customer_signature) => {
                        params.push(("evidence[customer_signature]", customer_signature.as_str()));
                    },
                    None => {}
                }
                match &evidence.duplicate_charge_documentation{
                    Some(duplicate_charge_documentation) => {
                        params.push(("evidence[duplicate_charge_documentation]", duplicate_charge_documentation.as_str()));
                    },
                    None => {}
                }
                match &evidence.duplicate_charge_explanation{
                    Some(duplicate_charge_explanation) => {
                        params.push(("evidence[duplicate_charge_explanation]", duplicate_charge_explanation.as_str()));
                    },
                    None => {}
                }
                match &evidence.duplicate_charge_id{
                    Some(duplicate_charge_id) => {
                        params.push(("evidence[duplicate_charge_id]", duplicate_charge_id.as_str()));
                    },
                    None => {}
                }
                match &evidence.product_description{
                    Some(product_description) => {
                        params.push(("evidence[product_description]", product_description.as_str()));
                    },
                    None => {}
                }
                match &evidence.receipt{
                    Some(receipt) => {
                        params.push(("evidence[receipt]", receipt.as_str()));
                    },
                    None => {}
                }
                match &evidence.refund_policy{
                    Some(refund_policy) => {
                        params.push(("evidence[refund_policy]", refund_policy.as_str()));
                    },
                    None => {}
                }
                match &evidence.refund_policy_disclosure{
                    Some(refund_policy_disclosure) => {
                        params.push(("evidence[refund_policy_disclosure]", refund_policy_disclosure.as_str()));
                    },
                    None => {}
                }
                match &evidence.refund_refusal_explanation{
                    Some(refund_refusal_explanation) => {
                        params.push(("evidence[refund_refusal_explanation]", refund_refusal_explanation.as_str()));
                    },
                    None => {}
                }
                match &evidence.service_date{
                    Some(service_date) => {
                        params.push(("evidence[service_date]", service_date.as_str()));
                    },
                    None => {}
                }
                match &evidence.service_documentation{
                    Some(service_documentation) => {
                        params.push(("evidence[service_documentation]", service_documentation.as_str()));
                    },
                    None => {}
                }
                match &evidence.shipping_address{
                    Some(shipping_address) => {
                        params.push(("evidence[shipping_address]", shipping_address.as_str()));
                    },
                    None => {}
                }
                match &evidence.shipping_carrier{
                    Some(shipping_carrier) => {
                        params.push(("evidence[shipping_carrier]", shipping_carrier.as_str()));
                    },
                    None => {}
                }
                match &evidence.shipping_date{
                    Some(shipping_date) => {
                        params.push(("evidence[shipping_date]", shipping_date.as_str()));
                    },
                    None => {}
                }
                match &evidence.shipping_documentation{
                    Some(shipping_documentation) => {
                        params.push(("evidence[shipping_documentation]", shipping_documentation.as_str()));
                    },
                    None => {}
                }
                match &evidence.shipping_tracking_number{
                    Some(shipping_tracking_number) => {
                        params.push(("evidence[shipping_tracking_number]", shipping_tracking_number.as_str()));
                    },
                    None => {}
                }
                match &evidence.uncategorized_file{
                    Some(uncategorized_file) => {
                        params.push(("evidence[uncategorized_file]", uncategorized_file.as_str()));
                    },
                    None => {}
                }
                match &evidence.uncategorized_text{
                    Some(uncategorized_text) => {
                        params.push(("evidence[uncategorized_text]", uncategorized_text.as_str()));
                    },
                    None => {}
                }
            },
            None => {}
        }
        match &self.submit{
            Some(submit) => {
                if *submit{
                    params.push(("submit", "true"));
                } else {
                    params.push(("submit", "false"));
                }
            },
            None => {}
        }
        return params;
    }
}
pub struct Event {
    pub id: Option<String>,
    pub object: Option<String>,
    pub api_version: Option<String>,
    pub created: Option<i64>,
    pub livemode: Option<bool>,
    pub pending_webhooks: Option<i64>,
    pub request: Option<Request>,
    pub type_field: Option<String>,
}
impl Event {
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/events/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/events/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<Events, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/events".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/events?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Events>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<Events, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/events".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/events?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Events>().await?;
        return Ok(json);
    }
}
pub struct File {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub expires_at: Option<i64>,
    pub filename: Option<String>,
    pub links: Option<Links>,
    pub purpose: Option<String>,
    pub size: Option<i64>,
    pub title: Option<String>,
    pub type_field: Option<String>,
    pub url: Option<String>,
    pub file: Option<Vec<u8>>,
}
impl File {
    pub fn new() -> Self {
        return File{
            id: None,
            object: None,
            created: None,
            expires_at: None,
            filename: None,
            links: None,
            purpose: None,
            size: None,
            title: None,
            type_field: None,
            url: None,
            file: None,
        };
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/files/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let mut form = self.to_multipart_form_async().await;
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/files")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .multipart(form)
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let mut form = self.to_multipart_form();
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/files")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .multipart(form)
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/files/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<Files, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/files".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/files?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Files>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<Files, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/files".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/files?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Files>().await?;
        return Ok(json);
    }
    fn to_multipart_form(&self) -> reqwest::blocking::multipart::Form {
        let mut form = reqwest::blocking::multipart::Form::new();
        match &self.purpose{
            Some(purpose) => {
                form = form.text("purpose", purpose.clone());
            },
            None => {
            }
        }
        match &self.file{
            Some(file) => {
                let part = reqwest::blocking::multipart::Part::bytes(file.clone());
                form = form.part("file", part);
            },
            None => {
            }
        }
        return form;
    }
    async fn to_multipart_form_async(&self) -> reqwest::multipart::Form {
        let mut form = reqwest::multipart::Form::new();
        match &self.purpose{
            Some(purpose) => {
                form = form.text("purpose", purpose.clone());
            },
            None => {
            }
        }
        match &self.file{
            Some(file) => {
                let part = reqwest::multipart::Part::bytes(file.clone());
                form = form.part("file", part);
            },
            None => {
            }
        }
        return form;
    }
}
pub struct FileLink {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub expired: Option<bool>,
    pub expires_at: Option<i64>,
    pub link_expires_at: Option<String>,
    pub file: Option<String>,
    pub livemode: Option<bool>,
    pub url: Option<String>
}
impl FileLink {
    pub fn new() -> Self {
        return FileLink {
            id: None,
            object: None,
            created: None,
            expired: None,
            expires_at: None,
            link_expires_at: None,
            file: None,
            livemode: None,
            url: None
        };
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/file_links")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new().post(format!("https://api.stripe.com/v1/file_links/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/file_links")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post(format!("https://api.stripe.com/v1/file_links/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<FileLinks, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/file_links".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/file_links?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<FileLinks>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<FileLinks, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/file_links".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/file_links?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<FileLinks>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.file{
            Some(file) => params.push(("file", file.as_str())),
            None => {}
        }
        match &self.link_expires_at{
            Some(link_expires_at) => params.push(("expires_at", link_expires_at.as_str())),
            None => {}
        }
        return params;
    }
}
pub struct Invoice {
    pub id: Option<String>,
    pub object: Option<String>,
    pub account_country: Option<String>,
    pub account_name: Option<String>,
    pub account_tax_ids: Option<String>,
    pub amount_due: Option<i64>,
    pub amount_paid: Option<i64>,
    pub amount_remaining: Option<i64>,
    pub application_fee_amount: Option<i64>,
    pub attempt_count: Option<i64>,
    pub attempted: Option<bool>,
    pub auto_advance: Option<bool>,
    pub billing_reason: Option<String>,
    pub collection_method: Option<String>,
    pub created: Option<i64>,
    pub currency: Option<String>,
    pub customer: Option<String>,
    pub customer_address: Option<String>,
    pub customer_email: Option<String>,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub customer_shipping: Option<String>,
    pub customer_tax_exempt: Option<String>,
    pub customer_tax_ids: Option<Vec<String>>,
    pub default_payment_method: Option<String>,
    pub default_source: Option<String>,
    pub description: Option<String>,
    pub hosted_invoice_url: Option<String>,
    pub invoice_pdf: Option<String>,
    pub lines: Option<InvoiceLines>,
    pub livemode: Option<bool>,
    pub next_payment_attempt: Option<i64>,
    pub paid: Option<bool>,
    pub paid_out_of_band: Option<bool>,
    pub payment_settings: Option<PaymentSettings>,
    pub period_end: Option<i64>,
    pub period_start: Option<i64>,
    pub post_payment_credit_notes_amount: Option<i64>,
    pub pre_payment_credit_notes_amount: Option<i64>,
    pub starting_balance: Option<i64>,
    pub status: Option<String>,
    pub status_transitions: Option<StatusTransitions>,
    pub subtotal: Option<i64>,
    pub subscription: Option<String>,
    pub total: Option<i64>,
}
impl Invoice {
    pub fn new() -> Self {
        return Invoice{
            id: None,
            object: None,
            account_country: None,
            account_name: None,
            account_tax_ids: None,
            amount_due: None,
            amount_paid: None,
            amount_remaining: None,
            application_fee_amount: None,
            attempt_count: None,
            attempted: None,
            auto_advance: None,
            billing_reason: None,
            collection_method: None,
            created: None,
            currency: None,
            customer: None,
            customer_address: None,
            customer_email: None,
            customer_name: None,
            customer_phone: None,
            customer_shipping: None,
            customer_tax_exempt: None,
            customer_tax_ids: None,
            default_payment_method: None,
            default_source: None,
            description: None,
            hosted_invoice_url: None,
            invoice_pdf: None,
            lines: None,
            livemode: None,
            next_payment_attempt: None,
            paid: None,
            paid_out_of_band: None,
            payment_settings: None,
            period_end: None,
            period_start: None,
            post_payment_credit_notes_amount: None,
            pre_payment_credit_notes_amount: None,
            subscription: None,
            status: None,
            starting_balance: None,
            status_transitions: None,
            subtotal: None,
            total: None
        };
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/invoices/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth, status: Option<String>, customer: Option<String>) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after, status.clone(), customer.clone()).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/invoices")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::Client::new().post(format!("https://api.stripe.com/v1/invoices/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/invoices/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth, status: Option<String>, customer: Option<String>) -> Result<Vec<Self>, reqwest::Error>{
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after, status.clone(), customer.clone())?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/invoices")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn update(&self, creds: Auth) ->  Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post(format!("https://api.stripe.com/v1/invoices/{}", self.clone().id.unwrap()))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>, status: Option<String>, customer: Option<String>) -> Result<Invoices, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/invoices".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/invoices?starting_after={}", starting_after.unwrap());
        }
        if status.is_some(){
            if url.contains("?"){
                url = format!("{}{}={}", url, "&status", status.unwrap());
            } else {
                url = format!("{}{}={}", url, "?status", status.unwrap());
            }
        }
        if customer.is_some(){
            if url.contains("?"){
                url = format!("{}{}={}", url, "&customer", customer.unwrap());
            } else {
                url = format!("{}{}={}", url, "?customer", customer.unwrap());
            }
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Invoices>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>, status: Option<String>, customer: Option<String>) -> Result<Invoices, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/invoices".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/invoices?starting_after={}", starting_after.unwrap());
        }
        if status.is_some(){
            if url.contains("?"){
                url = format!("{}{}={}", url, "&status", status.unwrap());
            } else {
                url = format!("{}{}={}", url, "?status", status.unwrap());
            }
        }
        if customer.is_some(){
            if url.contains("?"){
                url = format!("{}{}={}", url, "&customer", customer.unwrap());
            } else {
                url = format!("{}{}={}", url, "?customer", customer.unwrap());
            }
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Invoices>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.customer{
            Some(customer) => params.push(("customer", customer.as_str())),
            None => {}
        }
        match &self.collection_method{
            Some(collection_method) => params.push(("collection_method", collection_method.as_str())),
            None => {}
        }
        match &self.description{
            Some(description) => params.push(("description", description.as_str())),
            None => {}
        }
        match &self.subscription{
            Some(subscription) => params.push(("subscription", subscription.as_str())),
            None => {}
        }
        return params;
    }
}
pub struct Mandate {
    pub id: String,
    pub object: String,
    pub customer_acceptance: CustomerAcceptance,
    pub livemode: bool,
    pub payment_method: String,
    pub payment_method_details: PaymentMethodDetails,
    pub status: String,
    pub type_field: String,
}
impl Mandate {
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
}
pub struct PaymentMethod {
    pub id: Option<String>,
    pub method_type: Option<String>,
    pub created: Option<String>,
    pub customer: Option<String>,
    pub livemode:  Option<bool>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub billing_details: Option<crate::stripe::response::BillingDetails>,
    pub card: Option<Card>,
    pub type_field: Option<String>,
}
impl PaymentMethod {
    pub fn new() -> Self {
        return PaymentMethod{
            id: None,
            method_type: None,
            created: None,
            customer: None,
            livemode: None,
            name: None,
            phone: None,
            billing_details: None,
            card: None,
            type_field: None
        };
    }
    pub fn attach(&self, customer: Customer, creds: Auth) ->  Result<bool, reqwest::Error>{
        match &self.id{
            Some(id) => {
                match &customer.id{
                    Some(cust_id) => {
                        let url = format!("https://api.stripe.com/v1/payment_methods/{}/attach", id.clone());
                        let params = [
                            ("customer", cust_id.as_str())
                        ];
                        let request = reqwest::blocking::Client::new().post(url)
                        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
                        .form(&params)
                        .send()?;
                        return Ok(true);
                    },
                    None => return Ok(false)
                }
            },
            None => return Ok(false)
        }
        return Ok(false);
    }
    pub fn get(creds: Auth, id: String) -> Result<crate::stripe::response::PaymentMethod, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/payment_methods/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url)
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .send();
        match request{
            Ok(req) => {
                let json = req.json::<crate::stripe::response::PaymentMethod>().unwrap();
                return Ok(json);
            },
            Err(err) => Err(err)
        }
    }
    pub fn post(&self, creds: Auth) ->  Result<PaymentMethod, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post("https://api.stripe.com/v1/payment_methods")
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .form(&self.to_params())
        .send();
        match request{
            Ok(req) => {
                let mut plan = self.clone();
                let json = req.json::<crate::stripe::response::PaymentMethod>()?;
                plan.id = Some(json.id);
                Ok(plan)
            },
            Err(err) => Err(err)
        }
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.method_type{
            Some(method_type) => params.push(("type", method_type.as_str())),
            None => {}
        }
        match &self.card{
            Some(card) => {
                match &card.number{
                    Some(number) => params.push(("card[number]", number.as_str())),
                    None => {}
                }
                match &card.exp_month{
                    Some(exp_month) => params.push(("card[exp_month]", exp_month.as_str())),
                    None => {}
                }
                match &card.exp_year{
                    Some(exp_year) => params.push(("card[exp_year]", exp_year.as_str())),
                    None => {}
                }
                match &card.cvc{
                    Some(cvc) => params.push(("card[cvc]", cvc.as_str())),
                    None => {}
                }
            },
            None => {}
        }
        return params;
    }
}
pub struct Plan {
    pub id: Option<String>,
    pub active: Option<String>,
    pub amount: Option<String>,
    pub amount_decimal: Option<String>,
    pub billing_scheme: Option<String>,
    pub created: Option<i64>,
    pub currency: Option<String>,
    pub interval: Option<String>,
    pub interval_count: Option<String>,
    pub product: Option<String>,
}
impl Plan {
    pub fn new() -> Self {
        return Plan {
            id: None,
            active: None,
            amount: None,
            amount_decimal: None,
            billing_scheme: None,
            created: None,
            currency: None,
            interval: None,
            interval_count: None,
            product: None
        };
    }
    pub async fn async_delete(creds: Auth, id: String) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::Client::new().delete(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<crate::stripe::response::Plan>().await?;
        return Ok(json);
    }
    pub async fn async_get(auth: Auth, id: String) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::Client::new().get(url).basic_auth(auth.client.as_str(), Some(auth.secret.as_str())).send().await?;
        let json = request.json::<crate::stripe::response::Plan>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<crate::stripe::response::Plan>, reqwest::Error>{
        let mut objects: Vec<crate::stripe::response::Plan> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) ->  Result<crate::stripe::response::Plan, reqwest::Error> {
        let request = reqwest::Client::new().post("https://api.stripe.com/v1/plans")
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .form(&self.to_params())
        .send().await?;
        let json = request.json::<crate::stripe::response::Plan>().await?;
        return Ok(json);
    }
    pub fn delete(creds: Auth, id: String) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::blocking::Client::new().delete(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<crate::stripe::response::Plan>()?;
        return Ok(json);
    }
    pub fn get(auth: Auth, id: String) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url).basic_auth(auth.client.as_str(), Some(auth.secret.as_str())).send()?;
        let json = request.json::<crate::stripe::response::Plan>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<crate::stripe::response::Plan>, reqwest::Error>{
        let mut objects: Vec<crate::stripe::response::Plan> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more{
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data{
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) ->  Result<crate::stripe::response::Plan, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post("https://api.stripe.com/v1/plans")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<crate::stripe::response::Plan>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<crate::stripe::response::Plans, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/plans".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/plans?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::blocking::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send()?;
        let json = request.json::<crate::stripe::response::Plans>()?;
        return Ok(json);
    }
    async fn list_chunk_async(creds: Auth, starting_after: Option<String>) -> Result<crate::stripe::response::Plans, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/plans".to_string();
        if starting_after.is_some() {
            url = format!("https://api.stripe.com/v1/plans?starting_after={}", starting_after.unwrap());
        }
        let request = reqwest::Client::new().get(url).basic_auth(creds.client.as_str(), Some(creds.secret.as_str())).send().await?;
        let json = request.json::<crate::stripe::response::Plans>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.amount{
            Some(amount) => params.push(("amount", amount.as_str())),
            None => {}
        }
        match &self.currency{
            Some(currency) => params.push(("currency", currency.as_str())),
            None => {}
        }
        match &self.interval{
            Some(interval) => params.push(("interval", interval.as_str())),
            None => {}
        }
        match &self.product{
            Some(product) => params.push(("product", product.as_str())),
            None => {}
        }
        match &self.active{
            Some(active) => params.push(("active", active.as_str())),
            None => {}
        }
        return params;
    }
}
pub struct Price {
    pub id: Option<String>,
    pub active: Option<bool>,
    pub billing_scheme: Option<String>,
    pub created: Option<i64>,
    pub currency: Option<String>,
    pub livemode: Option<bool>,
    pub product: Option<String>,
    pub tax_behavior: Option<String>,
    pub type_field: Option<String>,
    pub unit_amount: Option<String>,
    pub unit_amount_decimal: Option<String>,
}
impl Price {
    pub fn new() -> Self {
        return Price{
            id: None,
            active: None,
            billing_scheme: None,
            created: None,
            currency: None,
            livemode: None,
            product: None,
            tax_behavior: None,
            type_field: None,
            unit_amount: None,
            unit_amount_decimal: None
        };
    }
    pub fn post(&self, creds: Auth) ->  Result<Price, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post("https://api.stripe.com/v1/prices")
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .form(&self.to_params())
        .send();
        match request{
            Ok(req) => {
                let mut plan = self.clone();
                let json = req.json::<crate::stripe::response::Price>()?;
                plan.id = Some(json.id);
                Ok(plan)
            },
            Err(err) => Err(err)
        }
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.currency{
            Some(currency) => params.push(("currency", currency.as_str())),
            None => {}
        }
        match &self.unit_amount{
            Some(unit_amount) => params.push(("unit_amount", unit_amount.as_str())),
            None => {}
        }
        return params;
    }
}
pub struct Subscription {
    pub id: Option<String>,
    pub billing_cycle_anchor: Option<i64>,
    pub cancel_at: Option<i64>,
    pub cancel_at_period_end: Option<bool>,
    pub canceled_at: Option<i64>,
    pub collection_method: Option<String>,
    pub created: Option<i64>,
    pub current_period_end: Option<i64>,
    pub current_period_start: Option<i64>,
    pub customer: Option<String>,
    pub days_until_due: Option<i64>,
    pub default_payment_method: Option<String>,
    pub ended_at: Option<i64>,
    pub latest_invoice: Option<String>,
    pub livemode: Option<bool>,
    pub quantity: Option<i64>,
    pub start_date: Option<i64>,
    pub status: Option<String>,
    pub price_items: Option<Vec<String>>
}
impl Subscription {
    pub fn new() -> Self {
        return Subscription{
            id: None,
            billing_cycle_anchor: None,
            cancel_at: None,
            cancel_at_period_end: None,
            canceled_at: None,
            collection_method: None,
            created: None,
            current_period_end: None,
            current_period_start: None,
            customer: None,
            price_items: None,
            days_until_due: None,
            default_payment_method: None,
            ended_at: None,
            latest_invoice: None,
            livemode: None,
            quantity: None,
            start_date: None,
            status: None
        };
    }
    pub fn cancel(creds: Auth, id: String) -> Result<crate::stripe::response::Subscription, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/subscriptions/{}", id.clone());
        let request = reqwest::blocking::Client::new().delete(url)
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .send();
        match request{
            Ok(req) => {
                let json = req.json::<crate::stripe::response::Subscription>().unwrap();
                return Ok(json);
            },
            Err(err) => Err(err)
        }
    }
    pub fn get(creds: Auth, id: String) -> Result<crate::stripe::response::Subscription, reqwest::Error> {
        let mut url = format!("https://api.stripe.com/v1/subscriptions/{}", id.clone());
        let request = reqwest::blocking::Client::new().get(url)
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .send();
        match request{
            Ok(req) => {
                let json = req.json::<crate::stripe::response::Subscription>().unwrap();
                return Ok(json);
            },
            Err(err) => Err(err)
        }
    }
    pub fn update(&self, creds: Auth) ->  Result<crate::stripe::response::Subscription, reqwest::Error> {
        let request = reqwest::blocking::Client::new().post(format!("https://api.stripe.com/v1/subscriptions/{}", self.clone().id.unwrap()))
        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
        .form(&self.to_params())
        .send();
        match request{
            Ok(req) => {
                let json = req.json::<crate::stripe::response::Subscription>()?;
                Ok(json)
            },
            Err(err) => Err(err)
        }
    }
    pub fn post(&self, creds: Auth) -> Result<Subscription, reqwest::Error>{
        let request = reqwest::blocking::Client::new().post("https://api.stripe.com/v1/subscriptions")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Subscription>()?;
        Ok(json)
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.customer{
            Some(customer) => params.push(("customer", customer.as_str())),
            None => {}
        }
        match &self.default_payment_method{
            Some(default_payment_method) => params.push(("default_payment_method", default_payment_method.as_str())),
            None => {}
        }
        match &self.price_items{
            Some(price_items) => {
                let mut ii = 0;
                for (item) in price_items{
                    if ii < 20{
                        if ii == 0{
                            params.push(("items[0][price]", item.as_str()));
                        }
                        ii+=1;
                    }
                }
            },
            None => {}
        }
        return params;
    }
}
pub struct BalanceAvailable {
    pub amount: i64,
    pub currency: String,
    pub source_types: BalanceSourceTypes,
}
pub struct BalanceSourceTypes {
    pub card: i64,
}
pub struct BalancePending {
    pub amount: i64,
    pub currency: String,
    pub source_types: BalanceSourceTypes,
}
pub struct BalanceTransactions {
    pub object: String,
    pub data: Vec<BalanceTransaction>,
    pub has_more: bool,
    pub url: String,
}
pub struct FeeDetail {
    pub amount: i64,
    pub currency: String,
    pub description: String,
    pub type_field: String,
}
pub struct Charges {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<Charge>,
}
pub struct PaymentMethodDetails {
    pub sepa_debit: Option<SepaDebit>,
    pub card: Option<Card>,
    pub type_field: Option<String>,
}
pub struct FraudDetails {
}
pub struct BillingDetails {
    pub address: Option<Address>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
}
pub struct Address {
    pub city: Option<String>,
    pub country: Option<String>,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub postal_code: Option<String>,
    pub state: Option<String>,
}
pub struct Refunds {
    pub object: String,
    pub has_more: bool,
    pub url: String,
}
pub struct Customers {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<Customer>,
}
pub struct Disputes {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<Dispute>,
}
pub struct Evidence {
    pub access_activity_log: Option<String>,
    pub billing_address: Option<String>,
    pub cancellation_policy: Option<String>,
    pub cancellation_policy_disclosure: Option<String>,
    pub cancellation_rebuttal: Option<String>,
    pub customer_communication: Option<String>,
    pub customer_email_address: Option<String>,
    pub customer_name: Option<String>,
    pub customer_purchase_ip: Option<String>,
    pub customer_signature: Option<String>,
    pub duplicate_charge_documentation: Option<String>,
    pub duplicate_charge_explanation: Option<String>,
    pub duplicate_charge_id: Option<String>,
    pub product_description: Option<String>,
    pub receipt: Option<String>,
    pub refund_policy: Option<String>,
    pub refund_policy_disclosure: Option<String>,
    pub refund_refusal_explanation: Option<String>,
    pub service_date: Option<String>,
    pub service_documentation: Option<String>,
    pub shipping_address: Option<String>,
    pub shipping_carrier: Option<String>,
    pub shipping_date: Option<String>,
    pub shipping_documentation: Option<String>,
    pub shipping_tracking_number: Option<String>,
    pub uncategorized_file: Option<String>,
    pub uncategorized_text: Option<String>,
}
impl Evidence {
    pub fn new() -> Self {
        return Evidence{
            access_activity_log: None,
            billing_address: None,
            cancellation_policy: None,
            cancellation_policy_disclosure: None,
            cancellation_rebuttal: None,
            customer_communication: None,
            customer_email_address: None,
            customer_name: None,
            customer_purchase_ip: None,
            customer_signature: None,
            duplicate_charge_documentation: None,
            duplicate_charge_explanation: None,
            duplicate_charge_id: None,
            product_description: None,
            receipt: None,
            refund_policy: None,
            refund_policy_disclosure: None,
            refund_refusal_explanation: None,
            service_date: None,
            service_documentation: None,
            shipping_address: None,
            shipping_carrier: None,
            shipping_date: None,
            shipping_documentation: None,
            shipping_tracking_number: None,
            uncategorized_file: None,
            uncategorized_text: None
        };
    }
}
pub struct EvidenceDetails {
    pub due_by: i64,
    pub has_evidence: bool,
    pub past_due: bool,
    pub submission_count: i64,
}
pub struct Events {
    pub object: String,
    pub data: Vec<Event>,
    pub has_more: bool,
    pub url: String,
}
pub struct Request {
    pub id: String,
    pub idempotency_key: String,
}
pub struct Files {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<File>,
}
pub struct Links {
    pub object: String,
    pub has_more: bool,
    pub url: String,
}
pub struct FileLinks {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<FileLink>,
}
pub struct Invoices {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<Invoice>,
}
pub struct CustomerAcceptance {
    pub accepted_at: i64,
    pub online: Online,
    pub type_field: String,
}
pub struct Online {
    pub ip_address: String,
    pub user_agent: String,
}
pub struct SepaDebit {
    pub reference: String,
    pub url: String,
}
pub struct InvoiceLines {
    pub object: String,
    pub data: Vec<InvoiceLine>,
    pub has_more: bool,
    pub url: String,
}
pub struct InvoiceLine {
    pub id: Option<String>,
    pub object: Option<String>,
    pub amount: Option<i64>,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub discountable: Option<bool>,
    pub invoice_item: Option<String>,
    pub livemode: Option<bool>,
    pub proration: Option<bool>,
    pub quantity: Option<i64>,
    pub type_field: Option<String>,
}
pub struct Period {
    pub end: Option<i64>,
    pub start: Option<i64>,
}
pub struct PaymentSettings {
}
pub struct StatusTransitions {
    pub finalized_at: Option<i64>,
    pub marked_uncollectible_at: Option<i64>,
    pub paid_at: Option<i64>,
    pub voided_at: Option<i64>,
}
# file: src/stripe.rs   // --- end

# file: src/stripe/response.rs   // --- start
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
pub struct Customer {
    pub id: String,
    pub object: String,
    pub description: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
}
pub struct InvoiceSettings {
    pub custom_fields: Value,
    pub default_payment_method: Value,
    pub footer: Value,
}
pub struct Plans {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<Plan>,
}
pub struct Plan {
    pub id: String,
    pub object: String,
    pub active: bool,
    pub amount: i64,
    pub amount_decimal: String,
    pub billing_scheme: String,
    pub created: i64,
    pub currency: String,
    pub interval: String,
    pub interval_count: i64,
    pub livemode: bool,
    pub product: String,
    pub usage_type: String,
}
pub struct Metadata {
}
pub struct BillingDetails {
    pub address: Address,
    pub email: Option<String>,
    pub name: Option<String>,
    pub phone: Option<String>,
}
pub struct Address {
    pub city: Option<String>,
    pub country: Option<String>,
    pub line1: Option<String>,
    pub line2: Option<String>,
    pub postal_code: Option<String>,
    pub state: Option<String>,
}
pub struct Card {
    pub brand: String,
    pub checks: Checks,
    pub country: String,
    pub exp_month: i64,
    pub exp_year: i64,
    pub fingerprint: String,
    pub funding: String,
    pub last4: String,
}
pub struct Checks {
    pub cvc_check: String,
}
pub struct PaymentMethods {
    pub object: String,
    pub url: String,
    pub has_more: bool,
    pub data: Vec<PaymentMethod>,
}
pub struct PaymentMethod {
    pub id: String,
    pub object: String,
    pub billing_details: BillingDetails,
    pub card: Card,
    pub created: i64,
    pub customer: Option<String>,
    pub livemode: bool,
    pub metadata: Metadata,
    pub type_field: String,
}
pub struct Networks {
    pub available: Vec<String>,
}
pub struct ThreeDSecureUsage {
    pub supported: bool,
}
pub struct Subscriptions {
    pub object: String,
    pub data: Vec<Subscription>,
    pub has_more: bool,
    pub url: String,
}
pub struct Subscription {
    pub id: Option<String>,
    pub object: Option<String>,
    pub automatic_tax: Option<AutomaticTax>,
    pub billing_cycle_anchor: Option<i64>,
    pub cancel_at: Option<i64>,
    pub cancel_at_period_end: Option<bool>,
    pub canceled_at: Option<i64>,
    pub collection_method: Option<String>,
    pub created: Option<i64>,
    pub current_period_end: Option<i64>,
    pub current_period_start: Option<i64>,
    pub customer: Option<String>,
    pub days_until_due: Option<i64>,
    pub default_payment_method: Option<String>,
    pub ended_at: Option<i64>,
    pub items: Option<SubscriptionItems>,
    pub latest_invoice: Option<String>,
    pub livemode: Option<bool>,
    pub quantity: Option<i64>,
    pub start_date: Option<i64>,
    pub status: Option<String>,
}
pub struct SubscriptionItems {
    pub object: String,
    pub data: Vec<SubscriptionItem>,
    pub has_more: bool,
    pub total_count: i64,
    pub url: String,
}
pub struct SubscriptionItem {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub quantity: i64,
    pub subscription: String,
}
pub struct AutomaticTax {
    pub enabled: Option<bool>,
    pub status: Option<String>,
}
pub struct Prices {
    pub object: String,
    pub data: Vec<Price>,
    pub has_more: bool,
    pub url: String,
}
pub struct Price {
    pub id: String,
    pub object: String,
    pub active: bool,
    pub billing_scheme: String,
    pub created: i64,
    pub currency: String,
    pub livemode: bool,
    pub product: String,
    pub recurring: Recurring,
    pub tax_behavior: String,
    pub type_field: String,
    pub unit_amount: i64,
    pub unit_amount_decimal: String,
}
pub struct Recurring {
    pub interval: String,
    pub interval_count: i64,
    pub usage_type: String,
}
pub struct Invoices {
    pub object: String,
    pub data: Vec<Invoice>,
    pub has_more: bool,
    pub url: String,
}
pub struct Invoice {
    pub id: String,
    pub object: String,
    pub account_country: String,
    pub amount_due: i64,
    pub amount_paid: i64,
    pub amount_remaining: i64,
    pub application_fee_amount: Value,
    pub attempt_count: i64,
    pub attempted: bool,
    pub auto_advance: bool,
    pub automatic_tax: AutomaticTax,
    pub billing_reason: String,
    pub charge: String,
    pub collection_method: String,
    pub created: i64,
    pub currency: String,
    pub customer: String,
    pub customer_email: String,
    pub customer_name: String,
    pub customer_phone: Option<String>,
    pub customer_tax_exempt: String,
    pub customer_tax_ids: Vec<Value>,
    pub default_payment_method: Value,
    pub default_source: Value,
    pub default_tax_rates: Vec<Value>,
    pub description: Value,
    pub discount: Value,
    pub discounts: Vec<Value>,
    pub due_date: Value,
    pub ending_balance: i64,
    pub footer: Value,
    pub hosted_invoice_url: String,
    pub invoice_pdf: String,
    pub last_finalization_error: Value,
    pub lines: InvoiceLines,
    pub livemode: bool,
    pub next_payment_attempt: Value,
    pub number: String,
    pub on_behalf_of: Value,
    pub paid: bool,
    pub paid_out_of_band: bool,
    pub payment_intent: String,
    pub period_end: i64,
    pub period_start: i64,
    pub post_payment_credit_notes_amount: i64,
    pub pre_payment_credit_notes_amount: i64,
    pub quote: Value,
    pub receipt_number: Value,
    pub starting_balance: i64,
    pub statement_descriptor: Value,
    pub status: String,
    pub status_transitions: StatusTransitions,
    pub subscription: String,
    pub subtotal: i64,
    pub tax: Value,
    pub total: i64,
    pub total_discount_amounts: Vec<Value>,
    pub total_tax_amounts: Vec<Value>,
    pub transfer_data: Value,
    pub webhooks_delivered_at: i64,
}
pub struct InvoiceLines {
    pub object: String,
    pub data: Vec<InvoiceLine>,
    pub has_more: bool,
    pub total_count: i64,
    pub url: String,
}
pub struct InvoiceLine {
    pub id: String,
    pub object: String,
    pub amount: i64,
    pub currency: String,
    pub description: String,
    pub discountable: bool,
    pub discounts: Vec<Value>,
    pub livemode: bool,
    pub period: Period,
    pub plan: Plan,
    pub price: Price,
    pub proration: bool,
    pub quantity: i64,
    pub subscription: String,
    pub subscription_item: String,
    pub type_field: String,
}
pub struct Period {
    pub end: i64,
    pub start: i64,
}
pub struct StatusTransitions {
    pub finalized_at: Option<i64>,
    pub paid_at: Option<i64>,
    pub voided_at: Option<i64>,
}
# file: src/stripe/response.rs   // --- end

# file: Cargo.toml   // --- start
[package]
name = "justpaystripe"
description = "A synchronous + asynchronous payment library for processing payments with rust + stripe."
version = "0.1.45"
edition = "2018"
authors = ["Caleb Mitchell Smith-Woolrich <calebsmithwoolrich@gmail.com>"]
license = "MIT OR Apache-2.0"
documentation = "https://docs.rs/justpaystripe"
repository = "https://github.com/PixelCoda/Payup-Rust"
readme = "README.md"
[dependencies]
serde_json = "1.0"
trust-dns-resolver = "0.20"
reqwest = { version = "0.11.9", default-features = false, features = ["blocking", "json", "multipart"] }
serde_derive = "1.0.130"
tokio = "1.19.2"
[dependencies.serde]
version = "1.0"
features = ["derive"]
[features]
default = ["reqwest/default-tls", "trust-dns-resolver/dns-over-native-tls"]
# file: Cargo.toml   // --- end

