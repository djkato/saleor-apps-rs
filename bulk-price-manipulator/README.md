# Using Bulk Price Manipulator

Only works for a single website. No locale support.

**THERE IS NO UNDO! THINK BEFORE YOU COMMIT**

To run the expressions, just install the app through dashboard. They run only once per installation (triggered on registering). To retriger, remove and add the app.
App uses [evalexpr](https://github.com/ISibboI/evalexpr) for expressions. To learn what expressions are allowed, just check their github. It supports If statements and all the fancy stuff.
To see the supported variables that come from Saleor (eg. `variant.id`), check [src/updater/mod.rs](https://github.com/djkato/saleor-apps-rs/blob/7f5b00057a77dedc2f079cbe574ca99b0b89b5ff/bulk-price-manipulator/src/updater/mod.rs#L226), the strings after "set_value", are the variable names, their value definitions right underneath.

For getting the price of a variant, I recommend grabbing it from `variant.current_channel_listing.price.amount`.

App always queries for all products available at a single channel, which must be set in .env. In the same channel, it updates the pricing.

To test out, if your expressions work before deploying, either keep reinstalling the app in a local dev saleor environment, or install the evalexpr CLI, like `cargo install evalexpr`, then use like `$: evalexpr 1 + 2`.
