ARTICLE:

The Stripe FRONT: CODE: =  BACKEND Flow new recommended:

    FRONTEND: "Subscribe" button should call a "generate-stripe-checkout" endpoint onClick
    USER: Clicks "subscribe" button on your app
    BACKEND: Create a Stripe customer
    BACKEND: Store binding between Stripe's customerId and your app's userId
    BACKEND: Create a "checkout session" for the user
        With the return URL set to a dedicated /success route in your app
    USER: Makes payment, subscribes, redirects back to /success
    FRONTEND: On load, triggers a syncAfterSuccess function on backend (hit an API, server action, rsc on load, whatever)
    BACKEND: Uses userId to get Stripe customerId from KV
    BACKEND: Calls syncStripeDataToKV with customerId
    FRONTEND: After sync succeeds, redirects user to wherever you want them to be :)
    BACKEND: On all relevant events, calls syncStripeDataToKV with customerId

Checkout flow:

export async function GET(req: Request) {
  const user = auth(req);

  // Get the stripeCustomerId from your KV store
  let stripeCustomerId = await kv.get(stripe:user:${user.id});

  // Create a new Stripe customer if this user doesn't have one
  if (!stripeCustomerId) {
    const newCustomer = await stripe.customers.create({
      email: user.email,
      metadata: {
        userId: user.id, // DO NOT FORGET THIS
      },
    });

    // Store the relation between userId and stripeCustomerId in your KV
    await kv.set(stripe:user:${user.id}, newCustomer.id);
    stripeCustomerId = newCustomer.id;
  }

  // ALWAYS create a checkout with a stripeCustomerId. They should enforce this.
  const checkout = await stripe.checkout.sessions.create({
    customer: stripeCustomerId,
    success_url: "https://t3.chat/success",
    ...
  });

syncStripeDataToKV:


implementation will vary based on doing subscriptions or one-time purchases. The example below is with subcriptions

// The contents of this function should probably be wrapped in a try/catch
export async function syncStripeDataToKV(customerId: string) {
  // Fetch latest subscription data from Stripe
  const subscriptions = await stripe.subscriptions.list({
    customer: customerId,
    limit: 1,
    status: "all",
    expand: ["data.default_payment_method"],
  });

  if (subscriptions.data.length === 0) {
    const subData = { status: "none" };
    await kv.set(stripe:customer:${customerId}, subData);
    return subData;
  }

  // If a user can have multiple subscriptions, that's your problem
  const subscription = subscriptions.data[0];

  // Store complete subscription state
  const subData = {
    subscriptionId: subscription.id,
    status: subscription.status,
    priceId: subscription.items.data[0].price.id,
    currentPeriodEnd: subscription.current_period_end,
    currentPeriodStart: subscription.current_period_start,
    cancelAtPeriodEnd: subscription.cancel_at_period_end,
    paymentMethod:
      subscription.default_payment_method &&
      typeof subscription.default_payment_method !== "string"
        ? {
            brand: subscription.default_payment_method.card?.brand ?? null,
            last4: subscription.default_payment_method.card?.last4 ?? null,
          }
        : null,
  };

  // Store the data in your KV
  await kv.set(stripe:customer:${customerId}, subData);
  return subData;
}

/success endpoint


export async function GET(req: Request) {
  const user = auth(req);
  const stripeCustomerId = await kv.get(stripe:user:${user.id});
  if (!stripeCustomerId) {
    return redirect("/");
  }

  await syncStripeDataToKV(stripeCustomerId);
  return redirect("/");
}


Do NOT use CHECKOUT_SESSION_ID  instead have a SINGLE syncStripeDataToKV function

/api/stripe (The Webhook)

export async function POST(req: Request) {
  const body = await req.text();
  const signature = (await headers()).get("Stripe-Signature");

  if (!signature) return NextResponse.json({}, { status: 400 });

  async function doEventProcessing() {
    if (typeof signature !== "string") {
      throw new Error("[STRIPE HOOK] Header isn't a string???");
    }

    const event = stripe.webhooks.constructEvent(
      body,
      signature,
      process.env.STRIPE_WEBHOOK_SECRET!
    );

    waitUntil(processEvent(event));
  }

  const { error } = await tryCatch(doEventProcessing());

  if (error) {
    console.error("[STRIPE HOOK] Error processing event", error);
  }

  return NextResponse.json({ received: true });
}

Note

If you are using Next.js Pages Router, make sure you turn this on (bodyParser below) . Stripe expects the body to be "untouched" so it can verify the signature.

export const config = {
  api: {
    bodyParser: false,
  },
};

processEvent:

This is the function called in the endpoint that actually takes the Stripe event and updates the KV.

async function processEvent(event: Stripe.Event) {
  // Skip processing if the event isn't one I'm tracking (list of all events below)
  if (!allowedEvents.includes(event.type)) return;

  // All the events I track have a customerId
  const { customer: customerId } = event?.data?.object as {
    customer: string; // Sadly TypeScript does not know this
  };

  // This helps make it typesafe and also lets me know if my assumption is wrong
  if (typeof customerId !== "string") {
    throw new Error(
      [STRIPE HOOK][CANCER] ID isn't string.\nEvent type: ${event.type}
    );
  }

  return await syncStripeDataToKV(customerId);
}



Events I Track:

const allowedEvents: Stripe.Event.Type[] = [
  "checkout.session.completed",
  "customer.subscription.created",
  "customer.subscription.updated",
  "customer.subscription.deleted",
  "customer.subscription.paused",
  "customer.subscription.resumed",
  "customer.subscription.pending_update_applied",
  "customer.subscription.pending_update_expired",
  "customer.subscription.trial_will_end",
  "invoice.paid",
  "invoice.payment_failed",
  "invoice.payment_action_required",
  "invoice.upcoming",
  "invoice.marked_uncollectible",
  "invoice.payment_succeeded",
  "payment_intent.succeeded",
  "payment_intent.payment_failed",
  "payment_intent.canceled",
];

Custom Stripe subscription type

export type STRIPE_SUB_CACHE =
  | {
      subscriptionId: string | null;
      status: Stripe.Subscription.Status;
      priceId: string | null;
      currentPeriodStart: number | null;
      currentPeriodEnd: number | null;
      cancelAtPeriodEnd: boolean;
      paymentMethod: {
        brand: string | null; // e.g., "visa", "mastercard"
        last4: string | null; // e.g., "4242"
      } | null;
    }
  | {
      status: "none";
    };



More Pro Tips

 as I remember them.
DISABLE "CASH APP PAY".

ENABLE "Limit customers to one subscription"


# file: src/lib.rs   // --- start
pub mod stripe;
pub mod types;
pub mod client;
pub use client::StripeClient;
# file: src/lib.rs   // --- end

# file: src/stripe.rs   // --- start
pub mod response;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
pub struct Auth {
    pub client: String,
    pub secret: String,
}
impl Auth {
    pub fn new(client: String, secret: String) -> Self {
        return Auth { client, secret };
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
        let url = format!("https://api.stripe.com/v1/balance");
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/balance");
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
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
        let url = format!(
            "https://api.stripe.com/v1/balance_transactions/{}",
            id.clone()
        );
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!(
            "https://api.stripe.com/v1/balance_transactions/{}",
            id.clone()
        );
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    fn list_chunk(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<BalanceTransactions, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/balance_transactions".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/balance_transactions?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<BalanceTransactions>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<BalanceTransactions, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/balance_transactions".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/balance_transactions?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
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
        return Card {
            id: None,
            brand: None,
            last4: None,
            number: None,
            cvc: None,
            network: None,
            country: None,
            exp_month: None,
            exp_year: None,
            fingerprint: None,
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
        return Charge {
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
            statement_descriptor_suffix: None,
        };
    }
    pub async fn async_capture(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let url = format!(
            "https://api.stripe.com/v1/charges/{}/capture",
            self.id.clone().unwrap()
        );
        let request = reqwest::Client::new()
            .post(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_capture_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/charges/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/charges")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/charges/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn capture(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let url = format!(
            "https://api.stripe.com/v1/charges/{}/capture",
            self.id.clone().unwrap()
        );
        let request = reqwest::blocking::Client::new()
            .post(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_capture_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/charges/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/charges")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/charges/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<Charges, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/charges".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/charges?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Charges>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<Charges, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/charges".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/charges?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Charges>().await?;
        return Ok(json);
    }
    fn to_capture_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.receipt_email {
            Some(receipt_email) => params.push(("receipt_email", receipt_email.as_str())),
            None => {}
        }
        match &self.amount {
            Some(amount) => params.push(("amount", amount.as_str())),
            None => {}
        }
        match &self.statement_descriptor {
            Some(statement_descriptor) => {
                params.push(("statement_descriptor", statement_descriptor.as_str()))
            }
            None => {}
        }
        match &self.statement_descriptor_suffix {
            Some(statement_descriptor_suffix) => params.push((
                "statement_descriptor_suffix",
                statement_descriptor_suffix.as_str(),
            )),
            None => {}
        }
        return params;
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.customer {
            Some(customer) => params.push(("customer", customer.as_str())),
            None => {}
        }
        match &self.description {
            Some(description) => params.push(("description", description.as_str())),
            None => {}
        }
        match &self.receipt_email {
            Some(receipt_email) => params.push(("receipt_email", receipt_email.as_str())),
            None => {}
        }
        match &self.amount {
            Some(amount) => params.push(("amount", amount.as_str())),
            None => {}
        }
        match &self.currency {
            Some(currency) => params.push(("currency", currency.as_str())),
            None => {}
        }
        match &self.source {
            Some(source) => params.push(("source", source.as_str())),
            None => {}
        }
        match &self.statement_descriptor {
            Some(statement_descriptor) => {
                params.push(("statement_descriptor", statement_descriptor.as_str()))
            }
            None => {}
        }
        match &self.statement_descriptor_suffix {
            Some(statement_descriptor_suffix) => params.push((
                "statement_descriptor_suffix",
                statement_descriptor_suffix.as_str(),
            )),
            None => {}
        }
        return params;
    }
}
pub struct CheckoutSession {
    pub customer: Option<String>,
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub mode: Option<String>,
    pub line_items: Option<Vec<LineItem>>,
    pub url: Option<String>,
}
impl CheckoutSession {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn async_post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let client = reqwest::Client::new();
        let res = client
            .post("https://api.stripe.com/v1/checkout/sessions")
            .basic_auth(creds.client, Some(creds.secret))
            .form(self)
            .send()
            .await?
            .json::<Self>()
            .await?;
        Ok(res)
    }
}
pub struct LineItem {
    pub price: Option<String>,
    pub quantity: Option<u32>,
}
impl LineItem {
    pub fn new() -> Self {
        Self::default()
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
    pub metadata: Option<HashMap<String, String>>,
    pub invoice_prefix: Option<String>,
    pub livemode: Option<bool>,
    pub name: Option<String>,
    pub next_invoice_sequence: Option<i64>,
    pub phone: Option<String>,
    pub tax_exempt: Option<String>,
}
impl Customer {
    pub fn new() -> Self {
        return Customer {
            id: None,
            object: None,
            balance: None,
            created: None,
            currency: None,
            default_source: None,
            payment_method: None,
            delinquent: None,
            description: None,
            metadata: None,
            email: None,
            invoice_prefix: None,
            livemode: None,
            name: None,
            next_invoice_sequence: None,
            phone: None,
            tax_exempt: None,
        };
    }
    pub async fn async_delete(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::Client::new()
            .delete(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_invoices(
        creds: Auth,
        customer_id: String,
    ) -> Result<Vec<crate::stripe::response::Invoice>, reqwest::Error> {
        let mut objects: Vec<crate::stripe::response::Invoice> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::get_invoices_chunk(
                creds.clone(),
                customer_id.clone(),
                starting_after.clone(),
            )?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_payment_methods(
        creds: Auth,
        customer_id: String,
        method_type: String,
    ) -> Result<Vec<crate::stripe::response::PaymentMethod>, reqwest::Error> {
        let mut objects: Vec<crate::stripe::response::PaymentMethod> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::get_payment_methods_chunk_async(
                creds.clone(),
                customer_id.clone(),
                method_type.clone(),
                starting_after.clone(),
            )
            .await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/customers")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await;
        match request {
            Ok(req) => {
                let json = req.json::<Self>().await?;
                return Ok(json);
            }
            Err(err) => Err(err),
        }
    }
    pub async fn async_update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/customers/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn delete(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .delete(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(auth: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/customers/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(auth.client.as_str(), Some(auth.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn invoices(
        creds: Auth,
        customer_id: String,
    ) -> Result<Vec<crate::stripe::response::Invoice>, reqwest::Error> {
        let mut objects: Vec<crate::stripe::response::Invoice> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::get_invoices_chunk(
                creds.clone(),
                customer_id.clone(),
                starting_after.clone(),
            )?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn payment_methods(
        creds: Auth,
        customer_id: String,
        method_type: String,
    ) -> Result<Vec<crate::stripe::response::PaymentMethod>, reqwest::Error> {
        let mut objects: Vec<crate::stripe::response::PaymentMethod> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::get_payment_methods_chunk(
                creds.clone(),
                customer_id.clone(),
                method_type.clone(),
                starting_after.clone(),
            )?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/customers")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send();
        match request {
            Ok(req) => {
                let json = req.json::<Self>()?;
                Ok(json)
            }
            Err(err) => Err(err),
        }
    }
    pub fn update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/customers/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn get_invoices_chunk(
        creds: Auth,
        customer_id: String,
        starting_after: Option<String>,
    ) -> Result<crate::stripe::response::Invoices, reqwest::Error> {
        let mut url = format!(
            "https://api.stripe.com/v1/invoices?customer={}",
            customer_id
        );
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/invoices?customer={}&starting_after={}",
                customer_id,
                starting_after.unwrap()
            )
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<crate::stripe::response::Invoices>()?;
        return Ok(json);
    }
    fn list_chunk(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<Customers, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/customers".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/customers?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Customers>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<Customers, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/customers".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/customers?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Customers>().await?;
        return Ok(json);
    }
    fn get_payment_methods_chunk(
        creds: Auth,
        customer_id: String,
        method_type: String,
        starting_after: Option<String>,
    ) -> Result<crate::stripe::response::PaymentMethods, reqwest::Error> {
        let mut url = format!(
            "https://api.stripe.com/v1/customers/{}/payment_methods?type={}",
            customer_id, method_type
        );
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/customers/{}/payment_methods?type={}&starting_after={}",
                customer_id,
                method_type,
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<crate::stripe::response::PaymentMethods>()?;
        return Ok(json);
    }
    async fn get_payment_methods_chunk_async(
        creds: Auth,
        customer_id: String,
        method_type: String,
        starting_after: Option<String>,
    ) -> Result<crate::stripe::response::PaymentMethods, reqwest::Error> {
        let mut url = format!(
            "https://api.stripe.com/v1/customers/{}/payment_methods?type={}",
            customer_id, method_type
        );
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/customers/{}/payment_methods?type={}&starting_after={}",
                customer_id,
                method_type,
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request
            .json::<crate::stripe::response::PaymentMethods>()
            .await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.payment_method {
            Some(payment_method) => params.push(("payment_method", payment_method.as_str())),
            None => {}
        }
        match &self.description {
            Some(description) => params.push(("description", description.as_str())),
            None => {}
        }
        match &self.email {
            Some(email) => params.push(("email", email.as_str())),
            None => {}
        }
        match &self.name {
            Some(name) => params.push(("name", name.as_str())),
            None => {}
        }
        match &self.phone {
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
        return Dispute {
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
            status: None,
        };
    }
    pub async fn async_close(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/disputes/{}/close",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/disputes/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/disputes/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn close(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/disputes/{}/close",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/disputes/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/disputes/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(creds: Auth, starting_after: Option<String>) -> Result<Disputes, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/disputes".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/disputes?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Disputes>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<Disputes, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/disputes".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/disputes?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Disputes>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.evidence {
            Some(evidence) => {
                match &evidence.access_activity_log {
                    Some(access_activity_log) => {
                        params.push((
                            "evidence[access_activity_log]",
                            access_activity_log.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.billing_address {
                    Some(billing_address) => {
                        params.push(("evidence[billing_address]", billing_address.as_str()));
                    }
                    None => {}
                }
                match &evidence.cancellation_policy {
                    Some(cancellation_policy) => {
                        params.push((
                            "evidence[cancellation_policy]",
                            cancellation_policy.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.cancellation_policy_disclosure {
                    Some(cancellation_policy_disclosure) => {
                        params.push((
                            "evidence[cancellation_policy_disclosure]",
                            cancellation_policy_disclosure.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.cancellation_rebuttal {
                    Some(cancellation_rebuttal) => {
                        params.push((
                            "evidence[cancellation_rebuttal]",
                            cancellation_rebuttal.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.customer_communication {
                    Some(customer_communication) => {
                        params.push((
                            "evidence[customer_communication]",
                            customer_communication.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.customer_email_address {
                    Some(customer_email_address) => {
                        params.push((
                            "evidence[customer_email_address]",
                            customer_email_address.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.customer_name {
                    Some(customer_name) => {
                        params.push(("evidence[customer_name]", customer_name.as_str()));
                    }
                    None => {}
                }
                match &evidence.customer_purchase_ip {
                    Some(customer_purchase_ip) => {
                        params.push((
                            "evidence[customer_purchase_ip]",
                            customer_purchase_ip.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.customer_signature {
                    Some(customer_signature) => {
                        params.push(("evidence[customer_signature]", customer_signature.as_str()));
                    }
                    None => {}
                }
                match &evidence.duplicate_charge_documentation {
                    Some(duplicate_charge_documentation) => {
                        params.push((
                            "evidence[duplicate_charge_documentation]",
                            duplicate_charge_documentation.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.duplicate_charge_explanation {
                    Some(duplicate_charge_explanation) => {
                        params.push((
                            "evidence[duplicate_charge_explanation]",
                            duplicate_charge_explanation.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.duplicate_charge_id {
                    Some(duplicate_charge_id) => {
                        params.push((
                            "evidence[duplicate_charge_id]",
                            duplicate_charge_id.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.product_description {
                    Some(product_description) => {
                        params.push((
                            "evidence[product_description]",
                            product_description.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.receipt {
                    Some(receipt) => {
                        params.push(("evidence[receipt]", receipt.as_str()));
                    }
                    None => {}
                }
                match &evidence.refund_policy {
                    Some(refund_policy) => {
                        params.push(("evidence[refund_policy]", refund_policy.as_str()));
                    }
                    None => {}
                }
                match &evidence.refund_policy_disclosure {
                    Some(refund_policy_disclosure) => {
                        params.push((
                            "evidence[refund_policy_disclosure]",
                            refund_policy_disclosure.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.refund_refusal_explanation {
                    Some(refund_refusal_explanation) => {
                        params.push((
                            "evidence[refund_refusal_explanation]",
                            refund_refusal_explanation.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.service_date {
                    Some(service_date) => {
                        params.push(("evidence[service_date]", service_date.as_str()));
                    }
                    None => {}
                }
                match &evidence.service_documentation {
                    Some(service_documentation) => {
                        params.push((
                            "evidence[service_documentation]",
                            service_documentation.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.shipping_address {
                    Some(shipping_address) => {
                        params.push(("evidence[shipping_address]", shipping_address.as_str()));
                    }
                    None => {}
                }
                match &evidence.shipping_carrier {
                    Some(shipping_carrier) => {
                        params.push(("evidence[shipping_carrier]", shipping_carrier.as_str()));
                    }
                    None => {}
                }
                match &evidence.shipping_date {
                    Some(shipping_date) => {
                        params.push(("evidence[shipping_date]", shipping_date.as_str()));
                    }
                    None => {}
                }
                match &evidence.shipping_documentation {
                    Some(shipping_documentation) => {
                        params.push((
                            "evidence[shipping_documentation]",
                            shipping_documentation.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.shipping_tracking_number {
                    Some(shipping_tracking_number) => {
                        params.push((
                            "evidence[shipping_tracking_number]",
                            shipping_tracking_number.as_str(),
                        ));
                    }
                    None => {}
                }
                match &evidence.uncategorized_file {
                    Some(uncategorized_file) => {
                        params.push(("evidence[uncategorized_file]", uncategorized_file.as_str()));
                    }
                    None => {}
                }
                match &evidence.uncategorized_text {
                    Some(uncategorized_text) => {
                        params.push(("evidence[uncategorized_text]", uncategorized_text.as_str()));
                    }
                    None => {}
                }
            }
            None => {}
        }
        match &self.submit {
            Some(submit) => {
                if *submit {
                    params.push(("submit", "true"));
                } else {
                    params.push(("submit", "false"));
                }
            }
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
        let url = format!("https://api.stripe.com/v1/events/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/events/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
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
            url = format!(
                "https://api.stripe.com/v1/events?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Events>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<Events, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/events".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/events?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
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
        return File {
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
        let url = format!("https://api.stripe.com/v1/files/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let form = self.to_multipart_form_async().await;
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/files")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .multipart(form)
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let form = self.to_multipart_form();
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/files")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .multipart(form)
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/files/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
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
            url = format!(
                "https://api.stripe.com/v1/files?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Files>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<Files, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/files".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/files?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Files>().await?;
        return Ok(json);
    }
    fn to_multipart_form(&self) -> reqwest::blocking::multipart::Form {
        let mut form = reqwest::blocking::multipart::Form::new();
        match &self.purpose {
            Some(purpose) => {
                form = form.text("purpose", purpose.clone());
            }
            None => {}
        }
        match &self.file {
            Some(file) => {
                let part = reqwest::blocking::multipart::Part::bytes(file.clone());
                form = form.part("file", part);
            }
            None => {}
        }
        return form;
    }
    async fn to_multipart_form_async(&self) -> reqwest::multipart::Form {
        let mut form = reqwest::multipart::Form::new();
        match &self.purpose {
            Some(purpose) => {
                form = form.text("purpose", purpose.clone());
            }
            None => {}
        }
        match &self.file {
            Some(file) => {
                let part = reqwest::multipart::Part::bytes(file.clone());
                form = form.part("file", part);
            }
            None => {}
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
    pub url: Option<String>,
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
            url: None,
        };
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/file_links")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/file_links/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/file_links")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/file_links/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<FileLinks, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/file_links".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/file_links?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<FileLinks>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<FileLinks, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/file_links".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/file_links?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<FileLinks>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.file {
            Some(file) => params.push(("file", file.as_str())),
            None => {}
        }
        match &self.link_expires_at {
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
        return Invoice {
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
            total: None,
        };
    }
    pub async fn async_get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/invoices/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_list(
        creds: Auth,
        status: Option<String>,
        customer: Option<String>,
    ) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(
                creds.clone(),
                starting_after,
                status.clone(),
                customer.clone(),
            )
            .await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub async fn async_post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/invoices")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub async fn async_update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/invoices/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/invoices/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn list(
        creds: Auth,
        status: Option<String>,
        customer: Option<String>,
    ) -> Result<Vec<Self>, reqwest::Error> {
        let mut objects: Vec<Self> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(
                creds.clone(),
                starting_after,
                status.clone(),
                customer.clone(),
            )?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone().unwrap());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/invoices")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    pub fn update(&self, creds: Auth) -> Result<Self, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/invoices/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
    fn list_chunk(
        creds: Auth,
        starting_after: Option<String>,
        status: Option<String>,
        customer: Option<String>,
    ) -> Result<Invoices, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/invoices".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/invoices?starting_after={}",
                starting_after.unwrap()
            );
        }
        if status.is_some() {
            if url.contains("?") {
                url = format!("{}{}={}", url, "&status", status.unwrap());
            } else {
                url = format!("{}{}={}", url, "?status", status.unwrap());
            }
        }
        if customer.is_some() {
            if url.contains("?") {
                url = format!("{}{}={}", url, "&customer", customer.unwrap());
            } else {
                url = format!("{}{}={}", url, "?customer", customer.unwrap());
            }
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Invoices>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
        status: Option<String>,
        customer: Option<String>,
    ) -> Result<Invoices, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/invoices".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/invoices?starting_after={}",
                starting_after.unwrap()
            );
        }
        if status.is_some() {
            if url.contains("?") {
                url = format!("{}{}={}", url, "&status", status.unwrap());
            } else {
                url = format!("{}{}={}", url, "?status", status.unwrap());
            }
        }
        if customer.is_some() {
            if url.contains("?") {
                url = format!("{}{}={}", url, "&customer", customer.unwrap());
            } else {
                url = format!("{}{}={}", url, "?customer", customer.unwrap());
            }
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Invoices>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.customer {
            Some(customer) => params.push(("customer", customer.as_str())),
            None => {}
        }
        match &self.collection_method {
            Some(collection_method) => {
                params.push(("collection_method", collection_method.as_str()))
            }
            None => {}
        }
        match &self.description {
            Some(description) => params.push(("description", description.as_str())),
            None => {}
        }
        match &self.subscription {
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
        let url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<Self>().await?;
        return Ok(json);
    }
    pub fn get(creds: Auth, id: String) -> Result<Self, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/file_links/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<Self>()?;
        return Ok(json);
    }
}
pub struct PaymentMethod {
    pub id: Option<String>,
    pub method_type: Option<String>,
    pub created: Option<String>,
    pub customer: Option<String>,
    pub livemode: Option<bool>,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub billing_details: Option<crate::stripe::response::BillingDetails>,
    pub card: Option<Card>,
    pub type_field: Option<String>,
}
impl PaymentMethod {
    pub fn new() -> Self {
        return PaymentMethod {
            id: None,
            method_type: None,
            created: None,
            customer: None,
            livemode: None,
            name: None,
            phone: None,
            billing_details: None,
            card: None,
            type_field: None,
        };
    }
    pub fn attach(&self, customer: Customer, creds: Auth) -> Result<bool, reqwest::Error> {
        match &self.id {
            Some(id) => match &customer.id {
                Some(cust_id) => {
                    let url = format!(
                        "https://api.stripe.com/v1/payment_methods/{}/attach",
                        id.clone()
                    );
                    let params = [("customer", cust_id.as_str())];
                    let _request = reqwest::blocking::Client::new()
                        .post(url)
                        .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
                        .form(&params)
                        .send()?;
                    return Ok(true);
                }
                None => return Ok(false),
            },
            None => return Ok(false),
        }
    }
    pub fn get(
        creds: Auth,
        id: String,
    ) -> Result<crate::stripe::response::PaymentMethod, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/payment_methods/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send();
        match request {
            Ok(req) => {
                let json = req
                    .json::<crate::stripe::response::PaymentMethod>()
                    .unwrap();
                return Ok(json);
            }
            Err(err) => Err(err),
        }
    }
    pub fn post(&self, creds: Auth) -> Result<PaymentMethod, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/payment_methods")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send();
        match request {
            Ok(req) => {
                let mut plan = self.clone();
                let json = req.json::<crate::stripe::response::PaymentMethod>()?;
                plan.id = Some(json.id);
                Ok(plan)
            }
            Err(err) => Err(err),
        }
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.method_type {
            Some(method_type) => params.push(("type", method_type.as_str())),
            None => {}
        }
        match &self.card {
            Some(card) => {
                match &card.number {
                    Some(number) => params.push(("card[number]", number.as_str())),
                    None => {}
                }
                match &card.exp_month {
                    Some(exp_month) => params.push(("card[exp_month]", exp_month.as_str())),
                    None => {}
                }
                match &card.exp_year {
                    Some(exp_year) => params.push(("card[exp_year]", exp_year.as_str())),
                    None => {}
                }
                match &card.cvc {
                    Some(cvc) => params.push(("card[cvc]", cvc.as_str())),
                    None => {}
                }
            }
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
            product: None,
        };
    }
    pub async fn async_delete(
        creds: Auth,
        id: String,
    ) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::Client::new()
            .delete(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<crate::stripe::response::Plan>().await?;
        return Ok(json);
    }
    pub async fn async_get(
        auth: Auth,
        id: String,
    ) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(auth.client.as_str(), Some(auth.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<crate::stripe::response::Plan>().await?;
        return Ok(json);
    }
    pub async fn async_list(
        creds: Auth,
    ) -> Result<Vec<crate::stripe::response::Plan>, reqwest::Error> {
        let mut objects: Vec<crate::stripe::response::Plan> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk_async(creds.clone(), starting_after).await?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub async fn async_post(
        &self,
        creds: Auth,
    ) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let request = reqwest::Client::new()
            .post("https://api.stripe.com/v1/plans")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()
            .await?;
        let json = request.json::<crate::stripe::response::Plan>().await?;
        return Ok(json);
    }
    pub fn delete(
        creds: Auth,
        id: String,
    ) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .delete(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<crate::stripe::response::Plan>()?;
        return Ok(json);
    }
    pub fn get(auth: Auth, id: String) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/plans/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(auth.client.as_str(), Some(auth.secret.as_str()))
            .send()?;
        let json = request.json::<crate::stripe::response::Plan>()?;
        return Ok(json);
    }
    pub fn list(creds: Auth) -> Result<Vec<crate::stripe::response::Plan>, reqwest::Error> {
        let mut objects: Vec<crate::stripe::response::Plan> = Vec::new();
        let mut has_more = true;
        let mut starting_after: Option<String> = None;
        while has_more {
            let json = Self::list_chunk(creds.clone(), starting_after)?;
            for json_object in json.data {
                objects.push(json_object);
            }
            has_more = json.has_more;
            starting_after = Some(objects[objects.len() - 1].id.clone());
        }
        return Ok(objects);
    }
    pub fn post(&self, creds: Auth) -> Result<crate::stripe::response::Plan, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/plans")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<crate::stripe::response::Plan>()?;
        return Ok(json);
    }
    fn list_chunk(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<crate::stripe::response::Plans, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/plans".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/plans?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()?;
        let json = request.json::<crate::stripe::response::Plans>()?;
        return Ok(json);
    }
    async fn list_chunk_async(
        creds: Auth,
        starting_after: Option<String>,
    ) -> Result<crate::stripe::response::Plans, reqwest::Error> {
        let mut url = "https://api.stripe.com/v1/plans".to_string();
        if starting_after.is_some() {
            url = format!(
                "https://api.stripe.com/v1/plans?starting_after={}",
                starting_after.unwrap()
            );
        }
        let request = reqwest::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send()
            .await?;
        let json = request.json::<crate::stripe::response::Plans>().await?;
        return Ok(json);
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.amount {
            Some(amount) => params.push(("amount", amount.as_str())),
            None => {}
        }
        match &self.currency {
            Some(currency) => params.push(("currency", currency.as_str())),
            None => {}
        }
        match &self.interval {
            Some(interval) => params.push(("interval", interval.as_str())),
            None => {}
        }
        match &self.product {
            Some(product) => params.push(("product", product.as_str())),
            None => {}
        }
        match &self.active {
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
        return Price {
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
            unit_amount_decimal: None,
        };
    }
    pub fn post(&self, creds: Auth) -> Result<Price, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/prices")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send();
        match request {
            Ok(req) => {
                let mut plan = self.clone();
                let json = req.json::<crate::stripe::response::Price>()?;
                plan.id = Some(json.id);
                Ok(plan)
            }
            Err(err) => Err(err),
        }
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.currency {
            Some(currency) => params.push(("currency", currency.as_str())),
            None => {}
        }
        match &self.unit_amount {
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
    pub price_items: Option<Vec<String>>,
}
impl Subscription {
    pub fn new() -> Self {
        return Subscription {
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
            status: None,
        };
    }
    pub fn cancel(
        creds: Auth,
        id: String,
    ) -> Result<crate::stripe::response::Subscription, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/subscriptions/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .delete(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send();
        match request {
            Ok(req) => {
                let json = req.json::<crate::stripe::response::Subscription>().unwrap();
                return Ok(json);
            }
            Err(err) => Err(err),
        }
    }
    pub fn get(
        creds: Auth,
        id: String,
    ) -> Result<crate::stripe::response::Subscription, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/subscriptions/{}", id.clone());
        let request = reqwest::blocking::Client::new()
            .get(url)
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .send();
        match request {
            Ok(req) => {
                let json = req.json::<crate::stripe::response::Subscription>().unwrap();
                return Ok(json);
            }
            Err(err) => Err(err),
        }
    }
    pub fn update(
        &self,
        creds: Auth,
    ) -> Result<crate::stripe::response::Subscription, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post(format!(
                "https://api.stripe.com/v1/subscriptions/{}",
                self.clone().id.unwrap()
            ))
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send();
        match request {
            Ok(req) => {
                let json = req.json::<crate::stripe::response::Subscription>()?;
                Ok(json)
            }
            Err(err) => Err(err),
        }
    }
    pub fn post(&self, creds: Auth) -> Result<Subscription, reqwest::Error> {
        let request = reqwest::blocking::Client::new()
            .post("https://api.stripe.com/v1/subscriptions")
            .basic_auth(creds.client.as_str(), Some(creds.secret.as_str()))
            .form(&self.to_params())
            .send()?;
        let json = request.json::<Subscription>()?;
        Ok(json)
    }
    fn to_params(&self) -> Vec<(&str, &str)> {
        let mut params = vec![];
        match &self.customer {
            Some(customer) => params.push(("customer", customer.as_str())),
            None => {}
        }
        match &self.default_payment_method {
            Some(default_payment_method) => {
                params.push(("default_payment_method", default_payment_method.as_str()))
            }
            None => {}
        }
        match &self.price_items {
            Some(price_items) => {
                let mut ii = 0;
                for item in price_items {
                    if ii < 20 {
                        if ii == 0 {
                            params.push(("items[0][price]", item.as_str()));
                        }
                        ii += 1;
                    }
                }
            }
            None => {}
        }
        return params;
    }
}
impl Subscription {
    pub async fn list_for_customer(customer_id: &str, creds: Auth) -> Result<Vec<Self>, reqwest::Error> {
        let url = format!("https://api.stripe.com/v1/subscriptions?customer={}", customer_id);
        let client = reqwest::Client::new();
        let res = client
            .get(&url)
            .basic_auth(creds.client, Some(creds.secret))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        let subs = res["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .collect();
        Ok(subs)
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
pub struct FraudDetails {}
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
        return Evidence {
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
            uncategorized_text: None,
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

# file: src/client.rs   // --- start
use crate::stripe::Auth;
use dotenvy::dotenv;
use std::{
    env as stdenv,
};
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
# file: src/client.rs   // --- end

# file: src/types/subscription.rs   // --- start
use serde::{Deserialize,
};
pub struct StripeSubscription {
    pub id: String,
    pub status: String,
    pub current_period_start: u64,
    pub current_period_end: u64,
    pub cancel_at_period_end: bool,
    pub items: Items,
}
pub struct Items {
    pub data: Vec<ItemData>,
}
pub struct ItemData {
    pub price: Price,
}
pub struct Price {
    pub id: String,
}
# file: src/types/subscription.rs   // --- end

# file: src/types/mod.rs   // --- start
pub mod subscription;
# file: src/types/mod.rs   // --- end

# file: src/kv_store.rs   // --- start
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;
pub struct KVStore {
    inner: Arc<Mutex<HashMap<String, String>>>,
}
impl KVStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn set(&self, key: &str, value: String) {
        let mut store = self.inner.lock().await;
        store.insert(key.to_string(), value);
    }
    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.inner.lock().await;
        store.get(key).cloned()
    }
}
# file: src/kv_store.rs   // --- end

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
pub struct Metadata {}
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

# file: src/stripe/checkout_session.rs   // --- start
use serde::{Deserialize, Serialize};
pub struct CheckoutSession {
    pub customer: Option<String>,
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
    pub mode: Option<String>,
    pub line_items: Option<Vec<LineItem>>,
    pub url: Option<String>,
}
pub struct LineItem {
    pub price: Option<String>,
    pub quantity: Option<u32>,
}
# file: src/stripe/checkout_session.rs   // --- end

# file: src/cors.rs   // --- start
use super::*;
pub fn check_env_cors() {
    let current_dir = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_cors_path = current_dir.join(".env_cors");
    if env_cors_path.exists() {
        info!(".env_cors file found at: {}", env_cors_path.display());
    } else {
        error!(
            ".env_cors file not found. Expected it at: {}",
            env_cors_path.display()
        );
    }
}
pub fn load_and_validate_cors_origins(path: &str) -> actix_web::Result<IOVec<IOString>, IOError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let mut origins = Vec::new();
    let mut all_lines_failed = true;
    for line in buf_reader.lines() {
        let line = line?;
        match line.parse::<Uri>() {
            Ok(_) => {
                origins.push(line);
                all_lines_failed = false;
            }
            Err(e) => {
                warn!("Invalid URI in CORS configuration: {}", e);
            }
        }
    }
    if all_lines_failed {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            "All CORS lines failed validation.",
        ));
    }
    Ok(origins)
}
# file: src/cors.rs   // --- end

# file: src/env.rs   // --- start
use super::*;
pub fn load_env_file() {
    let current_dir = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_path = current_dir.join(".env");
    if dotenv().is_err() {
        error!(
            ".env file not found. Expected it at: {}",
            env_path.display()
        );
    } else {
        info!(".env loading at: {}", env_path.display());
    }
}
pub fn load_env_var(key: &str, default: &str) -> String {
    stdenv::var(key).unwrap_or_else(|_| {
        if default == "/home/zeus" {
            stdenv::var("HOME").unwrap_or_else(|_| "/home".to_string())
        } else {
            default.to_string()
        }
    })
}
# file: src/env.rs   // --- end

# file: chatgipity/adding_feature.rs   // --- start
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
pub mod stripe;
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
# file: chatgipity/adding_feature.rs   // --- end

# file: tests/integration.rs   // --- start
use dotenvy::dotenv;
async fn test_create_customer() {
    dotenv().ok();
}
# file: tests/integration.rs   // --- end

# file: tests/stripe_tests.rs   // --- start
async fn test_env_load_and_client() {
    let client = justpaystripe::StripeClient::new();
    assert!(client.api_key.starts_with("sk_"));
}
# file: tests/stripe_tests.rs   // --- end

# file: examples/server/cors.rs   // --- start
use super::*;
pub fn check_env_cors() {
    let current_dir = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_cors_path = current_dir.join(".env_cors");
    if env_cors_path.exists() {
        info!(".env_cors file found at: {}", env_cors_path.display());
    } else {
        error!(
            ".env_cors file not found. Expected it at: {}",
            env_cors_path.display()
        );
    }
}
pub fn load_and_validate_cors_origins(path: &str) -> actix_web::Result<IOVec<IOString>, IOError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let mut origins = Vec::new();
    let mut all_lines_failed = true;
    for line in buf_reader.lines() {
        let line = line?;
        match line.parse::<Uri>() {
            Ok(_) => {
                origins.push(line);
                all_lines_failed = false;
            }
            Err(e) => {
                warn!("Invalid URI in CORS configuration: {}", e);
            }
        }
    }
    if all_lines_failed {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            "All CORS lines failed validation.",
        ));
    }
    Ok(origins)
}
# file: examples/server/cors.rs   // --- end

# file: examples/server/env.rs   // --- start
use super::*;
pub fn load_env_file() {
    let current_dir = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_path = current_dir.join(".env");
    if dotenv().is_err() {
        error!(
            ".env file not found. Expected it at: {}",
            env_path.display()
        );
    } else {
        info!(".env loading at: {}", env_path.display());
    }
}
pub fn load_env_var(key: &str, default: &str) -> String {
    stdenv::var(key).unwrap_or_else(|_| {
        if default == "/home/zeus" {
            stdenv::var("HOME").unwrap_or_else(|_| "/home".to_string())
        } else {
            default.to_string()
        }
    })
}
# file: examples/server/env.rs   // --- end

# file: examples/server/main.rs   // --- start
use actix_cors::Cors;
use dotenvy::*;
use justpaystripe::{
    stripe::{
        Auth,
        Charge,
        Customer,
        CheckoutSession,
        Subscription
    },
    StripeClient,
};
mod logger;
use crate::logger::*;
mod cors;
use crate::cors::*;
mod env;
use crate::env::*;
use actix_web::{
    http::{
        header,
        uri::Uri,
    },
    middleware::Logger as ActixLogger,
    get,
    post,
    web::{
        self,
    },
    App,
    HttpResponse,
    HttpServer,
    Responder,
};
use env_logger::{Builder, Env};
use colored::*;
use log::{debug, error, info, trace, warn};
use serde::{Deserialize,
};
use std::{
    env as stdenv,
    fs::File,
    io::{
        BufRead,
        BufReader,
        Error as IOError,
        ErrorKind,
        Write,
    },
    path::{
        Path,
    },
    process::{
        exit,
        id as process_id,
        Command,
    },
    string::String as IOString,
    sync::Arc,
    vec::Vec as IOVec,
    collections::HashMap,
};
use chrono::{
    Local,
};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
static KV: Lazy<KVStore> = Lazy::new(KVStore::new);
const VERSION: &str = stdenv!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = stdenv!("CARGO_PKG_DESCRIPTION");
const NAME: &str = stdenv!("CARGO_PKG_NAME");
pub struct KVStore {
    inner: Arc<Mutex<HashMap<String, String>>>,
}
impl KVStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    pub async fn set(&self, key: &str, value: String) {
        let mut store = self.inner.lock().await;
        store.insert(key.to_string(), value);
    }
    pub async fn get(&self, key: &str) -> Option<String> {
        let store = self.inner.lock().await;
        store.get(key).cloned()
    }
}
async fn health() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
async fn post_customer(item: web::Json<Customer>) -> impl Responder {
    let creds = StripeClient::new().into();
    match item.0.async_post(creds).await {
        Ok(cust) => HttpResponse::Ok().json(cust),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
async fn post_charge(item: web::Json<Charge>) -> impl Responder {
    let creds = StripeClient::new().into();
    match item.0.async_post(creds).await {
        Ok(charge) => HttpResponse::Ok().json(charge),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}
struct User {
    id: String,
    email: String,
}
async fn generate_checkout(user: web::Json<User>) -> impl Responder {
    let creds: Auth = StripeClient::new().into();
    let kv = KV.clone();
    let kv_key = format!("stripe:user:{}", user.id);
    let customer_id = kv.get(&kv_key).await;
    let customer_id = match customer_id {
        Some(cid) => cid,
        None => {
            let mut customer = Customer::new();
            customer.email = Some(user.email.clone());
            customer.metadata = Some(HashMap::from([("userId".to_string(), user.id.clone())]));
            let created = customer.async_post(creds.clone()).await.unwrap();
            let customer_id = created.id.clone().expect("missing customer id");
            kv.set(&kv_key, customer_id.clone()).await;
            created.id.expect("missing customer id")
        }
    };
    let mut session = CheckoutSession::new();
    session.customer = Some(customer_id);
    session.success_url = Some("http://localhost:3000/success".to_string());
    session.cancel_url = Some("http://localhost:3000/cancel".to_string());
    session.mode = Some("subscription".to_string());
    session.line_items = Some(vec![{
        let mut item = justpaystripe::stripe::LineItem::new();
        item.price = Some("price_abc123".to_string());
        item.quantity = Some(1);
        item
    }]);
    let created_session = session.async_post(creds).await.unwrap();
    HttpResponse::Ok().json(serde_json::json!({ "url": created_session.url }))
}
async fn success() -> impl Responder {
    let user_id = "demo-user";
    let kv = KV.clone();
    let customer_id = kv.get(&format!("stripe:user:{user_id}")).await.unwrap();
    let creds = StripeClient::new().into();
    let sub = Subscription::list_for_customer(&customer_id, creds).await.unwrap();
    kv.set(&format!("stripe:customer:{customer_id}"), format!("{:?}", sub)).await;
    HttpResponse::Ok().body("✅ Subscription synced.")
}
async fn main() -> std::io::Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("💣 Panic occurred: {}", panic_info);
    }));
    let this_script_relative_path = stdenv::args().next().unwrap_or_default();
    let this_script_name = std::path::Path::new(&this_script_relative_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    let this_script_absolute_pathbuf =
        std::env::current_exe().expect("Failed to get the current executable path");
    let this_script_absolute_path = std::path::Path::new(&this_script_absolute_pathbuf);
    setup_logger();
    print_help();
    load_env_file();
    check_env_cors();
    dotenv().ok();
    info!(
        "\x1b[01;35m
        this_script_name
    );
    info!(
        "\x1b[01;35m
        this_script_relative_path
    );
    info!(
        "\x1b[01;35m
        this_script_absolute_path
    );
    info!("PID: {}", std::process::id());
    let target_port = load_env_var("PORT", "8081");
    let target_host = load_env_var("HOST", "127.0.0.1");
    let target_server = format!("{}:{}", target_host, target_port);
    let mut cors_failed = false;
    let mut port_failed = false;
    let mut when_errors_detected = false;
    let allowed_origins = load_and_validate_cors_origins(".env_cors").unwrap_or_else(|e| {
        cors_failed = true;
        error!("Failed to load .env_cors, error: {:?}", e);
        vec![]
    });
    info!("Allowed origins: {:?}", allowed_origins);
    trace!(
        "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
        when_errors_detected,
        cors_failed,
        port_failed
    );
    let cors_origins = match load_and_validate_cors_origins(".env_cors") {
        Ok(origins) => {
            info!("CORS origins loaded successfully.");
            origins
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            cors_failed = true;
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            let pwd = stdenv::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            error!(".env_cors file not found in directory: {:?}", pwd.display());
            exit(1);
        }
        Err(e) => {
            cors_failed = true;
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Failed to load or validate all CORS origins: {}", e);
            exit(1);
        }
    };
    info!("Allowed cors_origins: {:?}", cors_origins);
    let lsof_available = Command::new("sh")
        .arg("-c")
        .arg("which lsof")
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);
    if !lsof_available {
        info!("`lsof` is not available. Please install `lsof` for more detailed diagnostics.");
        if std::net::TcpListener::bind(format!("{}", target_server)).is_err() {
            port_failed = true;
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Port {} is already in use.", target_port);
            exit(52);
        }
    }
    match std::net::TcpListener::bind(format!("{}", target_server)) {
        Ok(_) => {
        }
        Err(_) => {
            port_failed = true;
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Port {} is already in use.", target_port);
            if lsof_available {
                let output = Command::new("sh")
                    .arg("-c")
                    .arg(format!("lsof -i :{} -t -sTCP:LISTEN", target_port))
                    .output();
                match output {
                    Ok(output) if !output.stdout.is_empty() => {
                        let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        info!("PID using port {}: {}", target_port, pid);
                        let cmd = format!("ps -o user= -o comm= -p {}", pid);
                        if let Ok(output) = Command::new("sh").arg("-c").arg(cmd).output() {
                            info!(
                                "Process details: {}",
                                String::from_utf8_lossy(&output.stdout)
                            );
                        }
                    }
                    _ => error!("Could not determine the process using port {}", target_port),
                }
            }
            exit(52);
        }
    }
    when_errors_detected = cors_failed || port_failed;
    trace!(
        "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
        when_errors_detected,
        cors_failed,
        port_failed
    );
    let server_pid = process_id();
    info!("Server starting with PID: {}", server_pid);
    if when_errors_detected {
        error!("Server start-up failed due to errors.");
        exit(1);
    }
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .supports_credentials()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600);
        trace!("1 cors: {:?}", cors);
        let cors = Cors::permissive();
        trace!("2 cors: {:?}", cors);
        App::new()
            .wrap(ActixLogger::default())
            .wrap(cors)
            .configure(|cfg| {
                cfg.route("/health", web::get().to(health))
                    .route("/customer", web::post().to(post_customer))
                    .route("/charge", web::post().to(post_charge));
            })
    })
    .bind(format!("{}", target_server))?
    .run();
    info!("Server running at http://{} ", format!("{}", target_server));
    trace!(
        "when_errors_detected: {:?} cors_failed:{:?} port_failed:{:?}",
        when_errors_detected,
        cors_failed,
        port_failed
    );
    let execution = server.await;
    info!("Worker stopped with PID: {}", process_id());
    if let Err(e) = execution {
        trace!(
            "when_errors_detected: {:?} cors_failed:{:?} port_failed:{:?}",
            when_errors_detected,
            cors_failed,
            port_failed
        );
        error!("💥 Failed to start the server: {:?}", e);
        return Err(e);
    }
    if port_failed {
        error!("Port {} is already in use.", format!("{}", target_server));
        exit(1);
    }
    Ok(())
}
# file: examples/server/main.rs   // --- end

# file: stripe_evolved_project_fixed/main.rs   // --- start
use notify::{Watcher, RecursiveMode, recommended_watcher, EventKind};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::process::{Command, Stdio};
use std::env;
use std::path::Path;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use chrono::Local;
use env_logger::{Env, Builder};
use colored::*;
use log::{info, error};
const VERSION: &str = env!("CARGO_PKG_VERSION");
fn print_help() {
    let version = VERSION;
    let this_script_relative_path = env::args().next().unwrap_or_default();
    let _this_script_absolute_path = env::current_exe().expect("Failed to get the current executable path");
    let _call_from_absolute_path = env::current_dir().expect("Failed to get the current directory");
    let this_script_name = Path::new(&this_script_relative_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    println!("{} version:{} Usage: {}  --watch <path> --process <script> [--timeout <duration>] [--timeout-process <script>]", this_script_name, version, this_script_name);
    println!("Watches a folder and when a file appears it sends the name as argument to another process. ");
    println!("This one has timer optional which counts time after last file received and triggers choice timeout1 or timeout2");
    println!("\\___ choice timeout 1 is when no timeout-process is passed then it ends the server");
    println!("\\___ choice timeout 2 is when when timeout-process is passed then it triggers it, and keeps going");
    println!("Options:");
    println!("  --help, -h             Show this help message");
    println!("  --watch                Required. Path to the directory to watch");
    println!("  --process              Required. Script to execute when a file is found, with the file path as an argument");
    println!("  --timeout              Optional. Timeout duration for inactivity (e.g., '10ns', '10ms', '10s', '5m')");
    println!("  --timeout-process      Optional. Script to execute when timeout occurs");
}
fn init_logger() {
    let this_script_name = Path::new(&env::args().next().unwrap_or_default())
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    let this_script_name_with_version = format!("{}_{}", this_script_name, VERSION);
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(move |buf, record| {
            let level = match record.level() {
                log::Level::Error => format!("{}", record.level()).red(),
                log::Level::Warn =>  format!(" {}", record.level()).yellow(),
                log::Level::Info =>  format!(" {}", record.level()).green(),
                log::Level::Debug => format!("{}", record.level()).blue(),
                log::Level::Trace => format!("{}", record.level()).purple(),
            };
            writeln!(
                buf,
                "[{} {}]{}: {}",
                this_script_name_with_version.to_string().dimmed(),
                Local::now().format("%Y%m%d %H:%M:%S").to_string().dimmed(),
                level,
                record.args()
            )
        })
        .init();
}
fn parse_duration(duration_str: &str) -> Result<Duration, &'static str> {
    let mut chars = duration_str.chars().peekable();
    let mut num_str = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_digit(10) {
            num_str.push(ch);
            chars.next();
        } else {
            break;
        }
    }
    let num: u64 = num_str.parse().map_err(|_| "Invalid number in duration")?;
    let unit = chars.collect::<String>();
    match unit.as_str() {
        "ns" => Ok(Duration::from_nanos(num)),
        "ms" => Ok(Duration::from_millis(num)),
        "s" => Ok(Duration::from_secs(num)),
        "m" => Ok(Duration::from_secs(num * 60)),
        _ => Err("Invalid duration unit"),
    }
}
fn handle_event(event: notify::Result<notify::Event>, process_script: &str, retry_tx: Sender<String>) {
    if let Ok(event) = event {
        if let EventKind::Create(_) = event.kind {
            for path in event.paths {
                info!("New file detected: {:?}", path);
                if let Err(e) = start_process(process_script, &path) {
                    error!("Failed to start process: {:?}. Retrying...", e);
                    retry_tx.send(path.to_string_lossy().to_string()).expect("Failed to send to retry queue");
                }
            }
        }
    }
}
fn start_process(process_script: &str, path: &Path) -> Result<(), std::io::Error> {
    let mut child = Command::new(process_script)
        .arg(path.to_str().unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);
    for line in stdout_reader.lines() {
        println!("stdout: {}", line.unwrap());
    }
    for line in stderr_reader.lines() {
        println!("stderr: {}", line.unwrap());
    }
    match child.wait() {
        Ok(status) => {
            if status.success() {
                info!("Process succeeded with status: {}", status);
            } else {
                error!("Process failed with status: {}", status);
            }
        },
        Err(e) => error!("Failed to wait on child process: {:?}", e),
    }
    info!("Process triggered: {:?}", process_script);
    Ok(())
}
fn retry_thread(retry_rx: Receiver<String>, process_script: String, retry_delay: Duration) {
    let mut retry_queue: VecDeque<String> = VecDeque::new();
    loop {
        while let Ok(path) = retry_rx.try_recv() {
            retry_queue.push_back(path);
        }
        if let Some(path) = retry_queue.pop_front() {
            thread::sleep(retry_delay);
            info!("Retrying file: {:?}", path);
            if let Err(e) = start_process(&process_script, Path::new(&path)) {
                error!("Retry failed: {:?}. Requeueing...", e);
                retry_queue.push_back(path);
            }
        } else {
            thread::sleep(Duration::from_secs(1));
        }
    }
}
fn timeout_thread(rx: Receiver<notify::Result<notify::Event>>, process_script: String, timeout_duration: Duration, timeout_process_script: Option<String>, retry_tx: Sender<String>) {
    let last_event = Arc::new(Mutex::new(Instant::now()));
    let last_event_clone = Arc::clone(&last_event);
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let elapsed = Instant::now().duration_since(*last_event_clone.lock().unwrap());
            if elapsed >= timeout_duration {
                if let Some(script) = &timeout_process_script {
                    info!("Timeout reached, running timeout process: {}", script);
                    let mut child = Command::new(script)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to start timeout process");
                    let stdout = child.stdout.take().unwrap();
                    let stderr = child.stderr.take().unwrap();
                    let stdout_reader = BufReader::new(stdout);
                    let stderr_reader = BufReader::new(stderr);
                    for line in stdout_reader.lines() {
                        println!("stdout: {}", line.unwrap());
                    }
                    for line in stderr_reader.lines() {
                        println!("stderr: {}", line.unwrap());
                    }
                    match child.wait() {
                        Ok(status) => {
                            if status.success() {
                                info!("Timeout process succeeded with status: {}", status);
                            } else {
                                error!("Timeout process failed with status: {}", status);
                            }
                        },
                        Err(e) => error!("Failed to wait on timeout process: {:?}", e),
                    }
                    *last_event_clone.lock().unwrap() = Instant::now();
                } else {
                    info!("Timeout reached, stopping observer");
                    std::process::exit(0);
                }
            }
        }
    });
    loop {
        match rx.recv() {
            Ok(event) => {
                *last_event.lock().unwrap() = Instant::now();
                handle_event(event, &process_script, retry_tx.clone());
            },
            Err(e) => error!("Watch error: {:?}", e),
        }
    }
}
fn main() {
    init_logger();
    println!("{:?}", std::process::id());
    info!("Starting application");
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) {
        print_help();
        return;
    }
    let watch_index = args.iter().position(|x| x == "--watch");
    let process_index = args.iter().position(|x| x == "--process");
    let timeout_index = args.iter().position(|x| x == "--timeout");
    let timeout_process_index = args.iter().position(|x| x == "--timeout-process");
    if watch_index.is_none() || process_index.is_none() || args.len() <= watch_index.unwrap() + 1 || args.len() <= process_index.unwrap() + 1 {
        error!("Error: Missing required arguments");
        print_help();
        std::process::exit(1);
    }
    let watch_path = &args[watch_index.unwrap() + 1];
    let process_script = args[process_index.unwrap() + 1].clone();
    let timeout_duration = if let Some(index) = timeout_index {
        if args.len() <= index + 1 {
            error!("Error: Missing duration for --timeout");
            print_help();
            std::process::exit(1);
        }
        match parse_duration(&args[index + 1]) {
            Ok(duration) => Some(duration),
            Err(err) => {
                error!("Error: {}", err);
                print_help();
                std::process::exit(1);
            }
        }
    } else {
        None
    };
    let timeout_process_script = if let Some(index) = timeout_process_index {
        if args.len() <= index + 1 {
            error!("Error: Missing script for --timeout-process");
            print_help();
            std::process::exit(1);
        }
        Some(args[index + 1].clone())
    } else {
        None
    };
    if !Path::new(watch_path).is_dir() {
        error!("Error: --watch path is not a directory or cannot be accessed");
        std::process::exit(1);
    }
    if !Path::new(&process_script).is_file() || fs::metadata(&process_script).unwrap().permissions().readonly() {
        error!("Error: --process script is not a file or cannot be executed");
        std::process::exit(1);
    }
    if let Some(script) = &timeout_process_script {
        if !Path::new(script).is_file() || fs::metadata(script).unwrap().permissions().readonly() {
            error!("Error: --timeout-process script is not a file or cannot be executed");
            std::process::exit(1);
        }
    }
    info!("Watching directory: {}", watch_path);
    info!("Process to execute: {}", process_script);
    info!("PID: {}", std::process::id());
    let (tx, rx) = channel();
    let (retry_tx, retry_rx) = channel();
    let mut watcher = recommended_watcher(tx).unwrap();
    watcher.watch(Path::new(watch_path), RecursiveMode::Recursive).unwrap();
    let process_script_clone = process_script.clone();
    thread::spawn(move || {
        retry_thread(retry_rx, process_script_clone, Duration::from_secs(1));
    });
    if let Some(timeout) = timeout_duration {
        let process_script_clone = process_script.clone();
        let timeout_process_script_clone = timeout_process_script.clone();
        let retry_tx_clone = retry_tx.clone();
        let timeout_thread_handle = thread::spawn(move || {
            timeout_thread(rx, process_script_clone, timeout, timeout_process_script_clone, retry_tx_clone);
        });
        timeout_thread_handle.join().expect("Timeout thread panicked");
    } else {
        loop {
            match rx.recv() {
                Ok(event) => handle_event(event, &process_script, retry_tx.clone()),
                Err(e) => error!("Watch error: {:?}", e),
            }
        }
    }
    info!("Exiting application");
}
# file: stripe_evolved_project_fixed/main.rs   // --- end

# file: stripe_evolved_project/main.rs   // --- start
use actix_cors::Cors;
use actix_web::{
    http::{header, uri::Uri, StatusCode},
    middleware::Logger as ActixLogger,
    web,
    App,
    HttpResponse,
    HttpServer,
    Responder,
    HttpRequest,
};
use csv::ReaderBuilder;
use dotenv::dotenv;
use env_logger::{Builder, Env};
use lazy_static::lazy_static;
use ansi_term::Colour::{Blue, Green, Purple, Red, Yellow};
use log::{debug, error, info, trace, warn, Level};
use serde::{Serialize, Deserialize};
use serde_json::json;
use std::{
    env,
    fs::File,
    io::{
        Read, BufRead, BufReader, Error as IOError, ErrorKind, Write},
    path::Path,
    collections::HashMap,
    process::{id as process_id, self, exit, Command, Stdio},
    sync::Mutex,
};
use regex::Regex;
use futures::StreamExt as _;
use chrono::Local;
use colored::*;
struct ApiResponse {
    status: String,
    command: String,
    pid: u32,
    data: String,
    msg: String,
}
const VERSION: &str = env!("CARGO_PKG_VERSION");
lazy_static! {
    static ref PIDS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
}
fn load_env_file() {
    let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_path = current_dir.join(".env");
    if dotenv().is_err() {
        error!(
            ".env file not found. Expected it at: {}",
            env_path.display()
        );
    } else {
        info!(
            ".env loading at: {}",
            env_path.display()
        );
    }
}
fn check_env_cors() {
    let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let env_cors_path = current_dir.join(".env_cors");
    if env_cors_path.exists() {
        info!(".env_cors file found at: {}", env_cors_path.display());
    } else {
        error!(".env_cors file not found. Expected it at: {}", env_cors_path.display());
    }
}
fn load_env_var(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        if default == "/home/agrivero" {
            env::var("HOME").unwrap_or_else(|_| "/home".to_string())
        } else {
            default.to_string()
        }
    })
}
fn sanitize_input(input: &str) -> String {
    let re = Regex::new(r"[^a-zA-Z0-9 _\-.'$%öüä]").unwrap();
    re.replace_all(input, "").to_string()
}
async fn execute_command(command: &str) -> (StatusCode, String, u32) {
    let child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    match child {
        Ok(child) => {
            let pid = child.id();
            let output = child.wait_with_output().expect("failed to wait on child");
            let mut pids = PIDS.lock().unwrap();
            pids.push(pid);
            if output.status.success() {
                (
                    StatusCode::OK,
                    String::from_utf8_lossy(&output.stdout).to_string(),
                    pid,
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    String::from_utf8_lossy(&output.stderr).to_string(),
                    pid,
                )
            }
        }
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string(), 0),
    }
}
async fn run_script(command: &str) -> impl Responder {
    let (status, output, pid) = execute_command(command).await;
    HttpResponse::build(status).json(ApiResponse {
        command: command.to_string(),
        status: status.to_string(),
        pid,
        data: output,
        msg: if status == StatusCode::OK {
            "".to_string()
        } else {
            "Error executing command".to_string()
        },
    })
}
macro_rules! define_action {
    ($fn_name:ident, $script_name:expr) => {
        async fn $fn_name() -> impl Responder {
            let project_path = load_env_var("PROJECT_PATH", "/default/path");
            let command = format!("{}/{}", project_path, $script_name);
            run_script(&command).await
        }
    };
}
define_action!(arduino_start, "arduino_start.sh");
define_action!(arduino_stop, "arduino_stop.sh");
define_action!(arduino_error, "arduino_error.sh");
define_action!(capture_start, "capture_start.sh");
define_action!(capture_kill, "capture_kill.sh");
define_action!(capture_release, "capture_release.sh");
define_action!(analysis_will_start, "analysis_will_start.sh");
define_action!(analysis_will_stop, "analysis_will_stop.sh");
define_action!(analysis_vero_start, "analysis_vero_start.sh");
define_action!(analysis_vero_stop, "analysis_vero_stop.sh");
define_action!(action1, "action1.sh");
define_action!(action2, "action2.sh");
define_action!(action3, "action3.sh");
define_action!(action4, "action4.sh");
define_action!(action5, "action5.sh");
define_action!(action6, "action6.sh");
define_action!(action7, "action7.sh");
define_action!(action8, "action8.sh");
async fn reboot() -> impl Responder {
    let command = "systemctl reboot";
    let (status, output, pid) = execute_command(command).await;
    HttpResponse::build(status).json(ApiResponse {
        command: command.to_string(),
        status: status.to_string(),
        pid: pid,
        data: output,
        msg: if status == StatusCode::OK { "".to_string() } else { "Error executing command".to_string() },
    })
}
fn normalize_header(header: &str) -> String {
    use unidecode::unidecode;
    let header = unidecode(header);
    let header = header.to_lowercase();
    let header = header.replace(|c: char| !c.is_alphanumeric() && c != '_', " ");
    let header = header.split_whitespace().collect::<Vec<_>>().join("_");
    header
}
struct ReportsResponse<T> where T: Serialize {
    command: String,
    status: String,
    pid: u32,
    data: T,
    msg: String,
}
impl<T> ReportsResponse<T> where T: Serialize {
    pub fn new(command: String, status: String, data: T, msg: String) -> ReportsResponse<T> {
        ReportsResponse {
            command,
            status,
            pid: process::id(),
            data,
            msg,
        }
    }
}
async fn reports() -> impl Responder {
    let pwd = env::current_dir().unwrap_or_else(|_| ".".into());
    info!("Current working directory: {:?}", pwd.display());
    let path = pwd.join("reports.csv");
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => {
            error!("Failed to find reports.csv at {:?}", path.display());
            return HttpResponse::BadRequest().json(ApiResponse {
                status: "error".to_string(),
                command: format!("{:?}", path.display()),
                pid: std::process::id(),
                data: "".to_string(),
                msg: format!("Failed to find reports.csv at {:?}", path.display()),
            });
        }
    };
    let mut reader = ReaderBuilder::new().delimiter(b';').from_reader(file);
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(e) => {
                error!("Failed to read headers reports.csv at {:?} error:{:?}", path.display(), e);
                return HttpResponse::InternalServerError().json(ApiResponse {
                status: "error".to_string(),
                command: "".to_string(),
                pid: std::process::id(),
                data: "".to_string(),
                msg: format!("Failed to read headers reports.csv at {:?}, error:{:?}", path.display(), e),
            })
        }
    };
    let mut records = vec![];
    let mut errors = Vec::new();
    let mut row_count = 0;
    let mut error_count = 0;
    for (i, result) in reader.records().enumerate() {
        match result {
            Ok(record) => {
                if record.iter().any(|x| !x.trim().is_empty()) {
                    let record_map: serde_json::Map<String, serde_json::Value> = headers
                        .iter()
                        .zip(record.iter())
                        .map(|(h, v)| {
                            let key = normalize_header(h);
                            let value = if v.contains('|') {
                                json!(v.split('|').map(|s| s.trim()).collect::<Vec<_>>())
                            } else {
                                json!(v.trim())
                            };
                            (key, value)
                        })
                        .collect();
                    records.push(json!(record_map));
                } else {
                    error_count += 1;
                    error!("Invalid row at line {}: Empty or whitespace-only", i + 2);
                    errors.push(format!("Invalid row at line {}: Empty or whitespace-only", i + 2));
                }
                row_count += 1;
            },
            Err(e) => {
                error_count += 1;
                error!("Error reading row at line {}: {}", i + 2, e);
                errors.push(format!("Error reading row at line {}: {}", i + 2, e));
            }
        }
    }
    if row_count == 0 || row_count == error_count {
        error!("All rows failed or file is empty.");
        return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            command: "".to_string(),
            pid: std::process::id(),
            data: "".to_string(),
            msg: "All rows failed or file is empty.".to_string(),
        });
    }
    let response_data = json!({
        "success_count": records.len(),
        "errors_count": errors.len(),
        "records": records,
    });
    let command = "reports.csv".to_string();
    let status = if error_count > 0 { "partial_success" } else { "success" };
    let response = ReportsResponse::new(
        command.to_string(),
        status.to_string(),
        response_data,
        errors.join("\n"),
    );
    HttpResponse::Ok().json(response)
}
fn load_and_validate_cors_origins(path: &str) -> Result<Vec<String>, IOError> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let mut origins = Vec::new();
    let mut all_lines_failed = true;
    for line in buf_reader.lines() {
        let line = line?;
        match line.parse::<Uri>() {
            Ok(_) => {
                origins.push(line);
                all_lines_failed = false;
            }
            Err(e) => {
                warn!("Invalid URI in CORS configuration: {}", e);
            }
        }
    }
    if all_lines_failed {
        return Err(IOError::new(
            ErrorKind::InvalidData,
            "All CORS lines failed validation.",
        ));
    }
    Ok(origins)
}
fn setup_logger() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let this_script_name = Path::new(&env::args().next().unwrap_or_default())
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
                .to_owned();
            let this_script_name_with_version = format!("{}_{}", this_script_name, VERSION);
            let level = record.level();
            let color = match level {
                Level::Error => format!("{}", Red.paint(level.to_string())),
                Level::Warn =>  format!(" {}", Yellow.paint(level.to_string())),
                Level::Info =>  format!(" {}", Green.paint(level.to_string())),
                Level::Debug => format!("{}", Blue.paint(level.to_string())),
                Level::Trace => format!("{}", Purple.paint(level.to_string())),
            };
            writeln!(buf,
                "[{} {}]{}: {}",
                format!("{}", this_script_name_with_version).dimmed(),
                Local::now().format("%Y%m%d %H:%M:%S").to_string().dimmed(),
                color,
                record.args())
        })
        .init();
    info!("test");
    trace!("test");
    debug!("test");
    warn!("test");
    error!("test");
    info!("Logger initialized");
}
async fn list_routes() -> impl Responder {
    dotenv().ok();
    let target_port = load_env_var("PORT", "8081");
    let target_host = load_env_var("HOST", "127.0.0.1");
    let host = format!("{}://{}:{}", "http", target_host, target_port);
    let routes = vec![
        "/arduino/start",
        "/arduino/stop",
        "/arduino/error",
        "/capture/start",
        "/capture/kill",
        "/camera/release",
        "/analysis/will/start",
        "/analysis/will/stop",
        "/analysis/vero/start",
        "/analysis/vero/stop",
        "/reboot",
        "/reports",
        "/action1",
        "/action2",
        "/action3",
        "/action4",
        "/action5",
        "/action6",
        "/action7",
        "/action8",
    ];
    let data: Vec<_> = routes
        .iter()
        .map(|route| {
            json!({
                "relative_url": route,
                "absolute_url": format!("{}{}", host, route),
            })
        })
        .collect();
    HttpResponse::Ok().json(json!({ "data": data }))
}
async fn execute_script(req: HttpRequest, mut body: web::Payload) -> impl Responder {
    let action = sanitize_input(req.match_info().query("tail"));
    let project_path = load_env_var("PROJECT_PATH", "/home/agrivero");
    let mut body_bytes = web::BytesMut::new();
    while let Some(chunk) = body.next().await {
        match chunk {
            Ok(bytes) => body_bytes.extend_from_slice(&bytes),
            Err(_) => return HttpResponse::BadRequest().json(ApiResponse {
                status: "error".to_owned(),
                command: "".to_owned(),
                pid: std::process::id(),
                data: "".to_owned(),
                msg: "Invalid request body".to_owned(),
            }),
        }
    }
    let body_str = sanitize_input(&String::from_utf8(body_bytes.to_vec()).unwrap_or_default());
    let sanitized_action = Regex::new(r"[^a-z0-9]+").unwrap()
        .replace_all(&action, "_")
        .to_string();
    let sanitized_action = Regex::new(r"_+").unwrap()
        .replace_all(&sanitized_action, "_")
        .to_string();
    let query_params = web::Query::<HashMap<String, String>>::from_query(req.query_string()).unwrap();
    let mut args: Vec<String> = vec![];
    for (key, value) in query_params.iter() {
        let formatted_value = if value.contains(' ') {
            format!(r
        } else {
            format!(r
        };
        args.push(formatted_value);
    }
    let bash_script = format!("{}/{}.bash", project_path, sanitized_action);
    let sh_script = format!("{}/{}.sh", project_path, sanitized_action);
    let bash_script_to_run = std::path::Path::new(&bash_script);
    let sh_script_to_run = std::path::Path::new(&sh_script);
    let this_script_absolute_pathbuf = std::env::current_exe().expect("Failed to get the current executable path");
    let this_script_absolute_path = std::path::Path::new(&this_script_absolute_pathbuf);
    info!("\x1b[01;35m checking PATH\x1b[38;5;93m:\x1b[38;5;1m {:?}", this_script_absolute_path);
    let script_to_run = if bash_script_to_run.exists() {
        info!("\x1b[01;35m found script\x1b[38;5;93m:\x1b[38;5;1m {:?}", bash_script_to_run);
        bash_script_to_run
    } else if sh_script_to_run.exists() {
        info!("\x1b[01;35m found script\x1b[38;5;93m:\x1b[38;5;1m {:?}", sh_script_to_run);
        sh_script_to_run
    } else {
        error!("\x1b[01;35m No scripts found in path:\x1b[38;5;93m:\x1b[38;5;1m{:?} looked for bash: {:?} looked for sh:{:?}", this_script_absolute_path, bash_script_to_run, sh_script_to_run);
        this_script_absolute_path
    };
    if script_to_run == this_script_absolute_path {
        return HttpResponse::NotFound().json(ApiResponse {
            status: "error".to_string(),
            command: format!("bash {:?}  sh {:?} ",bash_script_to_run,  sh_script_to_run),
            pid: std::process::id(),
            data: format!(""),
            msg: format!("No script found for the requested action bash {:?}  sh {:?} ",bash_script,  sh_script),
        });
    }
    let  script_to_run_clone1 = script_to_run;
    let  script_to_run_clone2 = script_to_run;
    let  script_to_run_clone3 = script_to_run;
    let mut child = match Command::new(&script_to_run)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn() {
        Ok(child) => child,
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_owned(),
            command: format!("{:?}", script_to_run),
            pid: std::process::id(),
            data: "".to_owned(),
            msg: format!("Failed to execute script {:?}", script_to_run),
        }),
    };
    if let Some(mut stdin) = child.stdin.take() {
        if stdin.write_all(body_str.as_bytes()).is_err() {
            return HttpResponse::InternalServerError().json(ApiResponse {
                status: "error".to_owned(),
                command: "Failed to write script stdin".to_owned(),
                pid: std::process::id(),
                data: "".to_owned(),
                msg: "Failed to write script stdin".to_owned(),
            });
        }
    }
    let mut output = String::new();
    if let Some(ref mut stdout) = child.stdout {
        stdout.read_to_string(&mut output).unwrap_or_else(|_| 0);
    }
    let status = match child.wait() {
        Ok(status) => status.code().unwrap_or_default(),
        Err(_) => return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            command: format!("{:?}", script_to_run_clone3),
            pid: std::process::id(),
            data: format!(""),
            msg: format!("Error occurred while waiting for the script  {:?} ",script_to_run_clone3),
        }),
    };
    if status != 0 {
        let mut error_output = String::new();
        if let Some(ref mut stderr) = child.stderr {
            stderr.read_to_string(&mut error_output).unwrap_or_else(|_| 0);
        }
        return HttpResponse::InternalServerError().json(ApiResponse {
            status: "error".to_string(),
            command: format!("{:?}", script_to_run_clone2),
            pid: std::process::id(),
            data: format!("{:?}", output),
            msg: format!("Error occurred while waiting for the script  {:?} ", script_to_run_clone2),
        });
    }
    HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        command: format!("{:?}", script_to_run_clone1),
        pid: std::process::id(),
        data: format!("{:?}", output),
        msg: format!("Ran script  {:?} ",script_to_run_clone1),
    })
}
fn print_help() {
    let version = VERSION;
    let this_script_relative_path = env::args().next().unwrap_or_default();
    let _this_script_absolute_path = env::current_exe().expect("Failed to get the current executable path");
    let _call_from_absolute_path = env::current_dir().expect("Failed to get the current directory");
    let this_script_name = Path::new(&this_script_relative_path)
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_owned();
    println!("{} version:{} Usage: {} ", this_script_name, version, this_script_name);
    println!("Creates a server listener in provided port. Then runs bash files or reads csv files to report. ");
    println!("No arguments besides this --help message");
    println!("Depends on optional .env file ");
    println!("Depends on optional .env_cors file");
    println!("Options:");
    println!("  --help, -h             Show this help message");
}
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.contains(&String::from("--help")) || args.contains(&String::from("-h")) {
        print_help();
        exit(0);
    }
    let this_script_relative_path = std::env::args().next().unwrap_or_default();
    let this_script_name = std::path::Path::new(&this_script_relative_path)
      .file_name()
      .unwrap_or_default()
      .to_str()
      .unwrap_or_default()
      .to_owned();
    let this_script_absolute_pathbuf = std::env::current_exe().expect("Failed to get the current executable path");
    let this_script_absolute_path = std::path::Path::new(&this_script_absolute_pathbuf);
    setup_logger();
    load_env_file();
    check_env_cors();
    dotenv().ok();
    info!("\x1b[01;35m
    info!("\x1b[01;35m
    info!("\x1b[01;35m
    let target_port = load_env_var("PORT", "8081");
    let target_host = load_env_var("HOST", "127.0.0.1");
    let target_server = format!("{}:{}", target_host, target_port);
    let mut cors_failed = false;
    let mut port_failed = false;
    let mut when_errors_detected = false;
    let allowed_origins = load_and_validate_cors_origins(".env_cors").unwrap_or_else(|e| {
        cors_failed = true;
        error!("Failed to load .env_cors, error: {:?}", e);
        vec![]
    });
    info!("Allowed origins: {:?}", allowed_origins);
    trace!(
        "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
        when_errors_detected,
        cors_failed,
        port_failed
    );
    let cors_origins = match load_and_validate_cors_origins(".env_cors") {
        Ok(origins) => {
            info!("CORS origins loaded successfully.");
            origins
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            cors_failed = true;
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            let pwd = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
            error!(".env_cors file not found in directory: {:?}", pwd.display());
            exit(1);
        }
        Err(e) => {
            cors_failed = true;
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Failed to load or validate all CORS origins: {}", e);
            exit(1);
        }
    };
    info!("Allowed cors_origins: {:?}", cors_origins);
    let lsof_available = Command::new("sh")
        .arg("-c")
        .arg("which lsof")
        .output()
        .map(|output| !output.stdout.is_empty())
        .unwrap_or(false);
    if !lsof_available {
        info!("`lsof` is not available. Please install `lsof` for more detailed diagnostics.");
        if std::net::TcpListener::bind(format!("{}", target_server)).is_err() {
            port_failed = true;
            trace!(
                "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Port {} is already in use.", target_port);
            exit(52);
        }
    } else {
        match std::net::TcpListener::bind(format!("{}", target_server)) {
            Ok(_) => {
            }
            Err(_) => {
                port_failed = true;
                trace!(
                    "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
                    when_errors_detected,
                    cors_failed,
                    port_failed
                );
                error!("Port {} is already in use.", target_port);
                if lsof_available {
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(format!("lsof -i :{} -t -sTCP:LISTEN",target_port))
                        .output();
                    match output {
                        Ok(output) if !output.stdout.is_empty() => {
                            let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                            info!("PID using port {}: {}", target_port, pid);
                            let cmd = format!("ps -o user= -o comm= -p {}", pid);
                            if let Ok(output) = Command::new("sh").arg("-c").arg(cmd).output() {
                                info!(
                                    "Process details: {}",
                                    String::from_utf8_lossy(&output.stdout)
                                );
                            }
                        }
                        _ => error!("Could not determine the process using port {}", target_port),
                    }
                }
                exit(52);
            }
        }
    }
    when_errors_detected = cors_failed || port_failed;
    trace!(
        "when_errors_detected {:?} cors_failed: {:?} port_failed: {:?}",
        when_errors_detected,
        cors_failed,
        port_failed
    );
    let server_pid = process_id();
    info!("Server starting with PID: {}", server_pid);
    if when_errors_detected {
        error!("Server start-up failed due to errors.");
        exit(1);
    } else {
        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials()
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS"])
                .allowed_headers(vec![
                    header::AUTHORIZATION,
                    header::ACCEPT,
                    header::CONTENT_TYPE,
                ])
                .max_age(3600);
            trace!("1 cors: {:?}", cors);
            let cors = Cors::permissive();
            trace!("2 cors: {:?}", cors);
            App::new()
                .wrap(ActixLogger::default())
                .wrap(cors)
                .route("/arduino/start", web::get().to(arduino_start))
                .route("/arduino/stop", web::get().to(arduino_stop))
                .route("/arduino/error", web::get().to(arduino_error))
                .route("/capture/start", web::get().to(capture_start))
                .route("/capture/kill", web::get().to(capture_kill))
                .route("/camera/release", web::get().to(capture_release))
                .route("/analysis/will/start", web::get().to(analysis_will_start))
                .route("/analysis/will/stop", web::get().to(analysis_will_stop))
                .route("/analysis/vero/start", web::get().to(analysis_vero_start))
                .route("/analysis/vero/stop", web::get().to(analysis_vero_stop))
                .route("/reports", web::get().to(reports))
                .service(
                    web::resource("/actions/{tail:.*}")
                        .route(web::get().to(execute_script))
                        .route(web::post().to(execute_script))
                        .route(web::put().to(execute_script))
                        .route(web::patch().to(execute_script)),
                    )
                .route("/action1", web::get().to(action1))
                .route("/action2", web::get().to(action2))
                .route("/action3", web::get().to(action3))
                .route("/action4", web::get().to(action4))
                .route("/action5", web::get().to(action5))
                .route("/action6", web::get().to(action6))
                .route("/action7", web::get().to(action7))
                .route("/action8", web::get().to(action8))
                .route("/reboot", web::get().to(reboot))
                .route("/actions/{tail:.*}", web::get().to(execute_script))
                .route("/actions/{tail:.*}", web::post().to(execute_script))
                .route("/list_routes", web::get().to(list_routes))
                .default_service(web::get().to(list_routes))
            })
            .bind(format!("{}", target_server))?
            .run();
        info!("Server running at http://{} ", format!("{}", target_server));
        trace!(
            "when_errors_detected: {:?} cors_failed:{:?} port_failed:{:?}",
            when_errors_detected,
            cors_failed,
            port_failed
        );
        let execution = server.await;
        info!("Worker stopped with PID: {}", process_id());
        if let Err(e) = execution {
            trace!(
                "when_errors_detected: {:?} cors_failed:{:?} port_failed:{:?}",
                when_errors_detected,
                cors_failed,
                port_failed
            );
            error!("Failed to start the server: {:?}", e);
            return Err(e);
        } else {
            if port_failed {
                error!("Port {} is already in use.", format!("{}", target_server));
                exit(1);
            }
            Ok(())
        }
    }
}
# file: stripe_evolved_project/main.rs   // --- end

# file: build.rs   // --- start
use std::fs;
fn main() {
    let cargo_toml = fs::read_to_string("Cargo.toml").expect("Failed to read Cargo.toml");
    let cargo: toml::Value = cargo_toml.parse().expect("Failed to parse Cargo.toml");
    if let Some(version) = cargo
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| v.as_str())
    {
        println!("cargo:rustc-env=CARGO_PKG_VERSION={}", version);
    }
    if let Some(description) = cargo
        .get("package")
        .and_then(|pkg| pkg.get("description"))
        .and_then(|v| v.as_str())
    {
        println!("cargo:rustc-env=CARGO_PKG_DESCRIPTION={}", description);
    }
    if let Some(name) = cargo
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(|v| v.as_str())
    {
        println!("cargo:rustc-env=CARGO_PKG_NAME={}", name);
    }
}
# file: build.rs   // --- end

# file: Cargo.toml   // --- start
[package]
name = "justpaystripe"
description = "Renewed KV Store Plus A synchronous + asynchronous payment library for processing payments with rust + stripe."
version = "0.2.0"
edition = "2021"
authors = ["Jesus Alcaraz <jesusalc@gmail.com>, Caleb Mitchell Smith-Woolrich <calebsmithwoolrich@gmail.com>"]
license = "MIT OR Apache-2.0"
documentation = "https://docs.rs/justpaystripe"
repository = "https://github.com/jesusalc/JustPayStripe-Rust"
readme = "README.md"
[dependencies]
serde_json = "1.0"
trust-dns-resolver = "0.23.2"
reqwest = { version = "0.11.9", default-features = false, features = ["blocking", "json", "multipart"] }
serde_derive = "1.0.130"
tokio = { version = "1.19.2", features = ["full"] }
dotenvy = "0.15.7"
actix-web = "4.11.0"
actix-cors = "0.7.1"
colored = "3.0.0"
env_logger = "0.11.8"
log = "0.4.27"
chrono = "0.4.41"
once_cell = "1.18"
[dependencies.serde]
version = "1.0"
features = ["derive"]
[features]
default = ["reqwest/default-tls", "trust-dns-resolver/dns-over-native-tls"]
[build-dependencies]
toml = "0.8.14"
# file: Cargo.toml   // --- end

