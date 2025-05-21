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
    #[arguments(key: "heureka_courierid")]
    pub metafield: Option<String>,
    #[cynic(flatten)]
    pub shipping_methods: Vec<ShippingMethodType>,
    pub id: cynic::Id,
}

#[derive(cynic::QueryFragment, Debug, Serialize, Clone)]
pub struct ShippingMethodType {
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

impl ShippingZone {
    pub fn into_deliveries(
        self,
        variant_weight: Weight,
        is_shipping_cod: bool,
        shipping_price_cod_extra: Option<Decimal>,
    ) -> Result<Delivery, EventHandlerError> {
        let mut delivery_price = None;

        for method in self.shipping_methods {
            let min =
                method
                    .minimum_order_weight
                    .ok_or(EventHandlerError::ShippingZoneMisconfigured(
                        "Missing max weight in shipping method".to_string(),
                    ))?;

            let max =
                method
                    .maximum_order_weight
                    .ok_or(EventHandlerError::ShippingZoneMisconfigured(
                        "Missing max weight in shipping method".to_string(),
                    ))?;

            let price = method
                .channel_listings
                .get(0)
                .and_then(|c| c.price.clone())
                .and_then(|c| rust_decimal::Decimal::from_f64_retain(c.amount))
                .ok_or(EventHandlerError::ShippingZoneMisconfigured(
                    "Missing price in shipping method".to_string(),
                ))?;

            if min.value > variant_weight.value && variant_weight.value > max.value {
                delivery_price = Some(price);
            }
        }

        let delivery_price = delivery_price.ok_or(EventHandlerError::ShippingZoneMisconfigured(
            "Missing price in shipping method".to_string(),
        ))?;

        let courier_id = self
            .metafield
            .ok_or(EventHandlerError::ShippingZoneMisconfigured(
                "Missing courier id in metadata".to_string(),
            ))?;

        let delivery_id = DeliveryCourierId::from_str(&courier_id).map_err(|e| {
            EventHandlerError::ShippingZoneMisconfigured(format!(
                "Courier ID doesn't match heureka delivery courier ID, {}",
                e.to_string()
            ))
        })?;

        Ok(Delivery {
            delivery_price_cod: match is_shipping_cod {
                true => Some(
                    shipping_price_cod_extra.map_or_else(|| delivery_price, |e| delivery_price + e),
                ),
                false => None,
            },
            delivery_price,
            delivery_id,
        })
    }
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
