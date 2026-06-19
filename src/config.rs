use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // 64clouds - optional
    pub cloud_64clouds_veid: Option<String>,
    pub cloud_64clouds_api_key: Option<String>,
    pub cloud_64clouds_fetch_interval: u64,

    // AWS - optional
    pub aws_access_key_id: Option<String>,
    pub aws_secret_access_key: Option<String>,
    pub aws_region: Option<String>,
    pub aws_lightsail_instance_name: Option<String>,
    pub aws_lightsail_fetch_interval: u64,

    pub metrics_port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let cloud_64clouds_veid = env::var("CLOUD_64CLOUDS_VEID").ok();
        let cloud_64clouds_api_key = env::var("CLOUD_64CLOUDS_API_KEY").ok();

        let cloud_64clouds_fetch_interval = env::var("CLOUD_64CLOUDS_FETCH_INTERVAL")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .map_err(|_| "CLOUD_64CLOUDS_FETCH_INTERVAL must be a valid number of seconds")?;

        let aws_access_key_id = env::var("AWS_ACCESS_KEY_ID").ok();
        let aws_secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").ok();
        let aws_region = env::var("AWS_REGION").ok();
        let aws_lightsail_instance_name = env::var("AWS_LIGHTSAIL_INSTANCE_NAME").ok();

        let aws_lightsail_fetch_interval = env::var("AWS_LIGHTSAIL_FETCH_INTERVAL")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .map_err(|_| "AWS_LIGHTSAIL_FETCH_INTERVAL must be a valid number of seconds")?;

        let metrics_port = env::var("METRICS_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse::<u16>()
            .map_err(|_| "METRICS_PORT must be a valid port number")?;

        // Validate that at least one data source is configured
        let has_64clouds = cloud_64clouds_veid.is_some() && cloud_64clouds_api_key.is_some();
        let has_aws = aws_access_key_id.is_some()
            && aws_secret_access_key.is_some()
            && aws_region.is_some()
            && aws_lightsail_instance_name.is_some();

        if !has_64clouds && !has_aws {
            return Err(
                "At least one data source must be configured: either 64clouds (CLOUD_64CLOUDS_VEID + CLOUD_64CLOUDS_API_KEY) or AWS (AWS_ACCESS_KEY_ID + AWS_SECRET_ACCESS_KEY + AWS_REGION + AWS_LIGHTSAIL_INSTANCE_NAME)".to_string()
            );
        }

        Ok(Config {
            cloud_64clouds_veid,
            cloud_64clouds_api_key,
            cloud_64clouds_fetch_interval,
            aws_access_key_id,
            aws_secret_access_key,
            aws_region,
            aws_lightsail_instance_name,
            aws_lightsail_fetch_interval,
            metrics_port,
        })
    }

    pub fn has_64clouds(&self) -> bool {
        self.cloud_64clouds_veid.is_some() && self.cloud_64clouds_api_key.is_some()
    }

    pub fn has_aws(&self) -> bool {
        self.aws_access_key_id.is_some()
            && self.aws_secret_access_key.is_some()
            && self.aws_region.is_some()
            && self.aws_lightsail_instance_name.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    fn clean_env() {
        env::remove_var("CLOUD_64CLOUDS_VEID");
        env::remove_var("CLOUD_64CLOUDS_API_KEY");
        env::remove_var("CLOUD_64CLOUDS_FETCH_INTERVAL");
        env::remove_var("AWS_ACCESS_KEY_ID");
        env::remove_var("AWS_SECRET_ACCESS_KEY");
        env::remove_var("AWS_REGION");
        env::remove_var("AWS_LIGHTSAIL_INSTANCE_NAME");
        env::remove_var("AWS_LIGHTSAIL_FETCH_INTERVAL");
        env::remove_var("METRICS_PORT");
    }

    #[test]
    #[serial]
    fn test_config_from_env_both_sources() {
        clean_env();
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");
        env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
        env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
        env::set_var("AWS_REGION", "ap-northeast-1");
        env::set_var("AWS_LIGHTSAIL_INSTANCE_NAME", "jp-1");

        let config = Config::from_env().unwrap();
        assert!(config.has_64clouds());
        assert!(config.has_aws());
        assert_eq!(config.cloud_64clouds_veid, Some("12345".to_string()));
        assert_eq!(
            config.aws_access_key_id,
            Some("test_access_key".to_string())
        );
        assert_eq!(config.metrics_port, 9090);

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_only_64clouds() {
        clean_env();
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");

        let config = Config::from_env().unwrap();
        assert!(config.has_64clouds());
        assert!(!config.has_aws());
        assert_eq!(config.cloud_64clouds_veid, Some("12345".to_string()));
        assert!(config.aws_access_key_id.is_none());

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_only_aws() {
        clean_env();
        env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
        env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
        env::set_var("AWS_REGION", "ap-northeast-1");
        env::set_var("AWS_LIGHTSAIL_INSTANCE_NAME", "jp-1");

        let config = Config::from_env().unwrap();
        assert!(!config.has_64clouds());
        assert!(config.has_aws());
        assert!(config.cloud_64clouds_veid.is_none());
        assert_eq!(
            config.aws_access_key_id,
            Some("test_access_key".to_string())
        );

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_no_sources() {
        clean_env();

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("At least one data source must be configured"));

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_incomplete_64clouds() {
        clean_env();
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        // Missing CLOUD_64CLOUDS_API_KEY
        env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
        env::set_var("AWS_SECRET_ACCESS_KEY", "test_secret_key");
        env::set_var("AWS_REGION", "ap-northeast-1");
        env::set_var("AWS_LIGHTSAIL_INSTANCE_NAME", "jp-1");

        let config = Config::from_env().unwrap();
        assert!(!config.has_64clouds());
        assert!(config.has_aws());

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_incomplete_aws() {
        clean_env();
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");
        env::set_var("AWS_ACCESS_KEY_ID", "test_access_key");
        // Missing other AWS vars

        let config = Config::from_env().unwrap();
        assert!(config.has_64clouds());
        assert!(!config.has_aws());

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_custom_values() {
        clean_env();
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");
        env::set_var("CLOUD_64CLOUDS_FETCH_INTERVAL", "120");
        env::set_var("METRICS_PORT", "8080");

        let config = Config::from_env().unwrap();
        assert_eq!(config.cloud_64clouds_fetch_interval, 120);
        assert_eq!(config.metrics_port, 8080);

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_invalid_port() {
        clean_env();
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");
        env::set_var("METRICS_PORT", "invalid");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("METRICS_PORT"));

        clean_env();
    }

    #[test]
    #[serial]
    fn test_config_from_env_invalid_interval() {
        clean_env();
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");
        env::set_var("CLOUD_64CLOUDS_FETCH_INTERVAL", "invalid");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("CLOUD_64CLOUDS_FETCH_INTERVAL"));

        clean_env();
    }
}
