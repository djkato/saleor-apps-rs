use serde::Serialize;

#[cynic::schema("saleor")]
mod schema {}

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
pub struct ShippingZone {
    pub id: cynic::Id,
    #[arguments(key: "heureka_courierid")]
    pub metafield: Option<String>,
    pub shipping_methods: Option<Vec<ShippingMethodType>>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ShippingMethodType {
    pub minimum_order_weight: Option<Weight>,
    pub maximum_order_weight: Option<Weight>,
    pub channel_listings: Option<Vec<ShippingMethodChannelListing>>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Weight {
    pub value: f64,
    pub unit: WeightUnitsEnum,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct ShippingMethodChannelListing {
    pub price: Option<Money>,
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
    pub media: Option<Vec<ProductMedia>>,
    pub pricing: Option<VariantPricingInfo>,
    pub product: Product,
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
    pub variants: Option<Vec<ProductVariant2>>,
    pub name: String,
    pub description: Option<Jsonstring>,
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "ProductVariant")]
pub struct ProductVariant2 {
    pub sku: Option<String>,
    pub id: cynic::Id,
    pub name: String,
    pub media: Option<Vec<ProductMedia>>,
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

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum WeightUnitsEnum {
    G,
    Lb,
    Oz,
    Kg,
    Tonne,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "JSONString")]
pub struct Jsonstring(pub String);

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

fragment PageInfo on PageInfo {
  hasNextPage
  endCursor
}

fragment CategoryData on Category {
  name
  id
  metafield(key: "heureka_categorytext")
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
  category {
    ...CategoryData
  }
}
"#;
