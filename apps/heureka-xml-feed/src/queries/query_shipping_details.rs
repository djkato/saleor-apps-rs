use std::str::FromStr;

use heureka_xml_feed::{Delivery, DeliveryCourierId};
use rust_decimal::Decimal;
use serde::Serialize;

use crate::server::event_handler::EventHandlerError;

use super::schema;

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

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
pub struct ShippingZone {
    #[cynic(flatten)]
    pub shipping_methods: Vec<ShippingMethodType>,
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
pub struct ShippingMethodType {
    #[arguments(key: "heureka_courierid")]
    pub metafield: Option<String>,
    pub minimum_order_weight: Option<Weight>,
    pub maximum_order_weight: Option<Weight>,
    #[cynic(flatten)]
    pub channel_listings: Vec<ShippingMethodChannelListing>,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
pub struct ShippingMethodChannelListing {
    pub price: Option<Money>,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
pub struct Weight {
    pub value: f64,
    pub unit: WeightUnitsEnum,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
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
    id
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
