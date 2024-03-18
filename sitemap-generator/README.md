# Using sitemap-generator

To clear the cache, you can run the program with `./sitemap-generator --for-url https://my-saleor-api.com/graphql --cache-clear` or `docker compose --rm app-sitemap-generator sitemap-generator --for-url https://my-saleor-api.com/graphql --cache-clear`
To regenerate the cache, you can run the program with `./sitemap-generator --for-url https://my-saleor-api.com/graphql --cache-regenerate` or `docker compose --rm app-sitemap-generator sitemap-generator --for-url https://my-saleor-api.com/graphql --cache-regenerate`

You can also add both flags (do --cache-regenerate first), which will clear and then regenerate.

# Unofficial Saleor App Template

To update the saleor schema, you can download it from [here](https://raw.githubusercontent.com/saleor/saleor/main/saleor/graphql/schema.graphql) and put into schema/schema.graphql
To generate typings for events and gql queries, use: https://generator.cynic-rs.dev/
