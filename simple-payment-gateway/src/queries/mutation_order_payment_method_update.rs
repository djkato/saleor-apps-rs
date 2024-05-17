#[cynic::schema("saleor")]
mod schema {}
/*
mutation setOrderPaymentMethod($id:ID!, $metadata: [MetadataInput!]!){
  updateMetadata(id:$id, input: $metadata ){
    errors{
      field
      message
      code
    }
  }
}
*/

#[derive(cynic::QueryVariables, Debug)]
pub struct SetOrderPaymentMethodVariables<'a> {
    pub id: &'a cynic::Id,
    pub metadata: Vec<MetadataInput<'a>>,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Mutation",
    variables = "SetOrderPaymentMethodVariables"
)]
pub struct SetOrderPaymentMethod {
    #[arguments(id: $id, input: $metadata)]
    pub update_metadata: Option<UpdateMetadata>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct UpdateMetadata {
    pub errors: Vec<MetadataError>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct MetadataError {
    pub field: Option<String>,
    pub message: Option<String>,
    pub code: MetadataErrorCode,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum MetadataErrorCode {
    GraphqlError,
    Invalid,
    NotFound,
    Required,
    NotUpdated,
}

#[derive(cynic::InputObject, Debug)]
pub struct MetadataInput<'a> {
    pub key: &'a str,
    pub value: &'a str,
}
