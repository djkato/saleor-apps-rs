#[cynic::schema("saleor")]
mod schema {}
/*
query getCategoriesInitial {
  categories(first: 50) {
    totalCount
    pageInfo {
      hasNextPage
      endCursor
    }
    edges {
      node {
        updatedAt
        id
        slug
      }
    }
  }
}

query getCategoriesNext($after: String) {
  categories(first: 50, after: $after) {
    pageInfo {
      hasNextPage
      endCursor
    }
    edges {
      node {
        updatedAt
        id
        slug
      }
    }
  }
}
*/

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetCategoriesNextVariables<'a> {
    pub after: Option<&'a str>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetCategoryProductsInitialVariables<'a> {
    pub channel: &'a str,
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetCategoryProductsNextVariables<'a> {
    pub after: &'a str,
    pub channel: &'a str,
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(
    graphql_type = "Query",
    variables = "GetCategoryProductsInitialVariables"
)]
pub struct GetCategoryProductsInitial {
    #[arguments(id: $id)]
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "GetCategoryProductsNextVariables")]
pub struct GetCategoryProductsNext {
    #[arguments(id: $id)]
    pub category: Option<Category2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "GetCategoriesNextVariables")]
pub struct GetCategoriesNext {
    #[arguments(first: 50, after: $after)]
    pub categories: Option<CategoryCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query")]
pub struct GetCategoriesInitial {
    #[arguments(first: 50)]
    pub categories: Option<CategoryCountableConnection2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "CategoryCountableConnection")]
pub struct CategoryCountableConnection2 {
    pub total_count: Option<i32>,
    pub page_info: PageInfo,
    pub edges: Vec<CategoryCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CategoryCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<CategoryCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CategoryCountableEdge {
    pub node: Category3,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category3 {
    pub updated_at: DateTime,
    pub id: cynic::Id,
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(variables = "GetCategoryProductsInitialVariables")]
pub struct Category {
    pub slug: String,
    pub id: cynic::Id,
    pub updated_at: DateTime,
    #[arguments(first: 50, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
    pub total_count: Option<i32>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(
    graphql_type = "Category",
    variables = "GetCategoryProductsNextVariables"
)]
pub struct Category2 {
    #[arguments(first: 50, after: $after, channel: $channel)]
    pub products: Option<ProductCountableConnection2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "ProductCountableConnection")]
pub struct ProductCountableConnection2 {
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
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);
