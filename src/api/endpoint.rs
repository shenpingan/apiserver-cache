use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;

use crate::k8s::cache::EndpointSliceCache;

#[derive(Serialize)]
struct EndpointResponse {
    endpoint_slice_len: usize,
    endpoint_count: usize,
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
        Some(count) => HttpResponse::Ok().json(EndpointResponse {
            endpoint_slice_len: count,
            endpoint_count: count,
        }),
        None => HttpResponse::NotFound().json(EndpointResponse {
            endpoint_slice_len: 0,
            endpoint_count: 0,
        }),
    }
}
