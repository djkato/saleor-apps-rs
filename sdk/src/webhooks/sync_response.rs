use iso_currency::Currency;
use rust_decimal::Decimal;
use serde::Serialize;

//Why are these few in snake_case but rest is camelCase?
#[derive(Serialize, Debug, Clone)]
pub struct CheckoutCalculateTaxesResponse {
    #[serde(with = "rust_decimal::serde::float")]
    pub shipping_price_gross_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub shipping_price_net_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub shipping_tax_rate: Decimal,
    pub lines: Vec<LinesResponse>,
}

#[derive(Serialize, Debug, Clone)]
pub struct LinesResponse {
    #[serde(with = "rust_decimal::serde::float")]
    pub total_gross_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub total_net_amount: Decimal,
    #[serde(with = "rust_decimal::serde::float")]
    pub tax_rate: Decimal,
}

#[derive(Serialize, Debug, Clone)]
pub struct CheckoutFilterShippingMethodsResponse {
    pub excluded_methods: Vec<ExcludedMethodsResponse>,
}

#[derive(Serialize, Debug, Clone)]
pub struct ExcludedMethodsResponse {
    pub id: String,
    pub reason: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct OrderCalculateTaxes(CheckoutCalculateTaxesResponse);

#[derive(Serialize, Debug, Clone)]
pub struct OrderFilterShippingMethods(CheckoutFilterShippingMethodsResponse);

#[derive(Serialize, Debug, Clone)]
pub struct ShippingListMethodsForCheckout(Vec<ShippingListMethodsForCheckoutVec>);

#[derive(Serialize, Debug, Clone)]
struct ShippingListMethodsForCheckoutVec {
    pub id: String,
    pub name: Option<String>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub currency: String,
    pub maximum_delivery_days: Option<i32>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ChargeRequestedResult {
    ChargeSuccess,
    ChargeFailiure,
}

#[derive(Serialize, Debug, Clone)]
pub struct TransactionChargeRequestedResponse {
    pub psp_reference: String,
    pub result: Option<ChargeRequestedResult>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub amount: Option<Decimal>,
    pub time: Option<String>,
    pub external_url: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RefundRequestedResult {
    RefundSuccess,
    RefundFailiure,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRefundRequestedResponse {
    pub psp_reference: String,
    pub result: Option<RefundRequestedResult>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub amount: Option<Decimal>,
    pub time: Option<String>,
    pub external_url: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancelationRequestedResult {
    CancelSuccess,
    CancelFailiure,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionCancelationRequestedResponse {
    pub psp_reference: String,
    pub result: Option<CancelationRequestedResult>,
    #[serde(with = "rust_decimal::serde::float_option")]
    pub amount: Option<Decimal>,
    pub time: Option<String>,
    pub external_url: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PaymentGatewayInitializeSessionResponse<T: Serialize> {
    pub data: Option<T>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionSessionResult {
    ChargeSuccess,
    ChargeFailiure,
    ChargeRequested,
    ChargeActionRequired,
    AuthorizationSuccess,
    AuthorizationFailure,
    AuthorizationRequested,
    AuthorizationActionRequired,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInitializeSessionResponse<T: Serialize> {
    pub psp_reference: Option<String>,
    pub data: Option<T>,
    pub result: TransactionSessionResult,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub time: Option<String>,
    pub external_url: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransactionProcessSessionResponse<T: Serialize> {
    pub psp_reference: Option<String>,
    pub data: Option<T>,
    pub result: TransactionSessionResult,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: Decimal,
    pub time: Option<String>,
    pub external_url: Option<String>,
    pub message: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentMethodTokenizationResult {
    SucessfullyTokenized,
    AdditionalActionRequired,
    Pending,
    FailedToTokenize,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethodProcessTokenizationSession<T: Serialize> {
    pub result: PaymentMethodTokenizationResult,
    /**
    Should be present when `PaymentMethodTokenizationResult::{SuccessfullyTokenized && AdditionalActionRequired}`
    */
    pub id: Option<String>,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
pub struct PaymentMethodInitializeTokenizationSession<T: Serialize>(
    PaymentMethodProcessTokenizationSession<T>,
);

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentGatewayTokenisationResult {
    SuccessfullyInitialized,
    FailedToInitialize,
}

#[derive(Serialize, Debug, Clone)]
pub struct PaymentGatewayInitializeTokenizationSession<T: Serialize> {
    pub result: PaymentGatewayTokenisationResult,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StoredPaymentMethodDeleteResult {
    SucessfullyDeleted,
    FailedToDelete,
}

#[derive(Serialize, Debug, Clone)]
pub struct StoredPaymentMethodDeleteRequested {
    pub result: StoredPaymentMethodDeleteResult,
    pub error: Option<String>,
}

//TODO: Dahek is Array<"INTERACTIVE"> from app-sdk/../sync-webhook-response-builder.ts:LIST_STORED_PAYMENT_METHODS?
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod<T: Serialize> {
    pub id: String,
    pub supported_payment_flows: Vec<T>,
    #[serde(rename = "type")]
    pub typ: String,
    pub credit_card_info: Option<CreditCardInfo>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreditCardInfo {
    pub brand: String,
    pub last_digits: String,
    pub exp_month: String,
    pub exp_year: String,
    pub first_digits: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ListStoredPaymentMethodsResponse<T: Serialize, C: Serialize> {
    pub payment_methods: Vec<PaymentMethod<C>>,
    pub name: Option<String>,
    pub data: Option<T>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentGateway {
    pub id: String,
    pub name: String,
    pub currencies: Vec<Currency>,
    pub config: Vec<ConfigMap>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigMap {
    field: String,
    value: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PaymentListGatewaysResponse(pub Vec<PaymentGateway>);
