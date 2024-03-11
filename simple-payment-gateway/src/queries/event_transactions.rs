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
