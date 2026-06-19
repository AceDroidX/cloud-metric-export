mod api_client;
mod config;
mod metrics;
mod server;

use api_client::CloudClient;
use config::Config;
use metrics::Metrics;
use prometheus_client::registry::Registry;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    if let Err(e) = dotenvy::dotenv() {
        info!("No .env file found or failed to load: {}", e);
    } else {
        info!("Loaded .env file");
    }

    let config = Config::from_env().unwrap_or_else(|e| {
        error!("Failed to load configuration: {}", e);
        std::process::exit(1);
    });

    info!("Starting cloud-metric-export");
    info!("Metrics port: {}", config.metrics_port);
    info!("Fetch interval: {} seconds", config.fetch_interval_seconds);

    let metrics = Arc::new(Metrics::new());
    let mut registry = Registry::default();
    metrics.register(&mut registry);
    let registry = Arc::new(registry);

    let cloud_client = Arc::new(CloudClient::new(
        config.cloud_64clouds_veid.clone(),
        config.cloud_64clouds_api_key.clone(),
    ));

    let metrics_clone = metrics.clone();
    let cloud_client_clone = cloud_client.clone();
    let veid = config.cloud_64clouds_veid.clone();
    let fetch_interval = config.fetch_interval_seconds;

    tokio::spawn(async move {
        let mut interval_timer = interval(Duration::from_secs(fetch_interval));

        loop {
            interval_timer.tick().await;

            match cloud_client_clone.get_service_info().await {
                Ok(service_info) => {
                    metrics_clone.update(&veid, &service_info);
                }
                Err(e) => {
                    error!("Failed to fetch service info: {}", e);
                }
            }
        }
    });

    let metrics_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.metrics_port);
    server::start_metrics_server(metrics_addr, registry).await;
}
