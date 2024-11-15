#[cynic::schema("saleor")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct DeleteAppMetadataVariables<'a> {
    pub app_id: &'a cynic::Id,
    pub keys: Vec<&'a str>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SetAppPrivateMetadataVariables<'a> {
    pub app_id: &'a cynic::Id,
    pub input: Vec<MetadataInput>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct GetAppMetadataVariables<'a> {
    pub app_id: &'a cynic::Id,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct GetAppPrivateMetadataVariables<'a> {
    pub app_id: &'a cynic::Id,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct SetAppMetadataVariables<'a> {
    pub app_id: &'a cynic::Id,
    pub input: Vec<MetadataInput>,
}

#[derive(cynic::QueryVariables, Debug)]
pub struct DeleteAppPrivateMetadataVariables<'a> {
    pub app_id: &'a cynic::Id,
    pub keys: Vec<&'a str>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetAppPrivateMetadataVariables")]
pub struct GetAppPrivateMetadata {
    #[arguments(id: $app_id)]
    pub app: Option<App>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "GetAppMetadataVariables")]
pub struct GetAppMetadata {
    #[arguments(id: $app_id)]
    pub app: Option<App2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Mutation",
    variables = "SetAppPrivateMetadataVariables"
)]
pub struct SetAppPrivateMetadata {
    #[arguments(id: $app_id, input: $input)]
    pub update_private_metadata: Option<UpdatePrivateMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct UpdatePrivateMetadata {
    pub item: Option<ObjectWithMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "SetAppMetadataVariables")]
pub struct SetAppMetadata {
    #[arguments(id: $app_id, input: $input)]
    pub update_metadata: Option<UpdateMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct UpdateMetadata {
    pub item: Option<ObjectWithMetadata2>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "ObjectWithMetadata")]
pub struct ObjectWithMetadata2 {
    pub metadata: Vec<MetadataItem>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Mutation",
    variables = "DeleteAppPrivateMetadataVariables"
)]
pub struct DeleteAppPrivateMetadata {
    #[arguments(id: $app_id, keys: $keys)]
    pub delete_private_metadata: Option<DeletePrivateMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "DeleteAppMetadataVariables")]
pub struct DeleteAppMetadata {
    #[arguments(id: $app_id, keys: $keys)]
    pub delete_metadata: Option<DeleteMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DeletePrivateMetadata {
    pub item: Option<ObjectWithMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct DeleteMetadata {
    pub item: Option<ObjectWithMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ObjectWithMetadata {
    pub private_metadata: Vec<MetadataItem>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct App {
    pub private_metadata: Vec<MetadataItem>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "App")]
pub struct App2 {
    pub metadata: Vec<MetadataItem>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct MetadataItem {
    pub key: String,
    pub value: String,
}

#[derive(cynic::InputObject, Debug)]
pub struct MetadataInput {
    pub key: String,
    pub value: String,
}

/*
query GetAppMetadata($app_id: ID!) {
  app(id: $app_id) {
    metadata {
      key
      value
    }
  }
}

query GetAppPrivateMetadata($app_id: ID!) {
  app(id: $app_id) {
    privateMetadata {
      key
      value
    }
  }
}

mutation SetAppMetadata($app_id: ID!, $input: [MetadataInput!]!) {
  updateMetadata(id: $app_id, input: $input) {
    item {
      metadata {
        key
        value
      }
    }
  }
}

mutation SetAppPrivateMetadata($app_id: ID!, $input: [MetadataInput!]!) {
  updatePrivateMetadata(id: $app_id, input: $input) {
    item {
      privateMetadata {
        key
        value
      }
    }
  }
}

mutation DeleteAppMetadata($app_id: ID!, $keys: [String!]!) {
  deleteMetadata(id: $app_id, keys: $keys) {
    item {
      privateMetadata {
        key
        value
      }
    }
  }
}

mutation DeleteAppPrivateMetadata($app_id: ID!, $keys: [String!]!) {
  deletePrivateMetadata(id: $app_id, keys: $keys) {
    item {
      privateMetadata {
        key
        value
      }
    }
  }
}
*/
