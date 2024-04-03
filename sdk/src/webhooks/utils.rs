use http::HeaderMap;

use crate::headers::SALEOR_EVENT_HEADER;

use super::{AsyncWebhookEventType, SyncWebhookEventType};

#[derive(Debug)]
pub enum EitherWebhookType {
    Sync(SyncWebhookEventType),
    Async(AsyncWebhookEventType),
}

//header "saleor-event" can have either sync or async type, so we return enum witch has either or
pub fn get_webhook_event_type(header: &HeaderMap) -> anyhow::Result<EitherWebhookType> {
    if let Some(event) = header.get(SALEOR_EVENT_HEADER) {
        let event = event.to_str()?;
        let s_event: Result<SyncWebhookEventType, _> = SyncWebhookEventType::try_from(event);
        let a_event: Result<AsyncWebhookEventType, _> = AsyncWebhookEventType::try_from(event);
        let event = match s_event {
            Ok(s) => EitherWebhookType::Sync(s),
            Err(_) => match a_event {
                Ok(a) => EitherWebhookType::Async(a),
                Err(e) => anyhow::bail!(e),
            },
        };
        return Ok(event);
    }
    anyhow::bail!("Missing event type header")
}
