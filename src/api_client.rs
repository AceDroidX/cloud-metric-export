use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;
use tracing::{error, info};

#[derive(Debug, Deserialize)]
pub struct ServiceInfo {
    #[serde(rename = "plan_monthly_data")]
    pub plan_monthly_data: u64,

    #[serde(rename = "data_counter")]
    pub data_counter: u64,

    #[serde(rename = "data_next_reset")]
    pub data_next_reset: u64,

    #[serde(rename = "error")]
    pub error: u64,
}

pub struct CloudClient {
    client: Client,
    veid: String,
    api_key: String,
}

impl CloudClient {
    pub fn new(veid: String, api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        CloudClient {
            client,
            veid,
            api_key,
        }
    }

    pub async fn get_service_info(&self) -> Result<ServiceInfo, String> {
        let url = format!(
            "https://api.64clouds.com/v1/getServiceInfo?veid={}&api_key={}",
            self.veid, self.api_key
        );

        info!("Fetching service info from 64clouds API");

        let response = self.client.get(&url).send().await.map_err(|e| {
            error!("Failed to send request: {}", e);
            format!("Failed to send request: {}", e)
        })?;

        if !response.status().is_success() {
            let status = response.status();
            error!("API request failed with status: {}", status);
            return Err(format!("API request failed with status: {}", status));
        }

        let service_info: ServiceInfo = response.json().await.map_err(|e| {
            error!("Failed to parse response: {}", e);
            format!("Failed to parse response: {}", e)
        })?;

        if service_info.error != 0 {
            error!("API returned error code: {}", service_info.error);
            return Err(format!("API returned error code: {}", service_info.error));
        }

        info!("Successfully fetched service info");
        Ok(service_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_deserialization() {
        let json = r#"{
            "plan_monthly_data": 322122547200,
            "data_counter": 4804320660,
            "data_next_reset": 1783744807,
            "error": 0
        }"#;

        let service_info: ServiceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(service_info.plan_monthly_data, 322122547200);
        assert_eq!(service_info.data_counter, 4804320660);
        assert_eq!(service_info.data_next_reset, 1783744807);
        assert_eq!(service_info.error, 0);
    }

    #[test]
    fn test_service_info_deserialization_with_error() {
        let json = r#"{
            "plan_monthly_data": 0,
            "data_counter": 0,
            "data_next_reset": 0,
            "error": 1
        }"#;

        let service_info: ServiceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(service_info.error, 1);
    }

    #[test]
    fn test_service_info_deserialization_full_response() {
        let json = r#"{
            "vm_type": "kvm",
            "hostname": "nice-boot-1.localdomain",
            "node_alias": "v2922",
            "node_location_id": "USCA_9",
            "node_location": "US, California",
            "node_datacenter": "US: Los Angeles, California",
            "location_ipv6_ready": true,
            "plan": "kvmv3-10g-512m-300m-ca-cn2gia",
            "plan_monthly_data": 322122547200,
            "monthly_data_multiplier": 1,
            "plan_disk": 10737418240,
            "plan_ram": 536870912,
            "plan_swap": 0,
            "plan_max_ipv6s": 1,
            "os": "ubuntu-18.04-x86_64",
            "email": "test@example.com",
            "data_counter": 4804320660,
            "data_next_reset": 1783744807,
            "ip_addresses": ["67.230.162.134"],
            "private_ip_addresses": [],
            "ip_nullroutes": [],
            "iso1": "",
            "iso2": "",
            "available_isos": [],
            "plan_private_network_available": false,
            "location_private_network_available": true,
            "rdns_api_available": true,
            "ptr": {},
            "suspended": false,
            "policy_violation": false,
            "suspension_count": 0,
            "total_abuse_points": 0,
            "max_abuse_points": 1000,
            "plan_kiwivm_theme": "",
            "free_ip_replacement_interval": -100,
            "error": 0
        }"#;

        let service_info: ServiceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(service_info.plan_monthly_data, 322122547200);
        assert_eq!(service_info.data_counter, 4804320660);
        assert_eq!(service_info.data_next_reset, 1783744807);
        assert_eq!(service_info.error, 0);
    }

    #[test]
    fn test_cloud_client_new() {
        let client = CloudClient::new("12345".to_string(), "test_key".to_string());
        assert_eq!(client.veid, "12345");
        assert_eq!(client.api_key, "test_key");
    }
}
