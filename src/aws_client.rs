use aws_config::BehaviorVersion;
use aws_sdk_lightsail::Client;
use chrono::{Datelike, Utc};
use tracing::{error, info};

pub struct AwsLightsailClient {
    client: Client,
    instance_name: String,
}

#[derive(Debug)]
pub struct NetworkUsage {
    pub network_in_bytes: f64,
    pub network_out_bytes: f64,
    pub monthly_transfer_limit_bytes: f64,
}

impl AwsLightsailClient {
    pub async fn new(
        access_key_id: &str,
        secret_access_key: &str,
        region: &str,
        instance_name: &str,
    ) -> Self {
        // Set environment variables for aws-config to pick up
        std::env::set_var("AWS_ACCESS_KEY_ID", access_key_id);
        std::env::set_var("AWS_SECRET_ACCESS_KEY", secret_access_key);
        std::env::set_var("AWS_REGION", region);

        let config = aws_config::defaults(BehaviorVersion::latest()).load().await;

        let client = Client::new(&config);

        AwsLightsailClient {
            client,
            instance_name: instance_name.to_string(),
        }
    }

    pub async fn get_network_usage(&self) -> Result<NetworkUsage, String> {
        let now = Utc::now();
        let start_of_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let start_time = start_of_month.timestamp();
        let end_time = now.timestamp();

        info!(
            instance = self.instance_name,
            start_time = start_time,
            end_time = end_time,
            "Fetching AWS Lightsail network metrics"
        );

        let network_in = self
            .get_metric_data("NetworkIn", start_time, end_time)
            .await?;
        let network_out = self
            .get_metric_data("NetworkOut", start_time, end_time)
            .await?;

        let instance = self
            .client
            .get_instance()
            .instance_name(&self.instance_name)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get instance info: {}", e);
                format!("Failed to get instance info: {}", e)
            })?;

        let monthly_transfer_limit = instance
            .instance()
            .and_then(|i| i.networking())
            .and_then(|n| n.monthly_transfer())
            .and_then(|t| t.gb_per_month_allocated())
            .unwrap_or(0) as f64
            * 1073741824.0; // Convert GB to bytes

        info!(
            instance = self.instance_name,
            network_in = network_in,
            network_out = network_out,
            monthly_transfer_limit = monthly_transfer_limit,
            "Fetched AWS Lightsail network metrics"
        );

        Ok(NetworkUsage {
            network_in_bytes: network_in,
            network_out_bytes: network_out,
            monthly_transfer_limit_bytes: monthly_transfer_limit,
        })
    }

    async fn get_metric_data(
        &self,
        metric_name: &str,
        start_time: i64,
        end_time: i64,
    ) -> Result<f64, String> {
        let result = self
            .client
            .get_instance_metric_data()
            .instance_name(&self.instance_name)
            .metric_name(aws_sdk_lightsail::types::InstanceMetricName::from(
                metric_name,
            ))
            .period(86400) // 1 day in seconds
            .start_time(aws_sdk_lightsail::primitives::DateTime::from_secs(
                start_time,
            ))
            .end_time(aws_sdk_lightsail::primitives::DateTime::from_secs(end_time))
            .statistics(aws_sdk_lightsail::types::MetricStatistic::Sum)
            .unit(aws_sdk_lightsail::types::MetricUnit::Bytes)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to get metric data for {}: {}", metric_name, e);
                format!("Failed to get metric data for {}: {}", metric_name, e)
            })?;

        let total: f64 = result.metric_data().iter().filter_map(|dp| dp.sum()).sum();

        Ok(total)
    }
}
