# Bulk Price Manipulator

Pricing and license can be found in [root readme.md](https://github.com/djkato/saleor-apps-rs/tree/master/README.md)

Only works for a single website. No locale support.

**THERE IS NO UNDO! THINK BEFORE YOU COMMIT**

**APP QUERIES FOR ALL PRODUCT VARIANTS (in batches of 99), AND CHANGES ALL VARIANTS PRICES REGARDLESS OF THE EXPRESSION. DON'T DDOS YOURSELF, USE DURING DOWN-TIME!**

To run the expressions, just install the app through dashboard. They run only once per installation (triggered on registering). To retriger, remove and add the app.
App uses [evalexpr](https://github.com/ISibboI/evalexpr) for expressions. To learn what expressions are allowed, just check their github. It supports If statements and all the fancy stuff.
To see the supported variables that come from Saleor (eg. `variant.id`), check `src/updater/mod.rs` in the `create_context_map` function, the strings after "set_value", are the variable names, their value definitions right underneath.

For getting the price of a variant, I recommend grabbing it from `variant.current_channel_listing.price.amount`.

App always queries for all products available at a single channel, which must be set in .env. In the same channel, it updates the pricing.

To test out, if your expressions work before deploying, either keep reinstalling the app in a local dev saleor environment, or install the evalexpr CLI, like `cargo install evalexpr`, then use like `$: evalexpr 1 + 2`.

example expressions:

```toml
PRICE_EXPRESSION='variant.current_channel_listing.price.amount * 0.8'
COST_PRICE_EXPRESSION='variant.current_channel_listing.cost_price.amount * 0.8'
```

```toml
PRICE_EXPRESSION='if(variant.product.product_type.name == str::from("Shoe"), 25.99, variant.current_channel_listing.price.amount)'
COST_PRICE_EXPRESSION='if(variant.pricing.on_sale, variant.pricing.price.net.amount, variant.current_channel_listing.cost_price.amount)'
```
