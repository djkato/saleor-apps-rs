#[cynic::schema("saleor")]
mod schema {}

/*
query getProductsInitial($id: ID!, $channel: String!) {
  category(id: $id) {
    slug
    id
    updatedAt
    products(first: 50, channel: $channel) {
      pageInfo {
        hasNextPage
        endCursor
      }
      edges {
        node {
          ...ProductData
        }
      }
      totalCount
    }
  }
}

query getProductsNext($after: String!, $channel: String!) {
  products(first: 50, after: $after, channel: $channel) {
    pageInfo {
      hasNextPage
      endCursor
    }
    edges {
      node {
        ...ProductData
      }
    }
  }
}

fragment ProductData on Product {
  variant {
    id
    name
    pricing {
      price {
        gross {
          amount
        }
      }
    }
  }
  category {
    name
    id
    metafield(key: "heureka_categorytext")
    parent {
      name
      id
      metafield(key: "heureka_categorytext")
      parent {
        name
        id
        metafield(key: "heureka_categorytext")
        parent {
          name
          id
          metafield(key: "heureka_categorytext")
          parent {
            name
            id
            metafield(key: "heureka_categorytext")
            parent {
              name
              id
              metafield(key: "heureka_categorytext")
              parent {
                name
                id
                metafield(key: "heureka_categorytext")
                parent {
                  name
                  id
                  metafield(key: "heureka_categorytext")
                  parent {
                    name
                    id
                    metafield(key: "heureka_categorytext")
                    parent {
                      name
                      id
                      metafield(key: "heureka_categorytext")
                      parent {
                        name
                        id
                        metafield(key: "heureka_categorytext")
                        parent {
                          name
                          id
                          metafield(key: "heureka_categorytext")
                          parent {
                            name
                            id
                            metafield(key: "heureka_categorytext")
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
  name
  description
  media {
    url(format: WEBP, size: 1024)
    alt
  }
}
*/

#[derive(cynic::QueryVariables, Debug)]
pub struct GetProductsNextVariables<'a> {
    pub after: &'a str,
    pub channel: &'a str,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct GetProductsInitialVariables<'a> {
    pub channel: &'a str,
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetProductsNextVariables")]
pub struct GetProductsNext {
    #[arguments(first: 50, after: $after, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetProductsInitialVariables")]
pub struct GetProductsInitial {
    #[arguments(id: $id)]
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(variables = "GetProductsInitialVariables")]
pub struct Category {
    pub slug: String,
    pub id: cynic::Id,
    pub updated_at: DateTime,
    #[arguments(first: 50, channel: $channel)]
    pub products: Option<ProductCountableConnection2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ProductCountableConnection")]
pub struct ProductCountableConnection2 {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
    pub total_count: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductCountableEdge {
    pub node: Product,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Product {
    pub variant: Option<ProductVariant>,
    pub category: Option<Category2>,
    pub name: String,
    pub description: Option<Jsonstring>,
    pub media: Option<Vec<ProductMedia>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductMedia {
    #[arguments(format: "WEBP", size: 1024)]
    pub url: String,
    pub alt: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductVariant {
    pub id: cynic::Id,
    pub name: String,
    pub pricing: Option<VariantPricingInfo>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct VariantPricingInfo {
    pub price: Option<TaxedMoney>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct TaxedMoney {
    pub gross: Money,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Money {
    pub amount: f64,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category2 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category3>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category3 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category4>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category4 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category5>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category5 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category6>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category6 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category7>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category7 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category8>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category8 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category9>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category9 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category10>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category10 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category11>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category11 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category12>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category12 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category13>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category13 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category14>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category14 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum ThumbnailFormatEnum {
    Original,
    Avif,
    Webp,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "JSONString")]
pub struct Jsonstring(pub String);
