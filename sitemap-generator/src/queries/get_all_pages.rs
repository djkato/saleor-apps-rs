#[cynic::schema("saleor")]
mod schema {}

/*
query getPagesInitial {
  pages(first: 50) {
    totalCount
    pageInfo {
      hasNextPage
      endCursor
    }
    edges {
      node {
        publishedAt
        id
        slug
      }
    }
  }
}

query getPagesNext($after: String!) {
  pages(first: 50, after: $after) {
    pageInfo {
      hasNextPage
      endCursor
    }
    edges {
      node {
        publishedAt
        id
        slug
      }
    }
  }
}
*/

#[derive(cynic::QueryVariables, Debug)]
pub struct GetPagesNextVariables<'a> {
    pub after: &'a str,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetPagesNextVariables")]
pub struct GetPagesNext {
    #[arguments(first: 50, after: $after)]
    pub pages: Option<PageCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query")]
pub struct GetPagesInitial {
    #[arguments(first: 50)]
    pub pages: Option<PageCountableConnection2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "PageCountableConnection")]
pub struct PageCountableConnection2 {
    pub total_count: Option<i32>,
    pub page_info: PageInfo,
    pub edges: Vec<PageCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<PageCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageCountableEdge {
    pub node: Page,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Page {
    pub published_at: Option<DateTime>,
    pub id: cynic::Id,
    pub slug: String,
}

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);
