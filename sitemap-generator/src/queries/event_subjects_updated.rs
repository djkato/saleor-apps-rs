use serde::Serialize;

#[cynic::schema("saleor")]
mod schema {}

pub const EVENTS_QUERY: &str = r#"
subscription QueryProductsChanged {
  event {
    ... on ProductUpdated {
      product {
        ...BaseProduct
      }
    }
    ... on ProductCreated {
      product {
        ...BaseProduct
      }
    }
    ... on ProductDeleted {
      product {
        ...BaseProduct
      }
    }
    ... on CategoryCreated {
      category {
        ...BaseCategory
      }
    }
    ... on CategoryUpdated {
      category {
        ...BaseCategory
      }
    }
    ... on CategoryDeleted {
      category {
        ...BaseCategory
      }
    }
    ... on PageCreated {
      page {
        slug
        id
      }
    }
    ... on PageUpdated {
      page {
        slug
        id
      }
    }
    ... on PageDeleted {
      page {
        slug
        id
      }
    }
    ... on CollectionCreated {
      collection {
        id
        slug
      }
    }
    ... on CollectionUpdated {
      collection {
        id
        slug
      }
    }
    ... on CollectionDeleted {
      collection {
        id
        slug
      }
    }
  }
}

fragment BaseCategory on Category {
  id
  slug
  products(first: 100) {
    pageInfo {
      endCursor
      hasNextPage
    }
    edges {
      node {
        id
        slug
      }
    }
  }
}

fragment BaseProduct on Product {
  id
  slug
  category {
    slug
    id
  }
}
"#;

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Subscription")]
pub struct QueryProductsChanged {
    pub event: Option<Event>,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
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

#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Product {
    pub id: cynic::Id,
    pub slug: String,
    pub category: Option<Category>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageUpdated {
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageDeleted {
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageCreated {
    pub page: Option<Page>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Page {
    pub slug: String,
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CollectionUpdated {
    pub collection: Option<Collection>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CollectionDeleted {
    pub collection: Option<Collection>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CollectionCreated {
    pub collection: Option<Collection>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Collection {
    pub id: cynic::Id,
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CategoryUpdated {
    pub category: Option<Category2>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CategoryDeleted {
    pub category: Option<Category2>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct CategoryCreated {
    pub category: Option<Category2>,
}

#[derive(cynic::QueryFragment, Debug, Serialize)]
pub struct Category {
    pub slug: String,
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Category")]
pub struct Category2 {
    pub id: cynic::Id,
    pub slug: String,
    #[arguments(first: 100)]
    pub products: Option<ProductCountableConnection>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductCountableConnection {
    pub page_info: PageInfo,
    pub edges: Vec<ProductCountableEdge>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ProductCountableEdge {
    pub node: Product2,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Product")]
pub struct Product2 {
    pub id: cynic::Id,
    pub slug: String,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct PageInfo {
    pub end_cursor: Option<String>,
    pub has_next_page: bool,
}

#[derive(cynic::InlineFragments, Debug)]
pub enum Event {
    ProductUpdated(ProductUpdated),
    ProductCreated(ProductCreated),
    ProductDeleted(ProductDeleted),
    CategoryCreated(CategoryCreated),
    CategoryUpdated(CategoryUpdated),
    CategoryDeleted(CategoryDeleted),
    PageCreated(PageCreated),
    PageUpdated(PageUpdated),
    PageDeleted(PageDeleted),
    CollectionCreated(CollectionCreated),
    CollectionUpdated(CollectionUpdated),
    CollectionDeleted(CollectionDeleted),
    #[cynic(fallback)]
    Unknown,
}
