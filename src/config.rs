use serde::{Deserialize, Serialize};
use config as config_crate;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub kubernetes: KubernetesConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct KubernetesConfig {
    pub cluster_url: Option<String>,
    pub kubeconfig_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub api_token: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let settings = config_crate::Config::builder()
            .add_source(config_crate::File::from(path.as_ref()))
            .add_source(config_crate::Environment::with_prefix("APP").separator("_"))
            .build()?;
        
        Ok(settings.try_deserialize::<Config>()?)
    }
    
    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
