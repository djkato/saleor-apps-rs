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
  }
}

fragment ProductVariantData on ProductVariant {
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
  product {
    name
    description
    productType {
      metafield(key: "heureka_categorytext")
    }
    category {
      metafield(key: "heureka_categorytext")
    }
  }
}

fragment CategoryData on Category {
  metafield(key: "heureka_categorytext")
}

fragment ProductData on Product {
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
  productType {
    metafield(key: "heureka_categorytext")
  }
  category {
    metafield(key: "heureka_categorytext")
  }
}
"#;

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Subscription")]
pub struct QueryProductsChanged {
    pub event: Option<Event>,
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
    pub name: String,
    pub sku: Option<String>,
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

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Product")]
pub struct Product2 {
    pub variants: Option<Vec<ProductVariant2>>,
    pub name: String,
    pub description: Option<Jsonstring>,
    pub product_type: ProductType,
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "ProductVariant")]
pub struct ProductVariant2 {
    pub id: cynic::Id,
    pub name: String,
    pub media: Option<Vec<ProductMedia>>,
    pub pricing: Option<VariantPricingInfo>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct VariantPricingInfo {
    pub price: Option<TaxedMoney>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct TaxedMoney {
    pub gross: Money,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductMedia {
    #[arguments(format: "WEBP", size: 1024)]
    pub url: String,
    pub alt: String,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Product {
    pub name: String,
    pub description: Option<Jsonstring>,
    pub product_type: ProductType,
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct ProductType {
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Money {
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

#[derive(cynic::QueryFragment, Debug, Clone)]
pub struct Category {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category3>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category3 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category4>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category4 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category5>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category5 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category6>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category6 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category7>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category7 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category8>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category8 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category9>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category9 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category10>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category10 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category11>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category11 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category12>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category12 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category13>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category13 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
    pub parent: Option<Category14>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category14 {
    pub name: String,
    pub id: cynic::Id,
    #[arguments(key: "heureka_categorytext")]
    pub metafield: Option<String>,
}

#[derive(cynic::QueryFragment, Debug, Clone)]
#[cynic(graphql_type = "Category")]
pub struct Category2 {
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
    #[cynic(fallback)]
    Unknown,
}

#[derive(cynic::Enum, Copy, Debug, Clone)]
pub enum ThumbnailFormatEnum {
    Original,
    Avif,
    Webp,
}

#[derive(cynic::Scalar, Debug, Clone)]
#[cynic(graphql_type = "JSONString")]
pub struct Jsonstring(pub String);
