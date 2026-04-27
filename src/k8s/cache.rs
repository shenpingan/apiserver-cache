use std::sync::Arc;
use dashmap::DashMap;
use k8s_openapi::api::discovery::v1::EndpointSlice;

/// Cache key for a single EndpointSlice: (namespace, endpointslice_name)
type SliceKey = (String, String);
/// Cache key for a Service: (namespace, service_name)
type ServiceKey = (String, String);

/// High-performance cache with O(1) lookup by (namespace, service_name).
///
/// A Service may have multiple EndpointSlices (each max 100 endpoints by default),
/// so we track per-slice counts and aggregate on read.
#[derive(Clone)]
pub struct EndpointSliceCache {
    /// Per-EndpointSlice count: (namespace, slice_name) -> endpoints count
    slice_index: Arc<DashMap<SliceKey, ServiceKey>>,
    /// Per-Service total: (namespace, service_name) -> total endpoints count
    service_index: Arc<DashMap<ServiceKey, usize>>,
}

impl EndpointSliceCache {
    pub fn new() -> Self {
        Self {
            slice_index: Arc::new(DashMap::new()),
            service_index: Arc::new(DashMap::new()),
        }
    }

    fn extract_service_name(es: &EndpointSlice) -> Option<String> {
        es.metadata
            .labels
            .as_ref()
            .and_then(|l| l.get("kubernetes.io/service-name"))
            .map(|s| s.to_string())
    }

    /// Insert or update an EndpointSlice into the index.
    pub fn apply(&self, es: &EndpointSlice) {
        let namespace = es.metadata.namespace.as_deref().unwrap_or("").to_string();
        let slice_name = es.metadata.name.as_deref().unwrap_or("").to_string();
        let service_name = match Self::extract_service_name(es) {
            Some(s) if !s.is_empty() => s,
            _ => return,
        };

        let slice_key = (namespace.clone(), slice_name);
        let service_key = (namespace, service_name);
        let new_count = es.endpoints.len();

        // Update slice -> service mapping and adjust service total
        let old_count = self
            .slice_index
            .insert(slice_key, service_key.clone())
            .and_then(|old_service_key| {
                if old_service_key == service_key {
                    None
                } else {
                    // Slice changed service, decrement old service
                    self.service_index
                        .alter(&old_service_key, |_, v| v.saturating_sub(new_count));
                    self.service_index
                        .get(&old_service_key)
                        .map(|r| *r.value())
                }
            });

        // Adjust service total: add new, subtract old if same service
        self.service_index
            .alter(&service_key, |_, v| {
                if let Some(old) = old_count {
                    v + new_count - old
                } else {
                    v + new_count
                }
            });
    }

    /// Remove an EndpointSlice from the index.
    pub fn delete(&self, es: &EndpointSlice) {
        let namespace = es.metadata.namespace.as_deref().unwrap_or("").to_string();
        let slice_name = es.metadata.name.as_deref().unwrap_or("").to_string();

        let slice_key = (namespace, slice_name);
        if let Some((_, service_key)) = self.slice_index.remove(&slice_key) {
            let count = es.endpoints.len();
            self.service_index
                .alter(&service_key, |_, v| v.saturating_sub(count));
        }
    }

    /// Get total endpoint count for a service. O(1) lookup.
    pub fn get_endpoint_count(&self, namespace: &str, service_name: &str) -> Option<usize> {
        self.service_index
            .get(&(namespace.to_string(), service_name.to_string()))
            .map(|r| *r.value())
    }
}
