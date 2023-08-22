use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use actix_http::body::EitherBody;

use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{Error, HttpResponse};
use futures::future::{ok, Ready};
use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::db::models::api_config::ApiConfig;
use actix_web::web::Data;

pub(crate) type LockedApiConfig = Arc<RwLock<Option<ApiConfig>>>;

lazy_static! {
    pub static ref API_CONFIG_ARC: LockedApiConfig = Arc::new(RwLock::new(None));
}

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct ApiConfigStatus;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for ApiConfigStatus
where
    S: Service<ServiceRequest, Response = ServiceResponse<EitherBody<B>>, Error = actix_web::Error>
        + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = ApiConfigStatusMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiConfigStatusMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct ApiConfigStatusMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ApiConfigStatusMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<EitherBody<B>>, Error = actix_web::Error>
        + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let online = req.app_data::<Data<LockedApiConfig>>().map_or_else(
            || false,
            |c| {
                let config = c.read();
                !config.as_ref().unwrap().config_data.api.force_offline
            },
        );
        Box::pin(async move {
            if online {
                Ok(service.call(req).await?)
            } else {
                Ok(req.into_response(
                    HttpResponse::Gone()
                        .finish()
                        .map_into_boxed_body()
                        .map_into_right_body(),
                ))
            }
        })
    }
}
