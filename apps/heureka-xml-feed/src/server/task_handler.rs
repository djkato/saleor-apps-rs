use crate::{
    app::{AppSettings, AppState},
    queries::{
        event_products_updated::{
            Category, CategoryCreated, CategoryDeleted, CategoryUpdated, Product2, ProductCreated,
            ProductDeleted, ProductUpdated, ProductVariant, ProductVariantCreated,
            ProductVariantDeleted, ProductVariantUpdated, ShippingZone, ShippingZoneCreated,
            ShippingZoneDeleted, ShippingZoneUpdated,
        },
        query_get_all_products::{
            Category2, GetProductsInitial, GetProductsInitialVariables, GetProductsNext,
            GetProductsNextVariables, Product,
        },
    },
    server::{
        category_iter_parents, clear_relations_categorises, clear_relations_parents_in,
        clear_relations_parents_out, clear_relations_varies,
    },
};
use cynic::{GraphQlError, QueryBuilder, http::SurfExt};
use saleor_app_sdk::apl::AplError;
use surrealdb::{Surreal, engine::any::Any};
use tokio::{sync::mpsc::Receiver, task::JoinHandle};
use tracing::{debug, error, info, warn};

pub struct EventHandler {
    receiver: Receiver<Event>,
    settings: AppSettings,
    db_handle: Surreal<Any>,
}

#[derive(Debug, Clone)]
pub enum Event {
    ProductUpdated(ProductUpdated),
    ProductCreated(ProductCreated),
    ProductDeleted(ProductDeleted),
    CategoryCreated(CategoryCreated),
    CategoryUpdated(CategoryUpdated),
    CategoryDeleted(CategoryDeleted),
    ProductVariantCreated(ProductVariantCreated),
    ProductVariantUpdated(ProductVariantUpdated),
    ProductVariantDeleted(ProductVariantDeleted),
    ShippingZoneCreated(ShippingZoneCreated),
    ShippingZoneUpdated(ShippingZoneUpdated),
    ShippingZoneDeleted(ShippingZoneDeleted),
    Regenerate(RegenerateEvent),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RegenerateEvent {
    pub state: AppState,
    pub saleor_api_url: String,
}

impl EventHandler {
    pub fn start(
        settings: AppSettings,
        receiver: Receiver<Event>,
        db_handle: Surreal<Any>,
    ) -> JoinHandle<()> {
        let s = Self {
            settings,
            receiver,
            db_handle,
        };
        tokio::spawn(s.listen())
    }

    async fn listen(mut self) {
        while let Some(message) = self.receiver.recv().await {
            debug!("received Event: {:?}", &message);
            match message {
                Event::ProductCreated(product_created) => {
                    if let Some(product) = product_created.clone().product {
                        self.product_updated_or_created(product).await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductUpdated(product_updated) => {
                    if let Some(product) = product_updated.clone().product {
                        self.product_updated_or_created(product).await;
                    } else {
                        warn!("Event::ProductCreated/Updated missing data");
                    }
                }
                Event::ProductDeleted(product) => {
                    if let Some(product) = product.product {
                        self.any_deleted(&product.id, AnyDeletedType::Product).await;
                    } else {
                        warn!("Event::ProductDeleted missing data");
                    }
                }
                Event::CategoryCreated(category_created) => {
                    if let Some(category) = category_created.clone().category {
                        self.category_updated_or_created(category).await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryUpdated(category_updated) => {
                    if let Some(category) = category_updated.clone().category {
                        self.category_updated_or_created(category).await;
                    } else {
                        warn!("Event::CategoryCreated/Updated missing data");
                    }
                }
                Event::CategoryDeleted(category) => {
                    if let Some(category) = category.category {
                        self.any_deleted(&category.id, AnyDeletedType::Category)
                            .await;
                    } else {
                        warn!("Event::CategoryDeleted missing data");
                    }
                }
                Event::ProductVariantCreated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.variant_updated_or_created(variant).await;
                    }
                }
                Event::ProductVariantUpdated(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.variant_updated_or_created(variant).await;
                    }
                }
                Event::ProductVariantDeleted(variant) => {
                    if let Some(variant) = variant.product_variant {
                        self.any_deleted(&variant.id, AnyDeletedType::Variant).await;
                    }
                }
                Event::ShippingZoneCreated(shipping_zone) => {
                    if let Some(shipping_zone) = shipping_zone.shipping_zone {
                        self.shipping_zone_updated_or_created(shipping_zone).await;
                    }
                }
                Event::ShippingZoneUpdated(shipping_zone) => {
                    if let Some(shipping_zone) = shipping_zone.shipping_zone {
                        self.shipping_zone_updated_or_created(shipping_zone).await;
                    }
                }
                Event::ShippingZoneDeleted(shipping_zone) => {
                    if let Some(shipping_zone) = shipping_zone.shipping_zone {
                        self.any_deleted(&shipping_zone.id, AnyDeletedType::ShippingZone)
                            .await;
                    }
                }
                Event::Regenerate(r) => {
                    if let Err(e) = self.regenerate(r).await {
                        error!("{:?}", e);
                    };
                }
                Event::Unknown => (),
            }
            debug!("Event succesfully handled");
        }
    }
}

/* =============== Event handlers =============== */
#[derive(thiserror::Error, Debug)]
pub enum RegenerateError {
    #[error("Issue during graphql operation, {0}")]
    GraphQlError(#[from] GraphQlError),
    #[error("Error fetching APL token, {0}")]
    AplError(#[from] AplError),
    #[error("Error during graphql operation, {0}")]
    SurfError(surf::Error),
    #[error("Failed surrealdb query: {0}")]
    SurrealDBError(#[from] surrealdb::Error),
}

impl From<surf::Error> for RegenerateError {
    fn from(err: surf::Error) -> Self {
        RegenerateError::SurfError(err)
    }
}

impl EventHandler {
    async fn regenerate(&mut self, ev: RegenerateEvent) -> Result<(), RegenerateError> {
        /* COLLECT ALL PRODUCTS */
        debug!("starting database regeneration (querying all products)");
        //TODO: Get all channels and perform all this for each channel
        let mut all_products = vec![];

        let token = ev
            .state
            .saleor_app
            .lock()
            .await
            .apl
            .get(&ev.saleor_api_url)
            .await?;

        let res = surf::post(&ev.saleor_api_url)
            .header("authorization-bearer", &token.token)
            .run_graphql(GetProductsInitial::build(GetProductsInitialVariables {
                channel: "",
            }))
            .await?;

        if let Some(e) = res.errors {
            for error in &e {
                error!("Errors during graphql, {:?}", error.message);
            }
            for error in e {
                return Err(error.into());
            }
        }

        let mut next_cursor = None;

        if let Some(products_initial) = res.data
            && let Some(products) = products_initial.products
        {
            all_products.append(
                &mut products
                    .edges
                    .into_iter()
                    .map(|p| p.node)
                    .collect::<Vec<_>>(),
            );
            next_cursor = products.page_info.end_cursor;
        }

        debug!(
            "collected first {} products, is there more? {}",
            all_products.len(),
            next_cursor.is_some()
        );

        while let Some(cursor) = &mut next_cursor {
            let res = surf::post(&ev.saleor_api_url)
                .header("authorization-bearer", &token.token)
                .run_graphql(GetProductsNext::build(GetProductsNextVariables {
                    after: cursor,
                    channel: "",
                }))
                .await?;

            if let Some(e) = res.errors {
                for error in &e {
                    error!("Errors during graphql, {:?}", error.message);
                }
                for error in e {
                    return Err(error.into());
                }
            }

            if let Some(products_next) = res.data
                && let Some(products) = products_next.products
            {
                all_products.append(
                    &mut products
                        .edges
                        .into_iter()
                        .map(|p| p.node)
                        .collect::<Vec<_>>(),
                );
                next_cursor = products.page_info.end_cursor;
            }

            debug!(
                "collected {} products, is there more? {}",
                all_products.len(),
                next_cursor.is_some()
            );
        }
        info!("collected a total of {} products", all_products.len());

        /* SAVE THEM TO DB */
        let db = &mut self.db_handle;
        for product in all_products {
            debug!(
                "inserting product {}:{} into db",
                &product.name,
                &product.id.inner()
            );
            let _: Option<Product> = db
                .upsert(("product", product.id.inner().to_owned()))
                .content(product.clone())
                .await?;

            if let Some(category) = product.category {
                let categories = category_iter_parents(category);

                // To have a loop over (category, prev category), so I can pair existing ones in
                // parent
                let mut passed_first_loop = false;
                let mut shifted_catories = categories.clone();
                shifted_catories.rotate_right(1);

                for (category, prev_category) in
                    categories.clone().into_iter().zip(shifted_catories)
                {
                    debug!(
                        "inserting category {}:{} into db",
                        &category.name,
                        &category.id.inner()
                    );
                    let _: Option<Category2> = db
                        .upsert(("category", category.id.inner()))
                        .content(category.clone())
                        .await?;

                    clear_relations_categorises(db, category.id.inner()).await?;

                    debug!(
                        "relating category {}:{} -> categorises -> product {}:{}",
                        &category.name,
                        &category.id.inner(),
                        &product.name,
                        &product.id.inner()
                    );

                    db.query(format!(
                        "RELATE category:{}->categorises->product:{}",
                        category.id.inner().to_owned(),
                        product.id.inner().to_owned()
                    ))
                    .await?;

                    if passed_first_loop {
                        passed_first_loop = true;
                        continue;
                    }

                    clear_relations_parents_in(db, category.id.inner()).await?;
                    clear_relations_parents_out(db, prev_category.id.inner()).await?;

                    debug!(
                        "relating category {}:{} -> parents -> category {}:{}",
                        &category.name,
                        &category.id.inner(),
                        &prev_category.name,
                        &prev_category.id.inner()
                    );

                    db.query(format!(
                        "RELATE category(parent):{} -> parents -> category:{}",
                        prev_category.id.inner().to_owned(),
                        category.id.inner().to_owned(),
                    ))
                    .await?;
                }
            }

            for variant in product.variants.unwrap_or(vec![]) {
                debug!(
                    "inserting variant {}:{}",
                    &variant.name,
                    &variant.id.inner(),
                );
                let _: Option<ProductVariant> = db
                    .upsert(("variant", variant.id.inner()))
                    .content(variant.clone())
                    .await?;

                clear_relations_varies(db, variant.id.inner()).await?;

                debug!(
                    "relating variant {}:{} -> varies -> product {}:{}",
                    &variant.name,
                    &variant.id.inner(),
                    &product.name,
                    &product.id.inner()
                );

                db.query(format!(
                    "RELATE variant:{}->varies->product:{}",
                    variant.id.inner().to_owned(),
                    product.id.inner().to_owned(),
                ))
                .await?;
            }
        }
        debug!("Database regenerated!");
        Ok(())
    }

    async fn product_updated_or_created(&self, data: Product2) {
        info!("called!")
    }

    async fn variant_updated_or_created(&self, data: ProductVariant) {
        info!("called!")
    }

    async fn any_deleted(&self, id: &cynic::Id, typ: AnyDeletedType) {
        info!("called!")
    }

    async fn category_updated_or_created(&self, data: Category) {
        info!("called!")
    }

    async fn shipping_zone_updated_or_created(&self, data: ShippingZone) {
        info!("called!")
    }
}

enum AnyDeletedType {
    Product,
    Category,
    Variant,
    ShippingZone,
}
