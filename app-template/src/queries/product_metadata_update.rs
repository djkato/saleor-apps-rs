#[cynic::schema("saleor")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct UpdateProductMetadataVariables<'a> {
    pub metadata: Option<Vec<MetadataInput<'a>>>,
    pub product_id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(
    graphql_type = "Mutation",
    variables = "UpdateProductMetadataVariables"
)]
pub struct UpdateProductMetadata {
    #[arguments(id: $product_id, input: { metadata: $metadata })]
    pub product_update: Option<ProductUpdate>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductUpdate {
    pub errors: Vec<ProductError>,
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Product {
    pub id: cynic::Id,
    pub metadata: Vec<MetadataItem>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductError {
    pub field: Option<String>,
    pub message: Option<String>,
    pub code: ProductErrorCode,
    pub attributes: Option<Vec<cynic::Id>>,
    pub values: Option<Vec<cynic::Id>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct MetadataItem {
    pub key: String,
    pub value: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum ProductErrorCode {
    AlreadyExists,
    AttributeAlreadyAssigned,
    AttributeCannotBeAssigned,
    AttributeVariantsDisabled,
    MediaAlreadyAssigned,
    DuplicatedInputItem,
    GraphqlError,
    Invalid,
    InvalidPrice,
    ProductWithoutCategory,
    NotProductsImage,
    NotProductsVariant,
    NotFound,
    Required,
    Unique,
    VariantNoDigitalContent,
    CannotManageProductWithoutVariant,
    ProductNotAssignedToChannel,
    UnsupportedMediaProvider,
    PreorderVariantCannotBeDeactivated,
}

#[derive(cynic::InputObject, Debug)]
pub struct MetadataInput<'a> {
    pub key: &'a str,
    pub value: &'a str,
}
