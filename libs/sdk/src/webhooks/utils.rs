use crate::headers::SALEOR_EVENT_HEADER;
use http::{header::ToStrError, HeaderMap};

use super::{AsyncWebhookEventType, SyncWebhookEventType};

#[derive(Debug, Clone)]
pub enum EitherWebhookType {
    Sync(SyncWebhookEventType),
    Async(AsyncWebhookEventType),
}

#[derive(thiserror::Error, Debug)]
pub enum GetWebhookTypeError {
    #[error("Failed parsing webhook type, {0}")]
    ParseError(#[from] strum::ParseError),
    #[error("Failed parsing header to str, {0}")]
    ToStrError(#[from] ToStrError),
    #[error("Missing Event type header")]
    MissingWebhookTypeHeader,
}
//header "saleor-event" can have either sync or async type, so we return enum witch has either or
pub fn get_webhook_event_type(
    header: &HeaderMap,
) -> Result<EitherWebhookType, GetWebhookTypeError> {
    if let Some(event) = header.get(SALEOR_EVENT_HEADER) {
        let event = event.to_str()?;
        let s_event: Result<SyncWebhookEventType, _> = SyncWebhookEventType::try_from(event);
        let a_event: Result<AsyncWebhookEventType, _> = AsyncWebhookEventType::try_from(event);
        let event = match s_event {
            Ok(s) => EitherWebhookType::Sync(s),
            Err(_) => EitherWebhookType::Async(a_event?),
        };
        return Ok(event);
    }
    Err(GetWebhookTypeError::MissingWebhookTypeHeader)
}
