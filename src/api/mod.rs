pub mod endpoint;
pub mod health;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            .route("/endpoint/{namespace}/{name}", web::get().to(endpoint::get_endpoint))
    )
    .route("/health/check", web::get().to(health::health_check));
}
