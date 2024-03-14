#[cynic::schema("saleor")]
mod schema {}
/*
mutation transactionUpdate($id: ID!, $transaction: TransactionUpdateInput) {
  transactionUpdate(id: $id, transaction: $transaction) {
    transaction {
      id
      actions
      externalUrl
      message
    }
    errors {
      field
      message
      code
    }
  }
}
*/

#[derive(cynic::QueryVariables, Debug)]
pub struct TransactionUpdateVariables<'a> {
    pub id: &'a cynic::Id,
    pub transaction: Option<TransactionUpdateInput<'a>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "TransactionUpdateVariables")]
pub struct TransactionUpdate {
    #[arguments(id: $id, transaction: $transaction)]
    pub transaction_update: Option<TransactionUpdate2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TransactionUpdate")]
pub struct TransactionUpdate2 {
    pub transaction: Option<TransactionItem>,
    pub errors: Vec<TransactionUpdateError>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TransactionUpdateError {
    pub field: Option<String>,
    pub message: Option<String>,
    pub code: TransactionUpdateErrorCode,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TransactionItem {
    pub id: cynic::Id,
    pub actions: Vec<TransactionActionEnum>,
    pub external_url: String,
    pub message: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum TransactionActionEnum {
    Charge,
    Refund,
    Cancel,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum TransactionUpdateErrorCode {
    Invalid,
    GraphqlError,
    NotFound,
    IncorrectCurrency,
    MetadataKeyRequired,
    Unique,
}

#[derive(cynic::InputObject, Debug, Default)]
pub struct TransactionUpdateInput<'a> {
    pub name: Option<&'a str>,
    pub message: Option<&'a str>,
    pub psp_reference: Option<&'a str>,
    pub available_actions: Option<Vec<TransactionActionEnum>>,
    pub amount_authorized: Option<MoneyInput<'a>>,
    pub amount_charged: Option<MoneyInput<'a>>,
    pub amount_refunded: Option<MoneyInput<'a>>,
    pub amount_canceled: Option<MoneyInput<'a>>,
    pub metadata: Option<Vec<MetadataInput<'a>>>,
    pub private_metadata: Option<Vec<MetadataInput<'a>>>,
    pub external_url: Option<&'a str>,
}

#[derive(cynic::InputObject, Debug)]
pub struct MoneyInput<'a> {
    pub currency: &'a str,
    pub amount: PositiveDecimal,
}

#[derive(cynic::InputObject, Debug)]
pub struct MetadataInput<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct PositiveDecimal(pub String);
