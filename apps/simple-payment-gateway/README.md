# Simple Payment Gateway

Pricing and license can be found in [root readme.md](https://github.com/djkato/saleor-apps-rs/tree/master/README.md)

Saleor app that acts as a simple payment gateway for payment methods that do not require automatic validation.

The payment methods are toggleable with env variables. Currently it supports these methods:

- Accreditation
- Cash
- COD (Cash on Delivery)
- Inkaso
- Other
- Transfer

It also does some simple logic checks to make sure that when delivery method is `ClickAndCollect`/warehouse pickup, `COD` isn't possible, and if it's any other delivery method `Cash` isn't possible.
Availabe payment methods get send during `PaymentGatewayInitialize`, where this gateway returns available payment methods inside the data field.
When checking out with `TransactionInitializeSession`, include the desired payment method in data like `{"payment_method": "method"}`.
