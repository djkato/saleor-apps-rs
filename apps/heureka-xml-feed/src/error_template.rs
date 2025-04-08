#[cfg(feature = "ssr")]
use crate::server::task_handler;
#[cfg(feature = "ssr")]
use axum::response::{IntoResponse, Response};
#[cfg(feature = "ssr")]
use http::header::ToStrError;
use http::status::StatusCode;
use leptos::prelude::*;
use saleor_app_sdk::apl::AplError;
use thiserror::Error;
#[cfg(feature = "ssr")]
use tokio::sync::mpsc::error::SendError;

/* ERROR STUFF FOR AXUM */

#[cfg(feature = "ssr")]
#[derive(Error, Debug)]
pub enum AxumError {
    #[error("Error converting something to string, `{0}`")]
    ToStrError(#[from] ToStrError),
    #[error("Error converting something from json, `{0}`")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("Anyhow function error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("Request is missing header `{0}`")]
    MissingHeader(String),
    #[error("Internal server error, `{0}`")]
    InternalServerError(String),
    #[error("Internal server error with APL, `{0}`")]
    AplError(#[from] AplError),
    #[error("Failed sending task to task handler, `{0}`")]
    SendError(#[from] SendError<task_handler::Event>),
}

// Tell axum how to convert `AppError` into a response.
#[cfg(feature = "ssr")]
impl IntoResponse for AxumError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {:?}", self),
        )
            .into_response()
    }
}

/* THIS IS ERROR STUFF FOR LEPTOS */

#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}

// A basic function to display errors served by the error boundaries.
// Feel free to do more complicated things here than just displaying the error.
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => RwSignal::new(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // Get Errors from Signal
    let errors = errors.get_untracked();

    // Downcast lets us take a type that implements `std::error::Error`
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();
    println!("Errors: {errors:#?}");

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }

    view! {
        <h1>{if errors.len() > 1 { "Errors" } else { "Error" }}</h1>
        <For
            // a function that returns the items we're iterating over; a signal is fine
            each=move || { errors.clone().into_iter().enumerate() }
            // a unique key for each item as a reference
            key=|(index, _error)| *index
            // renders each item to a view
            children=move |error| {
                let error_string = error.1.to_string();
                let error_code = error.1.status_code();
                view! {
                    <h2>{error_code.to_string()}</h2>
                    <p>"Error: " {error_string}</p>
                }
            }
        />
    }
}
