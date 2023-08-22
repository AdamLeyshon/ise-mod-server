use std::io;
use std::io::Read;
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;

use actix_http::ContentEncoding;

use crate::request_helpers::ProtoBufConfig;
use actix_service::{Service, Transform};
use actix_web::web::BytesMut;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpMessage};
use flate2::read::GzDecoder;
use futures::future::{ok, Future, Ready};
use futures::stream::StreamExt;
use http::header::CONTENT_ENCODING;

pub struct DecompressPayload;

impl<S: 'static, B> Transform<S, ServiceRequest> for DecompressPayload
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = DecompressPayloadMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(DecompressPayloadMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct DecompressPayloadMiddleware<S> {
    // This is special: We need this to avoid lifetime issues.
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for DecompressPayloadMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let encoding = if let Some(enc) = req.headers().get(&CONTENT_ENCODING) {
                if let Ok(enc) = enc.to_str() {
                    ContentEncoding::from_str(enc).unwrap()
                } else {
                    ContentEncoding::Identity
                }
            } else {
                ContentEncoding::Identity
            };

            // We're only interested in Gzip data, skip everything else
            if encoding == ContentEncoding::Gzip {
                let mut body = BytesMut::new();
                let mut stream = req.take_payload();
                while let Some(chunk) = stream.next().await {
                    body.extend_from_slice(&chunk?);
                }
                debug!("Incoming payload headers");
                for header in req.headers().into_iter() {
                    debug!("{:?}", header)
                }
                debug!("Compressed payload size {:?}", body.len());
                let body = gzip_decompress(body.to_vec()).unwrap();
                debug!("Uncompressed payload size {:?}", body.len());
                let limit = req
                    .app_data::<ProtoBufConfig>()
                    .map(|c| c.limit)
                    .unwrap_or(262_144);
                if body.len() > limit {
                    warn!("ProtoBuf message size {} > {}", body.len(), limit)
                }
                let mut payload = actix_http::h1::Payload::create(false);
                payload.0.feed_data(body.into());
                payload.0.feed_eof();
                req.set_payload(payload.1.into());
            }

            let res = svc.call(req).await?;

            Ok(res)
        })
    }
}

fn gzip_decompress(bytes: Vec<u8>) -> io::Result<Vec<u8>> {
    let mut result = Vec::<u8>::new();
    let mut gz = GzDecoder::new(&bytes[..]);
    gz.read_to_end(&mut result)?;
    Ok(result)
}
