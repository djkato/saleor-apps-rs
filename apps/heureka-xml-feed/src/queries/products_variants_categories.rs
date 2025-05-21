use serde::{Deserialize, Serialize};

use crate::server::TryIntoShopItemError;

use super::{
    query_shipping_details::{ShippingZone, Weight},
    schema,
};

const DEFAULT_WEIGHT: Weight = Weight {
    value: 0.1,
    unit: super::query_shipping_details::WeightUnitsEnum::Kg,
};

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetProductsNextVariables<'a> {
    pub after: &'a str,
    pub channel: &'a str,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetCategoryParentVariables<'a> {
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetProductsInitialVariables<'a> {
    pub channel: &'a str,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Subscription")]
pub struct QueryProductsChanged {
    pub event: Option<Event>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ShippingZoneUpdated {
    pub shipping_zone: Option<ShippingZone>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ShippingZoneDeleted {
    pub shipping_zone: Option<ShippingZone>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ShippingZoneCreated {
    pub shipping_zone: Option<ShippingZone>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "GetProductsInitialVariables")]
pub struct GetProductsInitial {
    #[arguments(first: 100, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "GetProductsNextVariables")]
pub struct GetProductsNext {
    #[arguments(first: 100, after: $after, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "GetCategoryParentVariables")]
pub struct GetCategoryParent {
    #[arguments(id: $id)]
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductVariantUpdated {
    pub product_variant: Option<ProductVariant>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductVariantDeleted {
    pub product_variant: Option<ProductVariant>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductVariantCreated {
    pub product_variant: Option<ProductVariant>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductVariant {
    pub sku: Option<String>,
    pub id: cynic::Id,
    pub name: String,
    #[cynic(flatten)]
    pub media: Vec<ProductMedia>,
    pub pricing: Option<VariantPricingInfo>,
    pub product: Product,
    pub weight: Option<Weight>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductUpdated {
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductDeleted {
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductCreated {
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductCountableEdge {
    pub node: Product,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Product {
    pub id: cynic::Id,
    #[cynic(flatten)]
    pub variants: Vec<ProductVariant2>,
    pub name: String,
    pub description: Option<Jsonstring>,
    pub category: Option<Category>,
    pub weight: Option<Weight>,
    pub product_type: Option<ProductType>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "ProductType")]
pub struct ProductType {
    pub weight: Option<Weight>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "ProductVariant")]
pub struct ProductVariant2 {
    pub sku: Option<String>,
    pub weight: Option<Weight>,
    pub id: cynic::Id,
    pub name: String,
    #[cynic(flatten)]
    pub media: Vec<ProductMedia>,
    pub pricing: Option<VariantPricingInfo>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct VariantPricingInfo {
    pub price: Option<TaxedMoney>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct TaxedMoney {
    pub gross: Money2,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ProductMedia {
    #[arguments(format: "WEBP", size: 1024)]
    pub url: String,
    pub alt: String,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Money {
    pub currency: String,
    pub amount: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Money")]
pub struct Money2 {
    pub amount: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct CategoryUpdated {
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct CategoryDeleted {
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct CategoryCreated {
    pub category: Option<Category>,
}

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetCategoryChildrenVariables<'a> {
    pub after: Option<&'a str>,
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "GetCategoryChildrenVariables")]
pub struct GetCategoryChildren {
    #[arguments(id: $id)]
    pub category: Option<GetChildrenCategory>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(variables = "GetCategoryChildrenVariables")]
#[cynic(graphql_type = "Category")]
pub struct GetChildrenCategory {
    #[arguments(first: 100, after: $after)]
    pub children: Option<GetChildrenCategoryCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "CategoryCountableConnection")]
pub struct GetChildrenCategoryCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<GetChildrenCategoryCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "CategoryCountableEdge")]
pub struct GetChildrenCategoryCountableEdge {
    pub node: Category,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Category {
    pub parent: Option<Category2>,
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category2 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
}

#[derive(cynic::InlineFragments, Debug, Clone, Serialize)]
pub enum Event {
    ProductUpdated(ProductUpdated),
    ProductCreated(ProductCreated),
    ProductDeleted(ProductDeleted),
    ProductVariantCreated(ProductVariantCreated),
    ProductVariantUpdated(ProductVariantUpdated),
    ProductVariantDeleted(ProductVariantDeleted),
    CategoryCreated(CategoryCreated),
    CategoryUpdated(CategoryUpdated),
    CategoryDeleted(CategoryDeleted),
    ShippingZoneCreated(ShippingZoneCreated),
    ShippingZoneUpdated(ShippingZoneUpdated),
    ShippingZoneDeleted(ShippingZoneDeleted),
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum ThumbnailFormatEnum {
    Original,
    Avif,
    Webp,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "JSONString")]
pub struct Jsonstring(pub String);

impl ProductVariant2 {
    pub fn get_weight(self, product: Product) -> Weight {
        self.weight.unwrap_or(
            product.weight.unwrap_or(
                product
                    .product_type
                    .and_then(|t| t.weight)
                    .unwrap_or(DEFAULT_WEIGHT),
            ),
        )
    }
}

impl Jsonstring {
    pub fn to_string(&self) -> Result<String, TryIntoShopItemError> {
        let mut out = String::new();

        let editorjs: EditorJsJson = serde_json::from_str(&self.0)?;

        for block in editorjs.blocks {
            out = out
                + "\n"
                + &block.data.text.clone().unwrap_or(
                    //If block has items instead of text, it's likely a list. What kind?
                    //I don't care :) you get dashes not numbers
                    block
                        .data
                        .items
                        .clone()
                        .map(|mut i| {
                            i.first_mut().and_then(|s| {
                                *s = format!("- {s}");
                                Some(s)
                            });
                            i.join("\n")
                        })
                        .unwrap_or("".to_string()),
                );
        }
        Ok(out)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditorJsJson {
    pub blocks: Vec<EditorJsBlock>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditorJsBlock {
    pub data: EditorJsData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EditorJsData {
    pub text: Option<String>,
    pub items: Option<Vec<String>>,
}

pub const EVENTS_QUERY: &str = r#"
subscription QueryProductsChanged {
  event {
    ... on ProductUpdated {
      product {
        ...ProductData
      }
    }
    ... on ProductCreated {
      product {
        ...ProductData
      }
    }
    ... on ProductDeleted {
      product {
        ...ProductData
      }
    }
    ... on ProductVariantCreated {
      productVariant {
        ...ProductVariant
      }
    }
    ... on ProductVariantUpdated {
      productVariant {
        ...ProductVariant
      }
    }
    ... on ProductVariantDeleted {
      productVariant {
        ...ProductVariant
      }
    }
    ... on CategoryCreated {
      category {
        ...CategoryData
      }
    }
    ... on CategoryUpdated {
      category {
        ...CategoryData
      }
    }
    ... on CategoryDeleted {
      category {
        ...CategoryData
      }
    }
    ... on ShippingZoneCreated {
      shippingZone {
        ...ShippingZoneData
      }
    }
    ... on ShippingZoneUpdated {
      shippingZone {
        ...ShippingZoneData
      }
    }
    ... on ShippingZoneDeleted {
      shippingZone {
        ...ShippingZoneData
      }
    }
  }
}

query getProductsInitial($channel: String!) {
  products(first: 100, channel: $channel) {
    pageInfo {
      ...PageInfo
    }
    edges {
      node {
        ...ProductData
      }
    }
  }
}

query getProductsNext($after: String!, $channel: String!) {
  products(first: 100, after: $after, channel: $channel) {
    pageInfo {
      ...PageInfo
    }
    edges {
      node {
        ...ProductData
      }
    }
  }
}

query getCategoryParent($id: ID!) {
  category(id: $id) {
    ...CategoryData
  }
}

query getCategoryChildren($id:ID!, $after:String) {
	category(id: $id) {
    children(first:100, after:$after) {
      pageInfo{
        ...PageInfo
      }
      edges{
        node{
          ...CategoryData
        }
      }
    }
  }
}

fragment PageInfo on PageInfo {
  hasNextPage
  endCursor
}

fragment CategoryData on Category {
  name
  id
  metafield(key: "heureka_categorytext")
  parent {
    name
    id
    metafield(key: "heureka_categorytext")
  }
}

fragment ShippingZoneData on ShippingZone {
  metafield(key: "heureka_courierid")
  shippingMethods {
    minimumOrderWeight {
      ...Weight
    }
    maximumOrderWeight {
      ...Weight
    }
    channelListings {
      price {
        ...Price
      }
    }
  }
}

fragment Weight on Weight {
  value
  unit
}

fragment Price on Money {
  currency
  amount
}

fragment ProductVariant on ProductVariant {
  ...ProductVariantDetails
  product {
    ...ProductData
  }
}

fragment ProductVariantDetails on ProductVariant {
  sku
  weight {
    ...Weight
  }
  id
  name
  media {
    url(format: WEBP, size: 1024)
    alt
  }
  pricing {
    price {
      gross {
        amount
      }
    }
  }
}

fragment ProductData on Product {
  id
  variants {
    ...ProductVariantDetails
  }
  name
  description
  weight {
    ...Weight
  }
  productType {
    weight {
        ...Weight
    }
  }
  category {
    ...CategoryData
  }
}
"#;
