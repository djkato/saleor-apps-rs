use std::str::FromStr;

use heureka_xml_feed::{Delivery, DeliveryCourierId};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;
use tracing::{debug, info};

use crate::server::event_handler::EventHandlerError;

const DEFAULT_WEIGHT: Weight = Weight {
    value: 0.1,
    unit: WeightUnitsEnum::Kg,
};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Product {
    pub slug: String,
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
    pub id: RecordId,
    pub name: String,
    pub media: Vec<ProductMedia>,
    pub pricing: Option<VariantPricingInfo>,
    pub weight: Option<Weight>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct VariantPricingInfo {
    pub price: Option<TaxedMoney>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct TaxedMoney {
    pub gross: Money,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
    //TODO: CUrrency
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ProductMedia {
    pub url: String,
    pub alt: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Category {
    pub slug: String,
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
#[serde(rename_all = "UPPERCASE")]
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
    pub metafield: Option<String>,
    pub channel_listings: Vec<ShippingMethodChannelListing>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ShippingMethodChannelListing {
    pub price: Option<Money>,
}

impl ShippingZone {
    pub fn into_deliveries(
        self,
        variant_weight: Weight,
        is_shipping_cod: bool,
        shipping_price_cod_extra: Option<Decimal>,
    ) -> Result<Vec<Delivery>, EventHandlerError> {
        Ok(self
            .shipping_methods
            .into_iter()
            .filter_map(|method| {
                method_into_delivery(
                    method,
                    &variant_weight,
                    &is_shipping_cod,
                    shipping_price_cod_extra,
                )
            })
            .collect::<Vec<_>>())
    }
}

fn method_into_delivery(
    method: ShippingMethodType,
    variant_weight: &Weight,
    is_shipping_cod: &bool,
    shipping_price_cod_extra: Option<Decimal>,
) -> Option<Delivery> {
    let min = method.minimum_order_weight.unwrap_or(Weight {
        unit: WeightUnitsEnum::Kg,
        value: 0.,
    });

    let max = method.maximum_order_weight.unwrap_or(Weight {
        unit: WeightUnitsEnum::Kg,
        value: f64::MAX,
    });

    let price = method
        .channel_listings
        .into_iter()
        .find(|l| {
            l.clone().price.is_some_and(|p| {
                // in dev build allow any currency
                cfg!(debug_assertions) || p.currency == "EUR" || p.currency == "CZK"
            })
        })
        .and_then(|c| c.price.clone())
        .and_then(|c| rust_decimal::Decimal::from_f64_retain(c.amount))?;
    debug!("Got price");

    debug!(
        "{} < {} && {} < {}",
        min.value, variant_weight.value, variant_weight.value, max.value
    );
    let delivery_price = if min.value <= variant_weight.value && variant_weight.value <= max.value {
        price
    } else {
        return None;
    };
    debug!("Variant within weight limits");

    let courier_id = method.metafield?;
    debug!("courier_id: {} in metafield", &courier_id);

    let delivery_id = DeliveryCourierId::from_str(&courier_id).ok()?;
    debug!("got courier_id");

    Some(Delivery {
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
