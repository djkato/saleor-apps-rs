#[cynic::schema("saleor")]
mod schema {}
/*
query getProductsNext($after: String, $channel: String) {
  products(first: 50, after: $after, channel: $channel) {
    pageInfo {
      hasNextPage
      endCursor
    }
    edges {
      node {
        variants {
          id
          name
          sku
          created
          updatedAt
          quantityLimitPerCustomer
          margin
          externalReference
          preorder {
            endDate
            globalThreshold
            globalSoldUnits
          }
          pricing {
            priceUndiscounted {
              gross {
                currency
                amount
              }
              net {
                currency
                amount
              }
              tax {
                amount
                currency
              }
            }
          }
          product {
            collections {
              id
              name
              slug
              metadata {
                key
                value
              }
              privateMetadata {
                key
                value
              }
              seoTitle
              seoDescription
              description
            }
            availableForPurchaseAt
            isAvailableForPurchase
            taxClass {
              id
              name
              metadata {
                key
                value
              }
              privateMetadata {
                key
                value
              }
              countries {
                country {
                  code
                  country
                }
                rate
              }
            }
            externalReference
            id
            availableForPurchase
            id
            privateMetadata {
              key
              value
            }
            metadata {
              key
              value
            }
            seoTitle
            seoDescription
            name
            description
            productType {
              id
              privateMetadata {
                key
                value
              }
              metadata {
                key
                value
              }
              name
              slug
              hasVariants
              isShippingRequired
              isDigital
              weight {
                unit
                value
              }
              kind
              taxClass {
                id
                name
                countries {
                  country {
                    code
                    country
                  }
                  rate
                }
                metadata {
                  key
                  value
                }
                privateMetadata {
                  key
                  value
                }
              }
            }
            slug
            category {
              id
              name
              slug
              seoTitle
              seoDescription
              level
              metadata {
                key
                value
              }
              privateMetadata {
                key
                value
              }
            }
            created
            updatedAt
            weight {
              value
              unit
            }
            rating
            channel
            isAvailable
            id
            name
            description
            seoTitle
            seoDescription
            isAvailable
            isAvailableForPurchase
          }
          metadata {
            key
            value
          }
          privateMetadata {
            key
            value
          }
          quantityAvailable
          digitalContent {
            id
          }
          weight {
            value
            unit
          }
          trackInventory
        }
      }
    }
  }
}
*/

#[derive(cynic::QueryVariables, Debug, Clone)]
pub struct GetProductsNextVariables<'a> {
    pub after: Option<&'a str>,
    pub channel: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Query", variables = "GetProductsNextVariables")]
pub struct GetProductsNext {
    #[arguments(first: 50, after: $after, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductCountableEdge {
    pub node: Product,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Product {
    pub variants: Option<Vec<ProductVariant>>,
    pub name: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductVariant {
    pub id: cynic::Id,
    pub name: String,
    pub sku: Option<String>,
    pub created: DateTime,
    pub updated_at: DateTime,
    pub quantity_limit_per_customer: Option<i32>,
    pub margin: Option<i32>,
    pub external_reference: Option<String>,
    pub preorder: Option<PreorderData>,
    pub pricing: Option<VariantPricingInfo>,
    pub product: Product2,
    pub metadata: Vec<MetadataItem>,
    pub private_metadata: Vec<MetadataItem>,
    pub quantity_available: Option<i32>,
    pub digital_content: Option<DigitalContent>,
    pub weight: Option<Weight>,
    pub track_inventory: bool,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct VariantPricingInfo {
    pub price_undiscounted: Option<TaxedMoney>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TaxedMoney {
    pub gross: Money,
    pub net: Money,
    pub tax: Money2,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Product")]
pub struct Product2 {
    pub collections: Option<Vec<Collection>>,
    pub available_for_purchase_at: Option<DateTime>,
    pub is_available_for_purchase: Option<bool>,
    pub tax_class: Option<TaxClass>,
    pub external_reference: Option<String>,
    pub id: cynic::Id,
    pub available_for_purchase: Option<Date>,
    pub private_metadata: Vec<MetadataItem>,
    pub metadata: Vec<MetadataItem>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub name: String,
    pub description: Option<Jsonstring>,
    pub product_type: ProductType,
    pub slug: String,
    pub category: Option<Category>,
    pub created: DateTime,
    pub updated_at: DateTime,
    pub weight: Option<Weight>,
    pub rating: Option<f64>,
    pub channel: Option<String>,
    pub is_available: Option<bool>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Weight {
    pub value: f64,
    pub unit: WeightUnitsEnum,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductType {
    pub id: cynic::Id,
    pub private_metadata: Vec<MetadataItem>,
    pub metadata: Vec<MetadataItem>,
    pub name: String,
    pub slug: String,
    pub has_variants: bool,
    pub is_shipping_required: bool,
    pub is_digital: bool,
    pub weight: Option<Weight2>,
    pub kind: ProductTypeKindEnum,
    pub tax_class: Option<TaxClass2>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "TaxClass")]
pub struct TaxClass2 {
    pub id: cynic::Id,
    pub name: String,
    pub countries: Vec<TaxClassCountryRate>,
    pub metadata: Vec<MetadataItem>,
    pub private_metadata: Vec<MetadataItem>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Weight")]
pub struct Weight2 {
    pub unit: WeightUnitsEnum,
    pub value: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TaxClass {
    pub id: cynic::Id,
    pub name: String,
    pub metadata: Vec<MetadataItem>,
    pub private_metadata: Vec<MetadataItem>,
    pub countries: Vec<TaxClassCountryRate>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TaxClassCountryRate {
    pub country: CountryDisplay,
    pub rate: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct PreorderData {
    pub end_date: Option<DateTime>,
    pub global_threshold: Option<i32>,
    pub global_sold_units: i32,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Money {
    pub currency: String,
    pub amount: f64,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Money")]
pub struct Money2 {
    pub amount: f64,
    pub currency: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct DigitalContent {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct CountryDisplay {
    pub code: String,
    pub country: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Collection {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    pub metadata: Vec<MetadataItem>,
    pub private_metadata: Vec<MetadataItem>,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub description: Option<Jsonstring>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Category {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub level: i32,
    pub metadata: Vec<MetadataItem>,
    pub private_metadata: Vec<MetadataItem>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct MetadataItem {
    pub key: String,
    pub value: String,
}

#[derive(cynic::Enum, Clone, Copy, Debug, PartialEq)]
pub enum ProductTypeKindEnum {
    Normal,
    GiftCard,
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
pub struct Date(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
pub struct DateTime(pub String);

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "JSONString")]
pub struct Jsonstring(pub String);
