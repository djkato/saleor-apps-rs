# heureka-xml-feed

Pricing and license can be found in [root readme.md](https://github.com/djkato/saleor-apps-rs/tree/master/README.md)

## The way it works:

- keep local DB cache to not query for everything on each `/product-feed.xml` request, use XXXXUpdated webhooks to keep uptodate
- if DB is empty (0 products) or new, regenerate it by querying for all products/categories/variants/shippingMethods, and save. Create a test XML and validate, save errors to DB
- during any errors(DB unreachable, graphql errors, xml schema validation errors), collect them, save them to DB and show in dashboard
- on any subsequent changes to products, variants, categories, shippingMethods, receive webhook and update only that fragment in local db
- on request to `/product-feed.xml`, query local DB cache for everything, parse into SHOP/SHOPITEMs, and return XML

thanks WASM <3
