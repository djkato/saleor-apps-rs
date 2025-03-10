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
        name
        variants {
          id
          name
          sku
          channelListings {
            price {
              amount
              currency
            }
            costPrice {
              amount
              currency
            }
            margin
            channel {
              id
            }
            preorderThreshold {
              quantity
              soldUnits
            }
          }
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
            onSale
            price {
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
              seoTitle
              seoDescription
              description
            }
            availableForPurchaseAt
            isAvailableForPurchase
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
            }
            externalReference
            id
            availableForPurchase
            id
            seoTitle
            seoDescription
            name
            description
            productType {
              id
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

#[derive(cynic::QueryVariables, Clone, Debug)]
pub struct GetProductsNextVariables<'a> {
    pub after: Option<&'a str>,
    pub channel: Option<&'a str>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Query", variables = "GetProductsNextVariables")]
pub struct GetProductsNext {
    #[arguments(first: 50, after: $after, channel: $channel)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct ProductCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct ProductCountableEdge {
    pub node: Product,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct Product {
    pub variants: Option<Vec<ProductVariant>>,
    pub name: String,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct ProductVariant {
    pub id: cynic::Id,
    pub name: String,
    pub sku: Option<String>,
    pub channel_listings: Option<Vec<ProductVariantChannelListing>>,
    pub created: DateTime,
    pub updated_at: DateTime,
    pub quantity_limit_per_customer: Option<i32>,
    pub margin: Option<i32>,
    pub external_reference: Option<String>,
    pub preorder: Option<PreorderData>,
    pub pricing: Option<VariantPricingInfo>,
    pub product: Product2,
    pub quantity_available: Option<i32>,
    pub digital_content: Option<DigitalContent>,
    pub weight: Option<Weight>,
    pub track_inventory: bool,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct VariantPricingInfo {
    pub on_sale: Option<bool>,
    pub price: Option<TaxedMoney>,
    pub price_undiscounted: Option<TaxedMoney>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct TaxedMoney {
    pub gross: Money,
    pub net: Money,
    pub tax: Money2,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct ProductVariantChannelListing {
    pub price: Option<Money2>,
    pub cost_price: Option<Money2>,
    pub margin: Option<i32>,
    pub channel: Channel,
    pub preorder_threshold: Option<PreorderThreshold>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Channel {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Product")]
pub struct Product2 {
    pub collections: Option<Vec<Collection>>,
    pub available_for_purchase_at: Option<DateTime>,
    pub is_available_for_purchase: Option<bool>,
    pub tax_class: Option<TaxClass>,
    pub external_reference: Option<String>,
    pub id: cynic::Id,
    pub available_for_purchase: Option<Date>,
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

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct Weight {
    pub value: f64,
    pub unit: WeightUnitsEnum,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct ProductType {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    pub has_variants: bool,
    pub is_shipping_required: bool,
    pub is_digital: bool,
    pub weight: Option<Weight2>,
    pub kind: ProductTypeKindEnum,
    pub tax_class: Option<TaxClass2>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "TaxClass")]
pub struct TaxClass2 {
    pub id: cynic::Id,
    pub name: String,
    pub countries: Vec<TaxClassCountryRate>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Weight")]
pub struct Weight2 {
    pub unit: WeightUnitsEnum,
    pub value: f64,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct TaxClass {
    pub id: cynic::Id,
    pub name: String,
    pub countries: Vec<TaxClassCountryRate>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct TaxClassCountryRate {
    pub country: CountryDisplay,
    pub rate: f64,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct PreorderThreshold {
    pub quantity: Option<i32>,
    pub sold_units: i32,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct PreorderData {
    pub end_date: Option<DateTime>,
    pub global_threshold: Option<i32>,
    pub global_sold_units: i32,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct Money {
    pub currency: String,
    pub amount: f64,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
#[cynic(graphql_type = "Money")]
pub struct Money2 {
    pub amount: f64,
    pub currency: String,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct DigitalContent {
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct CountryDisplay {
    pub code: String,
    pub country: String,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct Collection {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub description: Option<Jsonstring>,
}

#[derive(cynic::QueryFragment, Clone, Debug)]
pub struct Category {
    pub id: cynic::Id,
    pub name: String,
    pub slug: String,
    pub seo_title: Option<String>,
    pub seo_description: Option<String>,
    pub level: i32,
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

#[derive(cynic::Scalar, Debug, Clone, PartialEq)]
#[cynic(graphql_type = "JSONString")]
pub struct Jsonstring(pub String);
