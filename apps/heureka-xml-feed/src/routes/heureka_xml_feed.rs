use axum::{extract::State, response::IntoResponse};
use http::HeaderMap;
use tokio::sync::mpsc;

use crate::{
    app::AppState,
    error_template::AxumError,
    server::event_handler::{CreateXMLEvent, Event},
};

pub async fn heureka_feed_xml(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AxumError> {
    let (tx, mut rx) = mpsc::channel(10);
    state
        .task_queue_sender
        .send(Event::CreateXML(CreateXMLEvent {
            state: state.clone(),
            sender: tx,
        }))
        .await?;

    let xml = rx
        .recv()
        .await
        .flatten()
        .ok_or(AxumError::InternalServerError(
            "Failed getting xml".to_owned(),
        ))?;

    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/xml".parse().unwrap());
    Ok((headers, xml))
}
