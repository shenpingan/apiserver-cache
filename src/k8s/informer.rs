use anyhow::Result;
use futures::StreamExt;
use k8s_openapi::api::discovery::v1::EndpointSlice;
use kube::{
    Client,
    api::Api,
    runtime::{WatchStreamExt, reflector, watcher},
};
use std::pin::pin;

use super::cache::EndpointSliceCache;

/// Start the Kubernetes reflector for EndpointSlices
pub async fn start_reflector() -> Result<EndpointSliceCache> {
    let client = Client::try_default().await?;

    let endpoint_slices: Api<EndpointSlice> = Api::all(client);

    let config = watcher::Config::default();

    let (store, writer) = reflector::store::<EndpointSlice>();

    let stream = watcher(endpoint_slices, config).default_backoff();
    let stream = reflector::reflector(writer, stream);

    log::info!("Starting Kubernetes EndpointSlice reflector...");

    // Spawn the watcher loop in background
    tokio::spawn(async move {
        let mut stream = pin!(stream);
        while let Some(event) = stream.next().await {
            match event {
                Ok(event) => match event {
                    kube::runtime::watcher::Event::Apply(obj)
                    | kube::runtime::watcher::Event::InitApply(obj) => {
                        let namespace = obj.metadata.namespace.as_deref().unwrap_or("default");
                        let name = obj.metadata.name.as_deref().unwrap_or("unknown");
                        log::debug!("EndpointSlice {}/{} applied", namespace, name);
                    }
                    kube::runtime::watcher::Event::Delete(obj) => {
                        let namespace = obj.metadata.namespace.as_deref().unwrap_or("default");
                        let name = obj.metadata.name.as_deref().unwrap_or("unknown");
                        log::debug!("EndpointSlice {}/{} deleted", namespace, name);
                    }
                    _ => {}
                },
                Err(e) => {
                    log::error!("Reflector error: {}", e);
                }
            }
        }
        log::warn!("Reflector stream ended");
    });

    Ok(EndpointSliceCache::new(store))
}

/// Start the reflector in a background task, returning the cache handle
pub fn start_reflector_task() -> tokio::task::JoinHandle<EndpointSliceCache> {
    tokio::spawn(async move {
        loop {
            match start_reflector().await {
                Ok(cache) => {
                    log::info!("Reflector started successfully");
                    // Return the cache - the reflector loop continues in background
                    return cache;
                }
                Err(e) => {
                    log::error!("Reflector start error: {}, retrying in 5 seconds...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    })
}
