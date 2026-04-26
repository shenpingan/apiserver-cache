use std::sync::Arc;
use dashmap::DashMap;
use k8s_openapi::api::discovery::v1::EndpointSlice;

/// Cache key: (namespace, service_name)
type CacheKey = (String, String);

/// High-performance cache with O(1) lookup by (namespace, service_name).
///
/// Uses DashMap for lock-free concurrent reads.
/// EndpointSlice names are `{service-name}-{hash}`, so we index by
/// `kubernetes.io/service-name` label.
#[derive(Clone)]
pub struct EndpointSliceCache {
    /// Index: (namespace, service_name) -> total endpoint count
    index: Arc<DashMap<CacheKey, usize>>,
}

impl EndpointSliceCache {
    pub fn new() -> Self {
        Self {
            index: Arc::new(DashMap::new()),
        }
    }

    /// Insert or update an EndpointSlice into the index.
    pub fn apply(&self, es: &EndpointSlice) {
        let namespace = es.metadata.namespace.as_deref().unwrap_or("").to_string();
        let service_name = es
            .metadata
            .labels
            .as_ref()
            .and_then(|l| l.get("kubernetes.io/service-name"))
            .map(|s| s.to_string())
            .unwrap_or_default();

        if service_name.is_empty() {
            return;
        }

        let key = (namespace, service_name);
        let count = es.endpoints.len();
        self.index.insert(key, count);
    }

    /// Remove an EndpointSlice from the index.
    pub fn delete(&self, es: &EndpointSlice) {
        let namespace = es.metadata.namespace.as_deref().unwrap_or("").to_string();
        let service_name = es
            .metadata
            .labels
            .as_ref()
            .and_then(|l| l.get("kubernetes.io/service-name"))
            .map(|s| s.to_string())
            .unwrap_or_default();

        if service_name.is_empty() {
            return;
        }

        let key = (namespace, service_name);
        self.index.remove(&key);
    }

    /// Get total endpoint count for a service. O(1) lookup.
    pub fn get_endpoint_count(&self, namespace: &str, service_name: &str) -> Option<usize> {
        self.index
            .get(&(namespace.to_string(), service_name.to_string()))
            .map(|r| *r.value())
    }
}
