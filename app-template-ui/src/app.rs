use crate::error_template::{AppError, ErrorTemplate};
use crate::routes::extensions::order_to_pdf::Pdf;
use crate::routes::home::Home;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use saleor_app_sdk::bridge::AppBridge;

#[derive(Params, PartialEq)]
pub struct UrlAppParams {
    slug: String,
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let app_bridge = AppBridge::new(Some(true)).unwrap();
    view! {
        <Stylesheet id="leptos" href="/pkg/saleor-app-template-ui.css" />

        // sets the document title
        <Title text="Example UI App template in Rust" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <main class="p-4 md:p-8 md:px-16">
                <Routes>
                    <Route path="/" view=Home />
                    <Route path="/extensions/order_to_pdf" view=Pdf />
                </Routes>
            </main>
        </Router>
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, axum::extract::FromRef)]
pub struct AppState {
    pub saleor_app: std::sync::Arc<tokio::sync::Mutex<saleor_app_sdk::SaleorApp>>,
    pub config: saleor_app_sdk::config::Config,
    pub manifest: saleor_app_sdk::manifest::AppManifest,
    pub leptos_options: LeptosOptions,
}
