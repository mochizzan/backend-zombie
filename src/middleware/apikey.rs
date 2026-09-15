use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use serde::Serialize;

use crate::config::Config;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    success: bool,
    error: ErrorDetail,
}

#[derive(Debug, Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

pub struct ApiKey;

impl<S, B> Transform<S, ServiceRequest> for ApiKey
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiKeyMiddleware { service })
    }
}

pub struct ApiKeyMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Extract the API key from Config
        let config = req.app_data::<web::Data<Config>>().cloned();
        
        // Get the API key from the request header
        let api_key = req
            .headers()
            .get("X-API-Key")
            .and_then(|value| value.to_str().ok())
            .map(String::from);

        let fut = self.service.call(req);

        Box::pin(async move {
            match (config, api_key) {
                (Some(config), Some(key)) => {
                    if key == config.api_key {
                        fut.await
                    } else {
                        let response = ErrorResponse {
                            success: false,
                            error: ErrorDetail {
                                code: "UNAUTHORIZED".to_string(),
                                message: "Invalid API key".to_string(),
                            },
                        };
                        Err(actix_web::error::ErrorUnauthorized(
                            serde_json::to_string(&response).unwrap(),
                        ))
                    }
                }
                (Some(_), None) => {
                    let response = ErrorResponse {
                        success: false,
                        error: ErrorDetail {
                            code: "UNAUTHORIZED".to_string(),
                            message: "Missing API key".to_string(),
                        },
                    };
                    Err(actix_web::error::ErrorUnauthorized(
                        serde_json::to_string(&response).unwrap(),
                    ))
                }
                _ => {
                    let response = ErrorResponse {
                        success: false,
                        error: ErrorDetail {
                            code: "INTERNAL_ERROR".to_string(),
                            message: "Server configuration error".to_string(),
                        },
                    };
                    Err(actix_web::error::ErrorInternalServerError(
                        serde_json::to_string(&response).unwrap(),
                    ))
                }
            }
        })
    }
}
