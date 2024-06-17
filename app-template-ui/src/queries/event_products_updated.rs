#[cynic::schema("saleor")]
mod schema {}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct QueryProductsChanged {
    pub event: Option<Event>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductUpdated {
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductDeleted {
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductCreated {
    pub product: Option<Product>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Product {
    pub id: cynic::Id,
    pub slug: String,
    pub name: String,
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Category {
    pub slug: String,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum Event {
    ProductUpdated(ProductUpdated),
    ProductCreated(ProductCreated),
    ProductDeleted(ProductDeleted),
    #[cynic(fallback)]
    Unknown,
}
