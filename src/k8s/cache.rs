use std::sync::Arc;
use k8s_openapi::api::discovery::v1::EndpointSlice;
use kube::runtime::reflector::Store;

/// Cache for storing EndpointSlice data using kube-rs reflector Store.
///
/// EndpointSlice names are `{service-name}-{hash}`, not the service name itself.
/// We look up by the `kubernetes.io/service-name` label instead.
#[derive(Clone)]
pub struct EndpointSliceCache {
    store: Store<EndpointSlice>,
}

impl EndpointSliceCache {
    pub fn new(store: Store<EndpointSlice>) -> Self {
        Self { store }
    }

    /// Get total endpoint count across all EndpointSlices for a given service.
    ///
    /// EndpointSlice names are `{service-name}-{hash}`, so we match by the
    /// `kubernetes.io/service-name` label instead of the resource name.
    pub fn get_endpoint_count(&self, namespace: &str, service_name: &str) -> Option<usize> {
        let total: usize = self
            .store
            .state()
            .into_iter()
            .filter(|es| {
                let ns = es.metadata.namespace.as_deref().unwrap_or("");
                let svc = es
                    .metadata
                    .labels
                    .as_ref()
                    .and_then(|l| l.get("kubernetes.io/service-name"))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                ns == namespace && svc == service_name
            })
            .map(|es| es.endpoints.len())
            .sum();

        if total > 0 {
            Some(total)
        } else {
            None
        }
    }

    /// Get all EndpointSlices for a service (by service name label).
    #[allow(dead_code)]
    pub fn get_by_service(
        &self,
        namespace: &str,
        service_name: &str,
    ) -> Vec<Arc<EndpointSlice>> {
        self.store
            .state()
            .into_iter()
            .filter(|es| {
                let ns = es.metadata.namespace.as_deref().unwrap_or("");
                let svc = es
                    .metadata
                    .labels
                    .as_ref()
                    .and_then(|l| l.get("kubernetes.io/service-name"))
                    .map(|s| s.as_str())
                    .unwrap_or("");
                ns == namespace && svc == service_name
            })
            .collect()
    }

    /// Get all cached EndpointSlices
    #[allow(dead_code)]
    pub fn state(&self) -> Vec<Arc<EndpointSlice>> {
        self.store.state()
    }
}
