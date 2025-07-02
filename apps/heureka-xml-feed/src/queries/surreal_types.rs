use std::str::FromStr;

use heureka_xml_feed::{Delivery, DeliveryCourierId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

use crate::server::event_handler::EventHandlerError;

const DEFAULT_WEIGHT: Weight = Weight {
    value: 0.1,
    unit: WeightUnitsEnum::Kg,
};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Product {
    pub id: RecordId,
    pub name: String,
    pub description: Option<String>,
    pub weight: Option<Weight>,
    pub product_type: Option<ProductType>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ProductType {
    pub weight: Option<Weight>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ProductVariant {
    pub sku: Option<String>,
    pub id: cynic::Id,
    pub name: String,
    pub media: Vec<ProductMedia>,
    pub pricing: Option<VariantPricingInfo>,
    pub product: Product,
    pub weight: Option<Weight>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct VariantPricingInfo {
    pub price: Option<TaxedMoney>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct TaxedMoney {
    pub gross: Money2,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Money2 {
    pub amount: f64,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ProductMedia {
    pub url: String,
    pub alt: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Category {
    pub name: String,
    pub id: RecordId,
    pub metafield: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Weight {
    pub value: f64,
    pub unit: WeightUnitsEnum,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum WeightUnitsEnum {
    G,
    Lb,
    Oz,
    Kg,
    Tonne,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ShippingZone {
    pub metafield: Option<String>,
    pub shipping_methods: Vec<ShippingMethodType>,
    pub id: RecordId,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ShippingMethodType {
    pub minimum_order_weight: Option<Weight>,
    pub maximum_order_weight: Option<Weight>,
    pub channel_listings: Vec<ShippingMethodChannelListing>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ShippingMethodChannelListing {
    pub price: Option<Money2>,
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

impl ProductVariant {
    pub fn get_weight(self, product: Product) -> Weight {
        self.weight.unwrap_or(
            product.weight.unwrap_or(
                product
                    .product_type
                    .and_then(|t| t.weight)
                    .unwrap_or(DEFAULT_WEIGHT),
            ),
        )
    }
}
