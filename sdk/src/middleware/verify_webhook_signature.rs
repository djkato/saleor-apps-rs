use axum::{body, extract::Request, http::StatusCode, middleware::Next, response::Response};
use serde_json::Value;

use jsonwebtoken::{crypto, Algorithm, DecodingKey};
use tracing::{debug, error};

use crate::headers::{SALEOR_API_URL_HEADER, SALEOR_SIGNATURE_HEADER};

pub async fn webhook_signature_verifier(request: Request, next: Next) -> Response {
    let unauthorized = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .body(body::Body::from("Not authenticated\n"))
        .unwrap();

    let jwks_url = request
        .headers()
        .get(SALEOR_API_URL_HEADER).and_then(|h| {
            h.to_str()
                .map_or(None, |h| url::Url::parse(h).ok())
        });

    //get jwk from saleor api
    let jwks: Value = 'block: {
        if let Some(mut jwks_url) = jwks_url {
            jwks_url.set_path("/.well-known/jwks.json");
            if let Ok(get_res) = reqwest::get(jwks_url).await {
                if let Ok(val) = get_res.json::<Value>().await {
                    break 'block val;
                }
            }
        }
        error!("Saleor webhook signature not verified, failed fetching jwks from saleor");
        return unauthorized;
    };
    let nstr = jwks["keys"][0]["n"].as_str().unwrap();
    let estr = jwks["keys"][0]["e"].as_str().unwrap();

    let pubkey = DecodingKey::from_rsa_components(nstr, estr).unwrap();

    let (parts, body) = request.into_parts();
    let payload = body::to_bytes(body, usize::MAX).await.unwrap();

    if let Some(is_verified) = parts
        .headers
        .get(SALEOR_SIGNATURE_HEADER)
        .and_then(|sig| sig.to_str().ok())
        .and_then(|sig| {
            let parts: Vec<&str> = sig.split('.').collect();
            match parts.as_slice() {
                [protected, _, signature] => Some((*protected, *signature)),
                _ => None,
            }
        })
        .and_then(|(protected, signature)| {
            let mut msg: Vec<u8> = Vec::new();
            msg.extend_from_slice(format!("{}.", protected).as_bytes());
            msg.extend_from_slice(&payload);

            crypto::verify(signature, &msg, &pubkey, Algorithm::RS256).ok()
        })
    {
        match is_verified {
            true => {
                debug!("Saleor webhook signature verified");
                next.run(Request::from_parts(parts, payload.into())).await
            }
            false => {
                error!("Saleor webhook signature not correct");
                unauthorized
            }
        }
    } else {
        error!("Saleor webhook signature not verified, error parsing headers");
        unauthorized
    }
}

/* OLD
use http::{Request, Response};
use std::task::{Context, Poll};
use tower::Service;

use crate::headers::SALEOR_SIGNATURE_HEADER;
#[derive(Clone, Debug)]
pub struct WebhookVerifyJWTs<S> {
    inner: S,
}

impl<S> WebhookVerifyJWTs<S> {
    pub fn new(inner: S) -> Self {
        WebhookVerifyJWTs { inner }
    }
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for WebhookVerifyJWTs<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        if let Some(signature_header) = req.headers().get(SALEOR_SIGNATURE_HEADER) {
            let b = req.body_mut().data();
            if let Ok(saleor_signature) = signature_header.to_str() {
                let split: Vec<&str> = saleor_signature.split(".").collect();
                let header = split.get(0);
                let signature = split.get(2);
                if let Some(signature) = signature {
                    /*
                    let jws = jose_jws::Signature {
                        signature: signature.parse().unwrap(),
                        header:,
                        protected: None,
                    };
                        */
                }
            }
            /*
            if req.extensions().get::<RequestId>().is_none() {
                let request_id = request_id.clone();
                req.extensions_mut().insert(RequestId::new(request_id));
            }
            */
        }
        self.inner.call(req)
    }
}
*/
