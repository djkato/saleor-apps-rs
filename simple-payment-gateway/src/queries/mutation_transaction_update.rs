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
