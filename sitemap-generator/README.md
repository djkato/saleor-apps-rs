# Using sitemap-generator

Only works for a single website. No locale support and no sitemap-index. Outputs Only pure sitemap.txt file. Downside is limit of 50 000 links. Upside: Easy to write c:
Partially supports relations of objects (Category-product), where the sitemap template can use info from both.

# Unofficial Saleor App Template

To update the saleor schema, you can download it from [here](https://raw.githubusercontent.com/saleor/saleor/main/saleor/graphql/schema.graphql) and put into schema/schema.graphql
To generate typings for events and gql queries, use: https://generator.cynic-rs.dev/
