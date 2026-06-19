mod api_client;
mod aws_client;
mod config;
mod metrics;
mod server;

use api_client::CloudClient;
use aws_client::AwsLightsailClient;
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

    if config.has_64clouds() {
        info!(
            "64clouds fetch interval: {} seconds",
            config.cloud_64clouds_fetch_interval
        );
    } else {
        info!("64clouds is not configured, skipping");
    }

    if config.has_aws() {
        info!(
            "AWS Lightsail fetch interval: {} seconds",
            config.aws_lightsail_fetch_interval
        );
    } else {
        info!("AWS Lightsail is not configured, skipping");
    }

    let metrics = Arc::new(Metrics::new());
    let mut registry = Registry::default();
    metrics.register(&mut registry);
    let registry = Arc::new(registry);

    // 64clouds data fetcher
    if config.has_64clouds() {
        let veid = config.cloud_64clouds_veid.clone().unwrap();
        let api_key = config.cloud_64clouds_api_key.clone().unwrap();
        let cloud_client = Arc::new(CloudClient::new(veid.clone(), api_key));
        let metrics_clone = metrics.clone();
        let cloud_client_clone = cloud_client.clone();
        let cloud_fetch_interval = config.cloud_64clouds_fetch_interval;

        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(cloud_fetch_interval));

            loop {
                interval_timer.tick().await;

                match cloud_client_clone.get_service_info().await {
                    Ok(service_info) => {
                        metrics_clone.update_64clouds(&veid, &service_info);
                    }
                    Err(e) => {
                        error!("Failed to fetch 64clouds service info: {}", e);
                    }
                }
            }
        });
    }

    // AWS Lightsail data fetcher
    if config.has_aws() {
        let access_key_id = config.aws_access_key_id.clone().unwrap();
        let secret_access_key = config.aws_secret_access_key.clone().unwrap();
        let region = config.aws_region.clone().unwrap();
        let instance_name = config.aws_lightsail_instance_name.clone().unwrap();

        let aws_client = Arc::new(
            AwsLightsailClient::new(&access_key_id, &secret_access_key, &region, &instance_name)
                .await,
        );

        let metrics_clone = metrics.clone();
        let aws_client_clone = aws_client.clone();
        let instance_name_clone = instance_name.clone();
        let region_clone = region.clone();
        let aws_fetch_interval = config.aws_lightsail_fetch_interval;

        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(aws_fetch_interval));

            loop {
                interval_timer.tick().await;

                match aws_client_clone.get_network_usage().await {
                    Ok(network_usage) => {
                        metrics_clone.update_aws(
                            &instance_name_clone,
                            &region_clone,
                            &network_usage,
                        );
                    }
                    Err(e) => {
                        error!("Failed to fetch AWS Lightsail network usage: {}", e);
                    }
                }
            }
        });
    }

    let metrics_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), config.metrics_port);
    server::start_metrics_server(metrics_addr, registry).await;
}
