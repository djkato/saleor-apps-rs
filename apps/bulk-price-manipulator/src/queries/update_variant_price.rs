use rust_decimal::Decimal;

#[cynic::schema("saleor")]
mod schema {}

/*
mutation updatePrice($channel: ID!, $variant: ID!, $price: PositiveDecimal!, $costPrice: PositiveDecimal) {
  productVariantChannelListingUpdate(
    id: $variant
    input: {channelId: $channel, price: $price, costPrice: $costPrice}
  ) {
    errors {
      code
      message
      field
    }
  }
}
*/

#[derive(cynic::QueryVariables, Debug)]
pub struct UpdatePriceVariables<'a> {
    pub channel: &'a cynic::Id,
    pub cost_price: Option<PositiveDecimal>,
    pub price: PositiveDecimal,
    pub variant: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Mutation", variables = "UpdatePriceVariables")]
pub struct UpdatePrice {
    #[arguments(id: $variant, input: { channelId: $channel, costPrice: $cost_price, price: $price })]
    pub product_variant_channel_listing_update: Option<ProductVariantChannelListingUpdate>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductVariantChannelListingUpdate {
    pub errors: Vec<ProductChannelListingError>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductChannelListingError {
    pub code: ProductErrorCode,
    pub message: Option<String>,
    pub field: Option<String>,
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

#[derive(cynic::Scalar, Debug, Clone)]
pub struct PositiveDecimal(pub Decimal);
