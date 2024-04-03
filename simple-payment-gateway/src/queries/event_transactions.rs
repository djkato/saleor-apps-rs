use const_format::concatcp;
#[cynic::schema("saleor")]
mod schema {}

pub const fragment_transaction_details: &str = r#"
fragment TransactionDetails on TransactionItem {
  id
  actions
  externalUrl
  message
  authorizedAmount {
    currency
    amount
  }
  authorizePendingAmount {
    currency
    amount
  }
  canceledAmount {
    currency
    amount
  }
  cancelPendingAmount {
    currency
    amount
  }
  chargedAmount {
    currency
    amount
  }
  chargePendingAmount {
    currency
    amount
  }
  refundedAmount {
    currency
    amount
  }
}
"#;

pub const fragment_order_details: &str = r#"
fragment OrderDetails on Order {
  checkoutId
  id
  status
  isPaid
  paymentStatus
  chargeStatus
  canFinalize
  totalBalance {
    currency
    amount
  }
}
"#;

// pub const sub_list_payment_gateways: &str = r#"
// subscription ListPaymentGateways {
//   event {
//     ... on PaymentListGateways {
//       checkout {
//         id
//       }
//     }
//   }
// }
// "#;

pub const sub_payment_gateway_initialize_session: &str = concatcp!(
    r#"
subscription PaymentGatewayInitializeSession {
  event {
    ... on PaymentGatewayInitializeSession {
      data
      amount
      sourceObject {
        ...OrderDetails
      }
      amount
    }
  }
}
"#,
    fragment_order_details
);

pub const sub_transaction_initialize_session: &str = concatcp!(
    r#"
subscription transactionInitializeSession {
  event {
    ... on TransactionInitializeSession {
      data
      sourceObject {
        ...OrderDetails
      }
      transaction {
        ...TransactionDetails
      }
      action {
        amount
        currency
        actionType
      }
    }
  }
}
"#,
    fragment_order_details,
    fragment_transaction_details
);

pub const sub_transaction_process_session: &str = concatcp!(
    r#"
subscription transactionProcessSession {
  event {
    ... on TransactionProcessSession {
      action {
        amount
        actionType
      }
      sourceObject {
        ...OrderDetails
      }
      transaction {
        ...TransactionDetails
      }
      data
    }
  }
}
"#,
    fragment_order_details,
    fragment_transaction_details
);

pub const sub_transaction_charge_requested: &str = concatcp!(
    r#"
subscription transactionChargeRequested {
  event {
    ... on TransactionChargeRequested {
      action {
        amount
        actionType
      }
      transaction {
        ...TransactionDetails
      }
    }
  }
}
"#,
    fragment_transaction_details
);

pub const sub_transaction_refund_requested: &str = concatcp!(
    r#"
subscription transactionRefundRequested {
  event {
    ... on TransactionRefundRequested {
      action {
        amount
        actionType
      }
      transaction {
        ...TransactionDetails
      }
    }
  }
}
"#,
    fragment_transaction_details
);

pub const sub_transaction_cancelation_requested: &str = concatcp!(
    r#"
subscription transactionCancelationRequested {
  event {
    ... on TransactionCancelationRequested {
      action {
        amount
        actionType
      }
      transaction {
        ...TransactionDetails
      }
    }
  }
}
"#,
    fragment_transaction_details
);

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TransactionRefundRequested")]
pub struct TransactionRefundRequested2 {
    pub action: TransactionAction,
    pub transaction: Option<TransactionItem>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TransactionProcessSession")]
pub struct TransactionProcessSession2 {
    pub action: TransactionProcessAction,
    pub source_object: OrderOrCheckout,
    pub transaction: TransactionItem,
    pub data: Option<Json>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TransactionProcessAction {
    pub amount: PositiveDecimal,
    pub action_type: TransactionFlowStrategyEnum,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TransactionInitializeSession")]
pub struct TransactionInitializeSession2 {
    pub data: Option<Json>,
    pub source_object: OrderOrCheckout,
    pub transaction: TransactionItem,
    pub action: TransactionProcessAction2,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TransactionProcessAction")]
pub struct TransactionProcessAction2 {
    pub amount: PositiveDecimal,
    pub currency: String,
    pub action_type: TransactionFlowStrategyEnum,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TransactionChargeRequested")]
pub struct TransactionChargeRequested2 {
    pub action: TransactionAction,
    pub transaction: Option<TransactionItem>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "TransactionCancelationRequested")]
pub struct TransactionCancelationRequested2 {
    pub action: TransactionAction,
    pub transaction: Option<TransactionItem>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TransactionItem {
    pub id: cynic::Id,
    pub actions: Vec<TransactionActionEnum>,
    pub external_url: String,
    pub message: String,
    pub authorized_amount: Money,
    pub authorize_pending_amount: Money,
    pub canceled_amount: Money,
    pub cancel_pending_amount: Money,
    pub charged_amount: Money,
    pub charge_pending_amount: Money,
    pub refunded_amount: Money,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TransactionAction {
    pub amount: Option<PositiveDecimal>,
    pub action_type: TransactionActionEnum,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct TransactionRefundRequested {
    pub event: Option<Event>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct TransactionProcessSession {
    pub event: Option<Event2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct TransactionInitializeSession {
    pub event: Option<Event3>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct TransactionChargeRequested {
    pub event: Option<Event4>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct TransactionCancelationRequested {
    pub event: Option<Event5>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct PaymentGatewayInitializeSession {
    pub event: Option<Event6>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "PaymentGatewayInitializeSession")]
pub struct PaymentGatewayInitializeSession2 {
    pub data: Option<Json>,
    pub amount: Option<PositiveDecimal>,
    pub source_object: OrderOrCheckout,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Order {
    pub checkout_id: Option<cynic::Id>,
    pub id: cynic::Id,
    pub status: OrderStatus,
    pub is_paid: bool,
    pub payment_status: PaymentChargeStatusEnum,
    pub charge_status: OrderChargeStatusEnum,
    pub can_finalize: bool,
    pub total_balance: Money,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Money {
    pub currency: String,
    pub amount: f64,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(graphql_type = "Event")]
pub enum Event6 {
    PaymentGatewayInitializeSession2(PaymentGatewayInitializeSession2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(graphql_type = "Event")]
pub enum Event5 {
    TransactionCancelationRequested2(TransactionCancelationRequested2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(graphql_type = "Event")]
pub enum Event4 {
    TransactionChargeRequested2(TransactionChargeRequested2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(graphql_type = "Event")]
pub enum Event3 {
    TransactionInitializeSession2(TransactionInitializeSession2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
#[cynic(graphql_type = "Event")]
pub enum Event2 {
    TransactionProcessSession2(TransactionProcessSession2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum Event {
    TransactionRefundRequested2(TransactionRefundRequested2),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum OrderOrCheckout {
    Order(Order),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderChargeStatusEnum {
    None,
    Partial,
    Full,
    Overcharged,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum OrderStatus {
    Draft,
    Unconfirmed,
    Unfulfilled,
    PartiallyFulfilled,
    PartiallyReturned,
    Returned,
    Fulfilled,
    Canceled,
    Expired,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum PaymentChargeStatusEnum {
    NotCharged,
    Pending,
    PartiallyCharged,
    FullyCharged,
    PartiallyRefunded,
    FullyRefunded,
    Refused,
    Cancelled,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum TransactionActionEnum {
    Charge,
    Refund,
    Cancel,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum TransactionFlowStrategyEnum {
    Authorization,
    Charge,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "JSON")]
pub struct Json(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct PositiveDecimal(pub String);
