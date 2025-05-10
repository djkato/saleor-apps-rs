use axum::{Json, extract::State};
use heureka_xml_feed::Shop;

use crate::{app::AppState, error_template::AxumError};

pub async fn heureka_xml_feed_xml(State(state): State<AppState>) -> Result<Json<Shop>, AxumError> {
    todo!()
}
