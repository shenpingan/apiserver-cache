use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;

use crate::k8s::cache::EndpointSliceCache;

#[derive(Serialize)]
struct EndpointResponse {
    endpoint_slice_len: usize,
}

/// GET /api/endpoint/:namespace/:name
/// Returns the endpoint slice information
pub async fn get_endpoint(
    path: web::Path<(String, String)>,
    cache: web::Data<EndpointSliceCache>,
) -> impl Responder {
    let (namespace, name) = path.into_inner();

    log::debug!("GET /api/endpoint/{}/{}", namespace, name);

    match cache.get_endpoint_count(&namespace, &name) {
        Some(count) => {
            HttpResponse::Ok().json(EndpointResponse { endpoint_slice_len: count })
        }
        None => {
            HttpResponse::NotFound().json(EndpointResponse { endpoint_slice_len: 0 })
        }
    }
}
