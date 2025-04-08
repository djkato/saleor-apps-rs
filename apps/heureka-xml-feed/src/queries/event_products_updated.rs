use serde::Serialize;

#[cynic::schema("saleor")]
mod schema {}

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
        ...ProductVariantData
      }
    }
    ... on ProductVariantUpdated {
      productVariant {
        ...ProductVariantData
      }
    }
    ... on ProductVariantDeleted {
      productVariant {
        ...ProductVariantData
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

fragment ShippingZoneData on ShippingZone {
  metafield(key: "heureka_courierid")
  shippingMethods {
    minimumOrderWeight {
      value
      unit
    }
    maximumOrderWeight {
      value
      unit
    }
    channelListings {
      price {
        currency
        amount
      }
    }
  }
}

fragment ProductVariantData on ProductVariant {
  id
  name
  sku
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
  product {
    id
    name
    description
    category {
      ...CategoryData
    }
  }
}

fragment ProductData on Product {
  id
  variants {
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
  name
  description
  category {
    ...CategoryData
  }
}

fragment CategoryData on Category {
  name
  id
  metafield(key: "heureka_categorytext")
  parent {
    name
    id
    metafield(key: "heureka_categorytext")
    parent {
      name
      id
      metafield(key: "heureka_categorytext")
      parent {
        name
        id
        metafield(key: "heureka_categorytext")
        parent {
          name
          id
          metafield(key: "heureka_categorytext")
          parent {
            name
            id
            metafield(key: "heureka_categorytext")
            parent {
              name
              id
              metafield(key: "heureka_categorytext")
              parent {
                name
                id
                metafield(key: "heureka_categorytext")
                parent {
                  name
                  id
                  metafield(key: "heureka_categorytext")
                  parent {
                    name
                    id
                    metafield(key: "heureka_categorytext")
                    parent {
                      name
                      id
                      metafield(key: "heureka_categorytext")
                      parent {
                        name
                        id
                        metafield(key: "heureka_categorytext")
                        parent {
                          name
                          id
                          metafield(key: "heureka_categorytext")
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Subscription")]
pub struct QueryProductsChanged {
    pub event: Option<Event>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ShippingZoneUpdated {
    pub shipping_zone: Option<ShippingZone>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ShippingZoneDeleted {
    pub shipping_zone: Option<ShippingZone>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ShippingZoneCreated {
    pub shipping_zone: Option<ShippingZone>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ShippingZone {
    #[arguments(key: "heureka_courierid")]
    pub metafield: Option<String>,
    pub shipping_methods: Option<Vec<ShippingMethodType>>,
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ShippingMethodType {
    pub minimum_order_weight: Option<Weight>,
    pub maximum_order_weight: Option<Weight>,
    pub channel_listings: Option<Vec<ShippingMethodChannelListing>>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Weight {
    pub value: f64,
    pub unit: WeightUnitsEnum,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ShippingMethodChannelListing {
    pub price: Option<Money>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductVariantUpdated {
    pub product_variant: Option<ProductVariant>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductVariantDeleted {
    pub product_variant: Option<ProductVariant>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductVariantCreated {
    pub product_variant: Option<ProductVariant>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductVariant {
    pub id: cynic::Id,
    pub sku: Option<String>,
    pub name: String,
    pub media: Option<Vec<ProductMedia>>,
    pub pricing: Option<VariantPricingInfo>,
    pub product: Product,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductUpdated {
    pub product: Option<Product2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductDeleted {
    pub product: Option<Product2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductCreated {
    pub product: Option<Product2>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Product")]
pub struct Product2 {
    pub variants: Option<Vec<ProductVariant2>>,
    pub name: String,
    pub description: Option<Jsonstring>,
    pub id: cynic::Id,
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
pub struct Product {
    pub id: cynic::Id,
    pub name: String,
    pub description: Option<Jsonstring>,
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Money {
    pub currency: String,
    pub amount: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Money")]
pub struct Money2 {
    pub amount: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CategoryUpdated {
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CategoryDeleted {
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CategoryCreated {
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
pub struct Category {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category2>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category2 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category3>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category3 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category4>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category4 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category5>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category5 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category6>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category6 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category7>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category7 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category8>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category8 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category9>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category9 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category10>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category10 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category11>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category11 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category12>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category12 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category13>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Category")]
pub struct Category13 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
}

#[derive(cynic::InlineFragments, Debug, Clone)]
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
