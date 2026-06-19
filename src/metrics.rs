use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use tracing::info;

use crate::api_client::ServiceInfo;
use crate::aws_client::NetworkUsage;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VeidLabel {
    pub veid: String,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct AwsInstanceLabel {
    pub instance_name: String,
    pub region: String,
}

#[derive(Debug)]
pub struct Metrics {
    // 64clouds metrics
    pub plan_monthly_data_bytes: Family<VeidLabel, Gauge>,
    pub data_counter_bytes: Family<VeidLabel, Gauge>,
    pub data_next_reset_timestamp: Family<VeidLabel, Gauge>,

    // AWS Lightsail metrics
    pub aws_monthly_network_in_bytes: Family<AwsInstanceLabel, Gauge>,
    pub aws_monthly_network_out_bytes: Family<AwsInstanceLabel, Gauge>,
    pub aws_monthly_transfer_limit_bytes: Family<AwsInstanceLabel, Gauge>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            plan_monthly_data_bytes: Family::default(),
            data_counter_bytes: Family::default(),
            data_next_reset_timestamp: Family::default(),
            aws_monthly_network_in_bytes: Family::default(),
            aws_monthly_network_out_bytes: Family::default(),
            aws_monthly_transfer_limit_bytes: Family::default(),
        }
    }

    pub fn register(&self, registry: &mut Registry) {
        // 64clouds metrics
        registry.register(
            "cloud_64clouds_plan_monthly_data_bytes",
            "Monthly data limit in bytes",
            self.plan_monthly_data_bytes.clone(),
        );

        registry.register(
            "cloud_64clouds_data_counter_bytes",
            "Monthly data usage in bytes",
            self.data_counter_bytes.clone(),
        );

        registry.register(
            "cloud_64clouds_data_next_reset_timestamp",
            "Data reset timestamp (Unix timestamp)",
            self.data_next_reset_timestamp.clone(),
        );

        // AWS Lightsail metrics
        registry.register(
            "aws_lightsail_monthly_network_in_bytes",
            "Monthly network input bytes",
            self.aws_monthly_network_in_bytes.clone(),
        );

        registry.register(
            "aws_lightsail_monthly_network_out_bytes",
            "Monthly network output bytes",
            self.aws_monthly_network_out_bytes.clone(),
        );

        registry.register(
            "aws_lightsail_monthly_transfer_limit_bytes",
            "Monthly transfer limit in bytes",
            self.aws_monthly_transfer_limit_bytes.clone(),
        );
    }

    pub fn update_64clouds(&self, veid: &str, service_info: &ServiceInfo) {
        let label = VeidLabel {
            veid: veid.to_string(),
        };

        self.plan_monthly_data_bytes
            .get_or_create(&label)
            .set(service_info.plan_monthly_data as i64);
        self.data_counter_bytes
            .get_or_create(&label)
            .set(service_info.data_counter as i64);
        self.data_next_reset_timestamp
            .get_or_create(&label)
            .set(service_info.data_next_reset as i64);

        info!(
            veid = veid,
            plan_monthly_data = service_info.plan_monthly_data,
            data_counter = service_info.data_counter,
            data_next_reset = service_info.data_next_reset,
            "Updated 64clouds metrics"
        );
    }

    pub fn update_aws(&self, instance_name: &str, region: &str, network_usage: &NetworkUsage) {
        let label = AwsInstanceLabel {
            instance_name: instance_name.to_string(),
            region: region.to_string(),
        };

        self.aws_monthly_network_in_bytes
            .get_or_create(&label)
            .set(network_usage.network_in_bytes as i64);
        self.aws_monthly_network_out_bytes
            .get_or_create(&label)
            .set(network_usage.network_out_bytes as i64);
        self.aws_monthly_transfer_limit_bytes
            .get_or_create(&label)
            .set(network_usage.monthly_transfer_limit_bytes as i64);

        info!(
            instance_name = instance_name,
            region = region,
            network_in = network_usage.network_in_bytes,
            network_out = network_usage.network_out_bytes,
            monthly_transfer_limit = network_usage.monthly_transfer_limit_bytes,
            "Updated AWS Lightsail metrics"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prometheus_client::encoding::text::encode;

    #[test]
    fn test_metrics_new() {
        let metrics = Metrics::new();
        let mut registry = Registry::default();
        metrics.register(&mut registry);

        let service_info = ServiceInfo {
            plan_monthly_data: 0,
            data_counter: 0,
            data_next_reset: 0,
            error: 0,
        };

        metrics.update_64clouds("test", &service_info);

        let network_usage = NetworkUsage {
            network_in_bytes: 0.0,
            network_out_bytes: 0.0,
            monthly_transfer_limit_bytes: 0.0,
        };

        metrics.update_aws("test-instance", "us-east-1", &network_usage);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer.contains("cloud_64clouds_plan_monthly_data_bytes"));
        assert!(buffer.contains("cloud_64clouds_data_counter_bytes"));
        assert!(buffer.contains("cloud_64clouds_data_next_reset_timestamp"));
        assert!(buffer.contains("aws_lightsail_monthly_network_in_bytes"));
        assert!(buffer.contains("aws_lightsail_monthly_network_out_bytes"));
        assert!(buffer.contains("aws_lightsail_monthly_transfer_limit_bytes"));
    }

    #[test]
    fn test_metrics_update_64clouds() {
        let metrics = Metrics::new();
        let mut registry = Registry::default();
        metrics.register(&mut registry);

        let service_info = ServiceInfo {
            plan_monthly_data: 322122547200,
            data_counter: 4804320660,
            data_next_reset: 1783744807,
            error: 0,
        };

        metrics.update_64clouds("1379859", &service_info);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer
            .contains(r#"cloud_64clouds_plan_monthly_data_bytes{veid="1379859"} 322122547200"#));
        assert!(buffer.contains(r#"cloud_64clouds_data_counter_bytes{veid="1379859"} 4804320660"#));
        assert!(buffer
            .contains(r#"cloud_64clouds_data_next_reset_timestamp{veid="1379859"} 1783744807"#));
    }

    #[test]
    fn test_metrics_update_aws() {
        let metrics = Metrics::new();
        let mut registry = Registry::default();
        metrics.register(&mut registry);

        let network_usage = NetworkUsage {
            network_in_bytes: 1000000.0,
            network_out_bytes: 2000000.0,
            monthly_transfer_limit_bytes: 322122547200.0,
        };

        metrics.update_aws("jp-1", "ap-northeast-1", &network_usage);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer.contains(r#"aws_lightsail_monthly_network_in_bytes{instance_name="jp-1",region="ap-northeast-1"} 1000000"#));
        assert!(buffer.contains(r#"aws_lightsail_monthly_network_out_bytes{instance_name="jp-1",region="ap-northeast-1"} 2000000"#));
        assert!(buffer.contains(r#"aws_lightsail_monthly_transfer_limit_bytes{instance_name="jp-1",region="ap-northeast-1"} 322122547200"#));
    }

    #[test]
    fn test_metrics_update_overwrite() {
        let metrics = Metrics::new();
        let mut registry = Registry::default();
        metrics.register(&mut registry);

        let service_info1 = ServiceInfo {
            plan_monthly_data: 100,
            data_counter: 50,
            data_next_reset: 1000,
            error: 0,
        };

        let service_info2 = ServiceInfo {
            plan_monthly_data: 200,
            data_counter: 75,
            data_next_reset: 2000,
            error: 0,
        };

        metrics.update_64clouds("1379859", &service_info1);
        metrics.update_64clouds("1379859", &service_info2);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer.contains(r#"cloud_64clouds_plan_monthly_data_bytes{veid="1379859"} 200"#));
        assert!(!buffer.contains(r#"cloud_64clouds_plan_monthly_data_bytes{veid="1379859"} 100"#));
    }

    #[test]
    fn test_veid_label_equality() {
        let label1 = VeidLabel {
            veid: "123".to_string(),
        };
        let label2 = VeidLabel {
            veid: "123".to_string(),
        };
        let label3 = VeidLabel {
            veid: "456".to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }

    #[test]
    fn test_aws_instance_label_equality() {
        let label1 = AwsInstanceLabel {
            instance_name: "jp-1".to_string(),
            region: "ap-northeast-1".to_string(),
        };
        let label2 = AwsInstanceLabel {
            instance_name: "jp-1".to_string(),
            region: "ap-northeast-1".to_string(),
        };
        let label3 = AwsInstanceLabel {
            instance_name: "jp-2".to_string(),
            region: "us-east-1".to_string(),
        };

        assert_eq!(label1, label2);
        assert_ne!(label1, label3);
    }
}
