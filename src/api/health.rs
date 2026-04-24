use actix_web::{HttpResponse, Responder};
use serde_json::json;

/// GET /health/check
/// Health check endpoint (no authentication required)
pub async fn health_check() -> impl Responder {
    let response = json!({
        "status": "ok",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    HttpResponse::Ok().json(response)
}
