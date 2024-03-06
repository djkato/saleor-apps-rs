/*use http::{Request, Response};
use std::{
    str::Bytes,
    task::{Context, Poll},
};
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
        /*
        if let Some(signature_header) = req.headers().get(SALEOR_SIGNATURE_HEADER) {
            if let Ok(saleor_signature) = signature_header.to_str() {
                let split: Vec<&str> = saleor_signature.split(".").collect();
                let header = split.get(0);
                let signature = split.get(2);
                if let Some(signature) = signature {
                    let jws = jose_jws::Signature {
                        signature: signature.parse().unwrap(),
                        header:,
                        protected: None,
                    };
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
            */
        todo!()
    }
}*/
