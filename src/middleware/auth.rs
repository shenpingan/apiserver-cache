use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;

/// Authentication middleware using x-api-token header
pub struct AuthMiddleware {
    api_token: String,
}

impl AuthMiddleware {
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service,
            api_token: self.api_token.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    api_token: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Skip authentication for /health/check
        let path = req.path();
        if path == "/health/check" {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }
        
        // Check x-api-token header
        let token = req.headers().get("x-api-token");
        
        match token {
            Some(token) => {
                if token.to_str().unwrap_or("") == self.api_token {
                    let fut = self.service.call(req);
                    Box::pin(async move {
                        let res = fut.await?;
                        Ok(res)
                    })
                } else {
                    Box::pin(async move {
                        Err(ErrorUnauthorized("Invalid API token"))
                    })
                }
            }
            None => {
                Box::pin(async move {
                    Err(ErrorUnauthorized("Missing x-api-token header"))
                })
            }
        }
    }
}
