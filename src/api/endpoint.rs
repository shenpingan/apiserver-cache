use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use crate::k8s::cache::EndpointSliceCache;

/// GET /api/endpoint/:namespace/:name
/// Returns the endpoint slice information
pub async fn get_endpoint(
    path: web::Path<(String, String)>,
    cache: web::Data<EndpointSliceCache>,
) -> impl Responder {
    let (namespace, name) = path.into_inner();

    log::info!("GET /api/endpoint/{}/{}", namespace, name);

    match cache.get_endpoint_count(&namespace, &name) {
        Some(count) => {
            let response = json!({
                "endpoint_slice_len": count
            });
            HttpResponse::Ok().json(response)
        }
        None => {
            let response = json!({
                "error": "EndpointSlice not found",
                "namespace": namespace,
                "name": name
            });
            HttpResponse::NotFound().json(response)
        }
    }
}
