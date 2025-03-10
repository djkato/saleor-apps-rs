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
          id
          slug
          updatedAt
          category {
            id
            slug
          }
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
        id
        slug
        updatedAt
        category {
          id
          slug
        }
      }
    }
  }
}
*/

#[derive(cynic::QueryVariables, Debug)]
pub struct GetProductsInitialVariables<'a> {
    pub channel: &'a str,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct GetProductsNextVariables<'a> {
    pub after: &'a str,
    pub channel: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetProductsInitialVariables")]
pub struct GetProductsInitial {
    #[arguments(first: 50, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetProductsNextVariables")]
pub struct GetProductsNext {
    #[arguments(first: 50, after: $after, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductCountableEdge {
    pub node: Product,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Product {
    pub id: cynic::Id,
    pub slug: String,
    pub updated_at: DateTime,
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Category {
    pub id: cynic::Id,
    pub slug: String,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);
