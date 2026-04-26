use anyhow::Result;
use futures::StreamExt;
use k8s_openapi::api::discovery::v1::EndpointSlice;
use kube::{
    Client,
    api::Api,
    runtime::{WatchStreamExt, watcher},
};
use std::pin::pin;

use super::cache::EndpointSliceCache;

/// Start the Kubernetes watcher for EndpointSlices with O(1) index cache.
pub async fn start_watcher() -> Result<EndpointSliceCache> {
    let client = Client::try_default().await?;
    let endpoint_slices: Api<EndpointSlice> = Api::all(client);
    let config = watcher::Config::default();

    let cache = EndpointSliceCache::new();
    let cache_clone = cache.clone();

    let stream = watcher(endpoint_slices, config).default_backoff();

    log::info!("Starting Kubernetes EndpointSlice watcher...");

    // Spawn the watcher loop in background
    tokio::spawn(async move {
        let mut stream = pin!(stream);
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => match event {
                    kube::runtime::watcher::Event::Apply(obj)
                    | kube::runtime::watcher::Event::InitApply(obj) => {
                        cache_clone.apply(&obj);
                        log::debug!(
                            "EndpointSlice {}/{} applied, {} endpoints",
                            obj.metadata.namespace.as_deref().unwrap_or("default"),
                            obj.metadata.name.as_deref().unwrap_or("unknown"),
                            obj.endpoints.len()
                        );
                    }
                    kube::runtime::watcher::Event::Delete(obj) => {
                        cache_clone.delete(&obj);
                        log::debug!(
                            "EndpointSlice {}/{} deleted",
                            obj.metadata.namespace.as_deref().unwrap_or("default"),
                            obj.metadata.name.as_deref().unwrap_or("unknown")
                        );
                    }
                    _ => {}
                },
                Err(e) => {
                    log::error!("Watcher error: {}", e);
                }
            }
        }
        log::warn!("Watcher stream ended");
    });

    Ok(cache)
}

/// Start the watcher in a background task, returning the cache handle
pub fn start_watcher_task() -> tokio::task::JoinHandle<Result<EndpointSliceCache>> {
    tokio::spawn(async move {
        loop {
            match start_watcher().await {
                Ok(cache) => {
                    log::info!("Watcher started successfully");
                    return Ok(cache);
                }
                Err(e) => {
                    log::error!("Watcher start error: {}, retrying in 5 seconds...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    })
}
