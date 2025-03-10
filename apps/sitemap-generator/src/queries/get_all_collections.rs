#[cynic::schema("saleor")]
mod schema {}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetCollectionsNextVariables<'a> {
    pub after: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "GetCollectionsNextVariables")]
pub struct GetCollectionsNext {
    #[arguments(first: 50, after: $after)]
    pub collections: Option<CollectionCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query")]
pub struct GetCollectionsInitial {
    #[arguments(first: 50)]
    pub collections: Option<CollectionCountableConnection2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "CollectionCountableConnection")]
pub struct CollectionCountableConnection2 {
    pub total_count: Option<i32>,
    pub page_info: PageInfo,
    pub edges: Vec<CollectionCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CollectionCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<CollectionCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CollectionCountableEdge {
    pub node: Collection,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Collection {
    pub id: cynic::Id,
    pub slug: String,
}
