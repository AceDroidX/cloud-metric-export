use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub cloud_64clouds_veid: String,
    pub cloud_64clouds_api_key: String,
    pub metrics_port: u16,
    pub fetch_interval_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let cloud_64clouds_veid = env::var("CLOUD_64CLOUDS_VEID")
            .map_err(|_| "CLOUD_64CLOUDS_VEID environment variable is required")?;

        let cloud_64clouds_api_key = env::var("CLOUD_64CLOUDS_API_KEY")
            .map_err(|_| "CLOUD_64CLOUDS_API_KEY environment variable is required")?;

        let metrics_port = env::var("METRICS_PORT")
            .unwrap_or_else(|_| "9090".to_string())
            .parse::<u16>()
            .map_err(|_| "METRICS_PORT must be a valid port number")?;

        let fetch_interval_seconds = env::var("FETCH_INTERVAL")
            .unwrap_or_else(|_| "60".to_string())
            .parse::<u64>()
            .map_err(|_| "FETCH_INTERVAL must be a valid number of seconds")?;

        Ok(Config {
            cloud_64clouds_veid,
            cloud_64clouds_api_key,
            metrics_port,
            fetch_interval_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_config_from_env_required_vars() {
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");

        let config = Config::from_env().unwrap();
        assert_eq!(config.cloud_64clouds_veid, "12345");
        assert_eq!(config.cloud_64clouds_api_key, "test_key");
        assert_eq!(config.metrics_port, 9090);
        assert_eq!(config.fetch_interval_seconds, 60);

        env::remove_var("CLOUD_64CLOUDS_VEID");
        env::remove_var("CLOUD_64CLOUDS_API_KEY");
    }

    #[test]
    fn test_config_from_env_custom_values() {
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");
        env::set_var("METRICS_PORT", "8080");
        env::set_var("FETCH_INTERVAL", "30");

        let config = Config::from_env().unwrap();
        assert_eq!(config.metrics_port, 8080);
        assert_eq!(config.fetch_interval_seconds, 30);

        env::remove_var("CLOUD_64CLOUDS_VEID");
        env::remove_var("CLOUD_64CLOUDS_API_KEY");
        env::remove_var("METRICS_PORT");
        env::remove_var("FETCH_INTERVAL");
    }

    #[test]
    fn test_config_from_env_missing_veid() {
        env::remove_var("CLOUD_64CLOUDS_VEID");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("CLOUD_64CLOUDS_VEID"));

        env::remove_var("CLOUD_64CLOUDS_API_KEY");
    }

    #[test]
    fn test_config_from_env_missing_api_key() {
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::remove_var("CLOUD_64CLOUDS_API_KEY");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("CLOUD_64CLOUDS_API_KEY"));

        env::remove_var("CLOUD_64CLOUDS_VEID");
    }

    #[test]
    fn test_config_from_env_invalid_port() {
        env::set_var("CLOUD_64CLOUDS_VEID", "12345");
        env::set_var("CLOUD_64CLOUDS_API_KEY", "test_key");
        env::set_var("METRICS_PORT", "invalid");

        let result = Config::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("METRICS_PORT"));

        env::remove_var("CLOUD_64CLOUDS_VEID");
        env::remove_var("CLOUD_64CLOUDS_API_KEY");
        env::remove_var("METRICS_PORT");
    }
}
