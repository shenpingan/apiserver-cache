use actix_web::{App, HttpServer, web};
use clap::Parser;
use log::info;
mod api;
mod config;
mod k8s;
mod middleware;

use config::Config;
use middleware::AuthMiddleware;

/// Kubernetes EndpointSlice Cache Server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to configuration file
    #[clap(short, long, default_value = "config.yaml")]
    config: String,

    /// Log level (trace, debug, info, warn, error)
    #[clap(short, long)]
    log_level: Option<String>,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = Config::from_file(&cli.config).unwrap_or_else(|e| {
        eprintln!("Failed to load config from {}: {}", cli.config, e);
        std::process::exit(1);
    });

    // Initialize logger
    let log_level = cli.log_level.unwrap_or(config.logging.level.clone());
    unsafe {
        std::env::set_var("RUST_LOG", &log_level);
    }
    env_logger::init();

    info!("Starting EndpointSlice Cache Server");
    info!("Configuration loaded from: {}", cli.config);
    info!("Server will listen on: {}", config.server_addr());

    // Start Kubernetes watcher and get cache handle
    info!("Starting Kubernetes watcher...");
    let cache: k8s::cache::EndpointSliceCache = k8s::start_watcher_task().await??;

    info!("Reflector started, cache ready");

    // Start HTTP server
    let server_addr = config.server_addr();
    let api_token = config.auth.api_token.clone();

    info!("Starting HTTP server on {}", server_addr);

    let cache_data = web::Data::new(cache);

    HttpServer::new(move || {
        App::new()
            .app_data(cache_data.clone())
            .wrap(AuthMiddleware::new(api_token.clone()))
            .wrap(actix_web::middleware::Logger::default())
            .configure(api::configure)
    })
    .bind(&server_addr)?
    .run()
    .await?;

    Ok(())
}
