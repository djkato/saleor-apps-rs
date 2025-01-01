# Using Bulk Price Manipulator

Only works for a single website. No locale support.

**THERE IS NO UNDO! THINK BEFORE YOU COMMIT**

To run the expressions, just install the app through dashboard. They run only once per installation (triggered on registering). To retriger, remove and add the app.
App uses [evalexpr](https://github.com/ISibboI/evalexpr) for expressions. To learn what expressions are allowed, just check their github. It supports If statements and all the fancy stuff.

App always queries for all products available at a single channel, which must be set in .env. In the same channel, it updates the pricing.

To test out, if your expressions work before deploying, either keep reinstalling the app in a local dev saleor environment, or install the evalexpr CLI, like `cargo install evalexpr`, then use like `$: evalexpr 1 + 2`.
