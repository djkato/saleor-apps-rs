use anyhow::bail;
use cynic::{http::SurfExt, MutationBuilder, QueryBuilder};
use evalexpr::{eval_float_with_context, ContextWithMutableVariables, HashMapContext, Value};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use tracing::{debug, error, warn};

use crate::{
    app::AppState,
    queries::{
        get_all_products::{
            GetProductsNext, GetProductsNextVariables, Jsonstring, Product, ProductTypeKindEnum,
            ProductVariant,
        },
        get_channel_id::{GetChannelID, GetChannelIDVariables},
        update_variant_price::{PositiveDecimal, UpdatePrice, UpdatePriceVariables},
    },
};

pub async fn update_prices(state: AppState, saleor_api_url: String) -> anyhow::Result<()> {
    debug!("fetching all products");
    let app = state.saleor_app.lock().await;
    let auth_data = app.apl.get(&saleor_api_url).await?;
    let products =
        get_all_products(&saleor_api_url, &state.target_channel, &auth_data.token).await?;
    let channel_id =
        get_channel_id(&saleor_api_url, &auth_data.token, &state.target_channel).await?;
    debug!("found {} products", products.len(),);
    // dbg!(&products);
    for product in products {
        debug!("Working on product {}", product.name);
        if let Some(variants) = product.variants {
            for variant in variants {
                debug!("Working on variant {}", variant.name);
                // dbg!(&create_context_map(variant.clone()).unwrap());
                match (
                    eval_float_with_context(
                        &state.manipulator.price_expression,
                        &create_context_map(variant.clone()).unwrap(),
                    ),
                    eval_float_with_context(
                        &state
                            .manipulator
                            .cost_price_expression
                            .clone()
                            .unwrap_or("".into()),
                        &create_context_map(variant.clone()).unwrap(),
                    ),
                ) {
                    (Ok(price), Ok(cost_price)) => {
                        set_variant_price(
                            &variant.id,
                            &channel_id,
                            price,
                            Some(cost_price as f32),
                            &saleor_api_url,
                            &auth_data.token,
                        )
                        .await?;
                    }
                    (Ok(price), Err(e)) => {
                        warn!(
                            "(Optional Field) Error during cost price expression processing of '{:?}': {:?}",
                            &variant.name, e
                        );
                        set_variant_price(
                            &variant.id,
                            &channel_id,
                            price,
                            None,
                            &saleor_api_url,
                            &auth_data.token,
                        )
                        .await?;
                    }
                    (Err(e1), _) => {
                        error!(
                            "Error during price expression processing of '{}': {:?}",
                            &variant.name, e1
                        )
                    }
                };
            }
        }
    }
    Ok(())
}

async fn set_variant_price(
    variant: &cynic::Id,
    channel: &cynic::Id,
    price: f64,
    cost_price: Option<f32>,
    saleor_api_url: &str,
    token: &str,
) -> anyhow::Result<()> {
    debug!("setting price'{:?}' for '{:?}'", &price, &variant);

    let operation = UpdatePrice::build(UpdatePriceVariables {
        channel,
        variant,
        cost_price: cost_price
            .and_then(|d| Some(PositiveDecimal(Decimal::from_f32(d).unwrap().round_dp(2)))),
        price: PositiveDecimal(Decimal::from_f64(price).unwrap().round_dp(2)),
    });
    // dbg!(&operation.query, &operation.variables);
    match surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(operation)
        .await
    {
        Ok(res) => {
            if let Some(e) = res.errors {
                for er in e {
                    error!("Error Happened during price setting mutation: {:?}", er)
                }
            }
            if let Some(data) = res.data {
                // dbg!(&data);

                if let Some(data) = data.product_variant_channel_listing_update {
                    // dbg!(&data);
                    for e in data.errors {
                        error!("Error Happened during price setting mutation: {:?}", e)
                    }
                    debug!("Done");
                }
            }
        }
        Err(e) => {
            error!("Error happened during price setting mutation: {:?}", e);
            bail!(e)
        }
    };
    Ok(())
}

async fn get_all_products(
    saleor_api_url: &str,
    channel: &str,
    token: &str,
) -> anyhow::Result<Vec<Product>> {
    let operation = GetProductsNext::build(GetProductsNextVariables {
        channel: Some(channel),
        after: None,
    });
    let mut all_categorised_products: Vec<Product> = vec![];
    let res = surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(operation)
        .await;
    if let Ok(query) = &res
        && let Some(data) = &query.data
        && let Some(products) = &data.products
    {
        all_categorised_products.append(
            &mut products
                .edges
                .clone()
                .into_iter()
                .map(|p| p.node)
                .collect::<Vec<_>>(),
        );
        //Keep fetching next page
        debug!("fetched first products, eg: {:?}", products.edges.first());
        let mut next_cursor = products.page_info.end_cursor.clone();
        while let Some(cursor) = &mut next_cursor {
            let res = surf::post(saleor_api_url)
                .header("authorization-bearer", token)
                .run_graphql(GetProductsNext::build(GetProductsNextVariables {
                    after: Some(cursor),
                    channel: Some(channel),
                }))
                .await;
            if let Ok(query) = &res
                && let Some(data) = &query.data
                && let Some(products) = &data.products
            {
                all_categorised_products.append(
                    &mut products
                        .edges
                        .clone()
                        .into_iter()
                        .map(|p| p.node)
                        .collect::<Vec<_>>(),
                );
                debug!("fetched next products, eg: {:?}", products.edges.first());
                next_cursor.clone_from(&products.page_info.end_cursor);
            } else {
                error!("Failed fetching initial products! {:?}", &res);
                anyhow::bail!("Failed fetching initial products! {:?}", res);
            }
        }
    }
    debug!("All products collected...");
    Ok(all_categorised_products)
}

pub async fn get_channel_id(
    saleor_api_url: &str,
    token: &str,
    channel: &str,
) -> anyhow::Result<cynic::Id> {
    match surf::post(saleor_api_url)
        .header("authorization-bearer", token)
        .run_graphql(GetChannelID::build(GetChannelIDVariables { channel }))
        .await
    {
        Ok(res) => {
            // dbg!(&res);
            if let Some(er) = res.errors {
                for e in er {
                    error!("failed getting channel ID, {:?}", e);
                }
            }
            if let Some(data) = res.data
                && let Some(c) = data.channel
            {
                return Ok(c.id);
            }
        }
        Err(e) => error!("failed getting channel ID, {:?}", e),
    }
    bail!("Failed getting channel ID")
}

pub fn create_context_map(variant: ProductVariant) -> anyhow::Result<HashMapContext> {
    let mut context = HashMapContext::new();
    context.set_value(
        "variant.id".into(),
        Value::from(variant.id.inner().to_owned()),
    )?;
    context.set_value("variant.name".into(), Value::from(variant.name.clone()))?;
    context.set_value(
        "variant.sku".into(),
        Value::from(variant.sku.unwrap_or("".into())),
    )?;
    context.set_value(
        "variant.quantity_limit_per_customer".into(),
        Value::from_int(variant.quantity_limit_per_customer.unwrap_or(0) as i64),
    )?;
    context.set_value(
        "variant.margin".into(),
        Value::from_int(variant.margin.unwrap_or(0) as i64),
    )?;
    context.set_value(
        "variant.external_reference".into(),
        Value::from(variant.external_reference.unwrap_or("".into())),
    )?;
    context.set_value(
        "variant.pricing.currency".into(),
        Value::from(
            variant
                .pricing
                .as_ref()
                .and_then(|p| {
                    p.price_undiscounted
                        .as_ref()
                        .and_then(|n| Some(n.net.currency.clone()))
                })
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.pricing.undiscounted.net.amount".into(),
        Value::from_float(
            variant
                .pricing
                .as_ref()
                .and_then(|v| {
                    v.price_undiscounted
                        .as_ref()
                        .and_then(|t| Some(t.net.amount))
                })
                .unwrap_or(0.),
        ),
    )?;
    context.set_value(
        "variant.pricing.undiscounted.gross.amount".into(),
        Value::from_float(
            variant
                .pricing
                .as_ref()
                .and_then(|v| {
                    v.price_undiscounted
                        .as_ref()
                        .and_then(|t| Some(t.gross.amount))
                })
                .unwrap_or(0.),
        ),
    )?;
    context.set_value(
        "variant.pricing.undiscounted.tax.amount".into(),
        Value::from_float(
            variant
                .pricing
                .and_then(|v| v.price_undiscounted.and_then(|t| Some(t.tax.amount)))
                .unwrap_or(0.),
        ),
    )?;
    context.set_value(
        "variant.product.is_available_for_purchase".into(),
        Value::from(variant.product.is_available_for_purchase.unwrap_or(false)),
    )?;
    context.set_value(
        "variant.product.tax_class.id".into(),
        Value::from(
            variant
                .product
                .tax_class
                .as_ref()
                .and_then(|c| Some(c.id.inner().to_owned()))
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.product.tax_class.name".into(),
        Value::from(
            variant
                .product
                .tax_class
                .as_ref()
                .and_then(|c| Some(c.name.clone()))
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.product.external_reference".into(),
        Value::from(variant.product.external_reference.unwrap_or("".into())),
    )?;
    context.set_value(
        "variant.product.id".into(),
        Value::from(variant.product.id.inner().to_owned()),
    )?;
    context.set_value(
        "variant.product.seo_title".into(),
        Value::from(variant.product.seo_title.unwrap_or("".into())),
    )?;
    context.set_value(
        "variant.product.seo_description".into(),
        Value::from(variant.product.seo_description.unwrap_or("".into())),
    )?;
    context.set_value(
        "variant.product.name".into(),
        Value::from(variant.product.name),
    )?;
    context.set_value(
        "variant.product.description".into(),
        Value::from(
            variant
                .product
                .description
                .unwrap_or(Jsonstring("".into()))
                .0,
        ),
    )?;
    context.set_value(
        "variant.product.product_type.id".into(),
        Value::from(variant.product.product_type.id.inner().to_owned()),
    )?;
    context.set_value(
        "variant.product.product_type.name".into(),
        Value::from(variant.product.product_type.name),
    )?;
    context.set_value(
        "variant.product.product_type.slug".into(),
        Value::from(variant.product.product_type.slug),
    )?;
    context.set_value(
        "variant.product.product_type.is_giftcard".into(),
        Value::from(variant.product.product_type.kind == ProductTypeKindEnum::GiftCard),
    )?;
    context.set_value(
        "variant.product.product_type.is_digital".into(),
        Value::from(variant.product.product_type.is_digital),
    )?;
    context.set_value(
        "variant.product.slug".into(),
        Value::from(variant.product.slug),
    )?;
    context.set_value(
        "variant.product.category.id".into(),
        Value::from(
            variant
                .product
                .category
                .as_ref()
                .and_then(|c| Some(c.id.inner().to_owned()))
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.product.category.name".into(),
        Value::from(
            variant
                .product
                .category
                .as_ref()
                .and_then(|c| Some(c.name.clone()))
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.product.category.slug".into(),
        Value::from(
            variant
                .product
                .category
                .as_ref()
                .and_then(|c| Some(c.slug.clone()))
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.product.category.seo_title".into(),
        Value::from(
            variant
                .product
                .category
                .as_ref()
                .and_then(|c| c.seo_title.clone())
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.product.category.seo_description".into(),
        Value::from(
            variant
                .product
                .category
                .as_ref()
                .and_then(|c| c.seo_description.clone())
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.product.category.level".into(),
        Value::from_int(
            variant
                .product
                .category
                .as_ref()
                .and_then(|c| Some(c.level))
                .unwrap_or(0) as i64,
        ),
    )?;
    context.set_value(
        "variant.product.rating".into(),
        Value::from_float(variant.product.rating.unwrap_or(0.)),
    )?;
    context.set_value(
        "variant.product.channel".into(),
        Value::from(variant.product.channel.unwrap_or("".into())),
    )?;
    context.set_value(
        "variant.product.is_available".into(),
        Value::from(variant.product.is_available.unwrap_or(false)),
    )?;
    context.set_value(
        "variant.quantity_available".into(),
        Value::from_int(variant.quantity_available.unwrap_or(0) as i64),
    )?;
    context.set_value(
        "variant.digital_content".into(),
        Value::from(
            variant
                .digital_content
                .and_then(|c| Some(c.id.inner().to_owned()))
                .unwrap_or("".into()),
        ),
    )?;
    context.set_value(
        "variant.weight".into(),
        Value::from_float(
            variant.weight.and_then(|w| Some(w.value)).unwrap_or(
                variant
                    .product
                    .weight
                    .and_then(|w| Some(w.value))
                    .unwrap_or(
                        variant
                            .product
                            .product_type
                            .weight
                            .and_then(|w| Some(w.value))
                            .unwrap_or(0.),
                    ),
            ),
        ),
    )?;
    context.set_value(
        "variant.track_inventory".into(),
        Value::from(variant.track_inventory),
    )?;
    Ok(context)
}
