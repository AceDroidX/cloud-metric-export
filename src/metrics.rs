use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::registry::Registry;
use tracing::info;

use crate::api_client::ServiceInfo;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct VeidLabel {
    pub veid: String,
}

#[derive(Debug)]
pub struct Metrics {
    pub plan_monthly_data_bytes: Family<VeidLabel, Gauge>,
    pub data_counter_bytes: Family<VeidLabel, Gauge>,
    pub data_next_reset_timestamp: Family<VeidLabel, Gauge>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            plan_monthly_data_bytes: Family::default(),
            data_counter_bytes: Family::default(),
            data_next_reset_timestamp: Family::default(),
        }
    }

    pub fn register(&self, registry: &mut Registry) {
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
    }

    pub fn update(&self, veid: &str, service_info: &ServiceInfo) {
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
            "Updated metrics"
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

        metrics.update("test", &service_info);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer.contains("cloud_64clouds_plan_monthly_data_bytes"));
        assert!(buffer.contains("cloud_64clouds_data_counter_bytes"));
        assert!(buffer.contains("cloud_64clouds_data_next_reset_timestamp"));
    }

    #[test]
    fn test_metrics_register() {
        let metrics = Metrics::new();
        let mut registry = Registry::default();
        metrics.register(&mut registry);

        let service_info = ServiceInfo {
            plan_monthly_data: 100,
            data_counter: 50,
            data_next_reset: 1000,
            error: 0,
        };

        metrics.update("test", &service_info);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer.contains("cloud_64clouds_plan_monthly_data_bytes"));
        assert!(buffer.contains("cloud_64clouds_data_counter_bytes"));
        assert!(buffer.contains("cloud_64clouds_data_next_reset_timestamp"));
    }

    #[test]
    fn test_metrics_update() {
        let metrics = Metrics::new();
        let mut registry = Registry::default();
        metrics.register(&mut registry);

        let service_info = ServiceInfo {
            plan_monthly_data: 322122547200,
            data_counter: 4804320660,
            data_next_reset: 1783744807,
            error: 0,
        };

        metrics.update("1379859", &service_info);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer
            .contains(r#"cloud_64clouds_plan_monthly_data_bytes{veid="1379859"} 322122547200"#));
        assert!(buffer.contains(r#"cloud_64clouds_data_counter_bytes{veid="1379859"} 4804320660"#));
        assert!(buffer
            .contains(r#"cloud_64clouds_data_next_reset_timestamp{veid="1379859"} 1783744807"#));
    }

    #[test]
    fn test_metrics_update_multiple_veids() {
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

        metrics.update("111", &service_info1);
        metrics.update("222", &service_info2);

        let mut buffer = String::new();
        encode(&mut buffer, &registry).unwrap();

        assert!(buffer.contains(r#"cloud_64clouds_plan_monthly_data_bytes{veid="111"} 100"#));
        assert!(buffer.contains(r#"cloud_64clouds_plan_monthly_data_bytes{veid="222"} 200"#));
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

        metrics.update("1379859", &service_info1);
        metrics.update("1379859", &service_info2);

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
}
