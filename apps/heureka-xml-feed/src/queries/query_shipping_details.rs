#[cynic::schema("saleor")]
mod schema {}

#[derive(cynic::QueryVariables, Debug)]
pub struct DefaultShippingZoneVariables<'a> {
    pub channel: &'a str,
    pub id: &'a cynic::Id,
}

#[derive(cynic::QueryFragment, Debug)]
#[cynic(graphql_type = "Query", variables = "DefaultShippingZoneVariables")]
pub struct DefaultShippingZone {
    #[arguments(id: $id, channel: $channel)]
    pub shipping_zone: Option<ShippingZone>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ShippingZone {
    #[arguments(key: "heureka_courierid")]
    pub metafield: Option<String>,
    pub shipping_methods: Option<Vec<ShippingMethodType>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ShippingMethodType {
    pub minimum_order_weight: Option<Weight>,
    pub maximum_order_weight: Option<Weight>,
    pub channel_listings: Option<Vec<ShippingMethodChannelListing>>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct ShippingMethodChannelListing {
    pub price: Option<Money>,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Weight {
    pub value: f64,
    pub unit: WeightUnitsEnum,
}

#[derive(cynic::QueryFragment, Debug)]
pub struct Money {
    pub currency: String,
    pub amount: f64,
}

#[derive(cynic::Enum, Clone, Copy, Debug)]
pub enum WeightUnitsEnum {
    G,
    Lb,
    Oz,
    Kg,
    Tonne,
}

/*
query DefaultShippingZone($id: ID!, $channel: String!) {
  shippingZone(id: $id, channel: $channel) {
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
}
*/
