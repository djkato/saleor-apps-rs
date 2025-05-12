# Using sitemap-generator

Pricing and license can be found in [root readme.md](https://github.com/djkato/saleor-apps-rs/tree/master/README.md)

Only works for a single website. No locale support and no sitemap-index. Outputs Only pure sitemap.txt file. Downside is limit of 50 000 links. Upside: Easy to write c:
Partially supports relations of objects (Category-product), where the sitemap template can use info from both.

to create the links, a template set up in ENV is used, eg:

```toml
SITEMAP_PRODUCT_TEMPLATE="https://example.com/{product.category.slug}/{product.slug}"
```
