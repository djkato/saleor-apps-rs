use crate::{
    app::{AppSettings, AppState},
    queries::products_variants_categories::{
        Category, CategoryCreated, CategoryDeleted, CategoryUpdated, GetCategoryParent,
        GetCategoryParentVariables, GetProductsInitial, GetProductsInitialVariables,
        GetProductsNext, GetProductsNextVariables, Product, ProductCreated, ProductDeleted,
        ProductUpdated, ProductVariant, ProductVariant2, ProductVariantCreated,
        ProductVariantDeleted, ProductVariantUpdated, ShippingZone, ShippingZoneCreated,
        ShippingZoneDeleted, ShippingZoneUpdated,
    },
    server::{
        clear_relations_categorises, clear_relations_parents_in, clear_relations_parents_out,
        clear_relations_varies,
        graphqls::{get_all_products, get_category_parents},
    },
};
use cynic::{GraphQlError, QueryBuilder, http::SurfExt};
use heureka_xml_feed::Shop;
use saleor_app_sdk::{AuthData, apl::AplError};
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
pub enum EventHandlerError {
    ///(Product, Variant/Category)
    #[error("Product is missing an important relation, like category or variant on product, {0:?}")]
    ProductMissingRelation(MissingRelation),
    #[error("Issue during graphql operation, {0}")]
    GraphQl(#[from] GraphQlError),
    #[error("Error fetching APL token, {0}")]
    Apl(#[from] AplError),
    #[error("Error during graphql operation, {0}")]
    Surf(surf::Error),
    #[error("Failed surrealdb query: {0}")]
    SurrealDB(#[from] surrealdb::Error),
    #[error("Failed surrealdb query: {0}")]
    HeurekaXml(#[from] heureka_xml_feed::Error),
}

impl From<surf::Error> for EventHandlerError {
    fn from(err: surf::Error) -> Self {
        EventHandlerError::Surf(err)
    }
}

#[derive(Debug, Clone)]
pub enum MissingRelation {
    Variant,
    Category,
}

impl EventHandler {
    async fn regenerate(&mut self, ev: RegenerateEvent) -> Result<(), Vec<EventHandlerError>> {
        debug!("starting database regeneration (querying all products)");
        let mut errors: Vec<EventHandlerError> = vec![];

        let token = ev
            .state
            .saleor_app
            .lock()
            .await
            .apl
            .get(&ev.saleor_api_url)
            .await
            .map_err(|e| vec![e.into()])?;

        let all_products = get_all_products(&ev.saleor_api_url, &token.token)
            .await
            .map_err(|e| vec![e.into()])?;

        /* SAVE THEM TO DB */
        let db = &mut self.db_handle;
        for product in all_products {
            if let Err(e) = save_product_and_category_to_db(&product, &ev, &token, db).await {
                errors.push(e);
            }

            for variant in
                product
                    .clone()
                    .variants
                    .ok_or(vec![EventHandlerError::ProductMissingRelation(
                        MissingRelation::Variant,
                    )])?
            {
                if let Err(e) = save_variants_to_db(&variant, db, &product).await {
                    errors.push(e);
                }
            }
        }

        debug!("Database regenerated!");
        debug!("Validating XML from DB");

        let (_, mut db_to_xml_errors) = self.db_to_xml().await;
        if !db_to_xml_errors.is_empty() {
            error!("Errors during db_to_xml(), {:?}", &db_to_xml_errors);
            errors.append(&mut db_to_xml_errors);
        }

        match errors.is_empty() {
            true => Ok(()),
            false => Err(errors),
        }
    }

    async fn product_updated_or_created(&self, _data: Product) {
        info!("called!")
    }

    async fn variant_updated_or_created(&self, _data: ProductVariant) {
        info!("called!")
    }

    async fn any_deleted(&self, _id: &cynic::Id, _typ: AnyDeletedType) {
        info!("called!")
    }

    async fn category_updated_or_created(&self, _data: Category) {
        info!("called!")
    }

    async fn shipping_zone_updated_or_created(&self, _data: ShippingZone) {
        info!("called!")
    }

    async fn db_to_xml(&self) -> (String, Vec<EventHandlerError>) {
        let db = &mut self.db_handle;
        let products: Vec<Product> = db.select("product").await?;

        let mut shopitems: Vec<ShopItem> = vec![];
        for mut product in products {
            let variants: Vec<ProductVariant2> = db
                .query(format!(
                    "SELECT * FROM variant WHERE product:{}<-varies<-variant",
                    product.id.inner().to_owned()
                ))
                .await?
                .take(0)?;

            let categories: Vec<Category> = db
                .query(format!(
                    "SELECT * FROM category WHERE product:{}<-categorises<-category LIMIT 1",
                    product.id.inner().to_owned()
                ))
                .await?
                .take(0)?;

            let mut category = categories.into_iter().nth(0);

            if let Some(base_category) = &mut category {
                let mut parent_category: Option<Category> = db
                    .query(format!(
                        "SELECT * FROM category WHERE category:{}<-parents<-category",
                        base_category.id.inner().to_owned()
                    ))
                    .await?
                    .take(0)?;

                while let Some(category) = parent_category {
                    parent_category = db
                        .query(format!(
                            "SELECT * FROM category WHERE category:{}<-parents<-category",
                            category.id.inner().to_owned()
                        ))
                        .await?
                        .take(0)?;
                }
            }
            // variants and categories that are present with product in db aren't being updated, only
            // the tables are. Just cba to strip the db of these parts
            product.category = category;
            product.variants = match variants.is_empty() {
                true => None,
                false => Some(variants),
            };
            shopitems.append(&mut try_shopitem_from_product(
                product.clone(),
                deliveries.clone(),
                url_template.clone(),
                &product,
                settings.clone(),
            )?);
        }
    }
}

async fn save_product_and_category_to_db(
    product: &Product,
    ev: &RegenerateEvent,
    token: &AuthData,
    db: &mut Surreal<Any>,
) -> Result<(), EventHandlerError> {
    debug!(
        "inserting product {}:{} into db",
        &product.name,
        &product.id.inner()
    );
    let _: Option<Product> = db
        .upsert(("product", product.id.inner().to_owned()))
        .content(product.clone())
        .await?;

    let category = product
        .clone()
        .category
        .ok_or(EventHandlerError::ProductMissingRelation(
            MissingRelation::Category,
        ))?;

    let all_category_parents =
        get_category_parents(&category, &ev.saleor_api_url, &token.token).await?;

    for parent in all_category_parents {
        debug!(
            "inserting category {}:{} into db",
            &parent.name,
            &parent.id.inner()
        );
        let _: Option<Category> = db
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

        clear_relations_parents_in(db, category.id.inner()).await?;
        clear_relations_parents_out(db, parent.id.inner()).await?;

        debug!(
            "relating category {}:{} -> parents -> category {}:{}",
            &category.name,
            &category.id.inner(),
            &parent.name,
            &parent.id.inner()
        );

        db.query(format!(
            "RELATE category(parent):{} -> parents -> category:{}",
            parent.id.inner().to_owned(),
            category.id.inner().to_owned(),
        ))
        .await?;
    }
    Ok(())
}

async fn save_variants_to_db(
    variant: &ProductVariant2,
    db: &mut Surreal<Any>,
    product: &Product,
) -> Result<(), EventHandlerError> {
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
    Ok(())
}

enum AnyDeletedType {
    Product,
    Category,
    Variant,
    ShippingZone,
}
