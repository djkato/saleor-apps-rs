/*
query getProductsInitial($id: ID!, $channel: String!) {
  category(id: $id) {
    slug
    id
    updatedAt
    products(first: 50, channel: $channel) {
      pageInfo {
        hasNextPage
        endCursor
      }
      edges {
        node {
          ...ProductData
        }
      }
      totalCount
    }
  }
}

query getProductsNext($after: String!, $channel: String!) {
  products(first: 50, after: $after, channel: $channel) {
    pageInfo {
      hasNextPage
      endCursor
    }
    edges {
      node {
        ...ProductData
      }
    }
  }
}

fragment ProductData on Product {
  variant {
    id
    name
    pricing {
      price {
        gross {
          amount
        }
      }
    }
  }
  category {
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
  name
  description
  media {
    url(format: WEBP, size: 1024)
    alt
  }
  productType {
    metafield(key: "heureka_categorytext")
  }
  category {
    metafield(key: "heureka_categorytext")
  }
}
*/
