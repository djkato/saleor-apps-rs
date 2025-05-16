use axum::extract::State;
use tokio::sync::mpsc;

use crate::{
    app::AppState,
    error_template::AxumError,
    server::event_handler::{CreateXMLEvent, Event},
};

pub async fn heureka_xml_feed_xml(State(state): State<AppState>) -> Result<String, AxumError> {
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
    Ok(xml)
}
