use std::{
    env,
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::OnceLock,
};

use config::{Config, Environment, File};
use serde::Deserialize;

use crate::error::AppResult;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub distribution: DistributionConfig,
    pub node: NodeConfig,
    pub server: ServerConfig,
    pub auth: AuthConfig,
    pub database: DatabaseConfig,
    pub tracing: TracingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub grpc_address: SocketAddr,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum AuthConfig {
    Memory,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum DatabaseConfig {
    Memory,
    Postgres(PostgresConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    pub connection: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TracingConfig {
    Memory,
    Stdout,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributionConfig {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    pub host_ip: IpAddr,
    pub worker_number: u16,
}

const CONFIG_PATH_ENV: &str = "BOOKSTORE_CONFIG_PATH";
const ENV_PREFIX: &str = "BOOKSTORE";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const DISTRIBUTION_VERSION_KEY: &str = "distribution.version";
const NODE_HOST_IP_KEY: &str = "node.host_ip";
const NODE_WORKER_NUMBER_KEY: &str = "node.worker_number";

impl AppConfig {
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<AppConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::load().unwrap())
    }

    pub fn load() -> AppResult<Self> {
        let config_path = env::var(CONFIG_PATH_ENV).unwrap_or_else(|_| "config".to_string());

        let mut config_builder =
            Config::builder().set_default(DISTRIBUTION_VERSION_KEY, VERSION)?;

        // Initial "default" configuration file
        let default_path = Path::new(&config_path).join("default");
        config_builder = config_builder.add_source(File::with_name(default_path.to_str().unwrap()));

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        let local_path = Path::new(&config_path).join("local");
        config_builder = config_builder
            .add_source(File::with_name(local_path.to_str().unwrap()).required(false));

        // Add override settings file.
        let override_path = env::var(CONFIG_PATH_ENV).ok();
        if let Some(override_path) = override_path {
            config_builder =
                config_builder.add_source(File::with_name(&override_path).required(false));
        }

        // Add in settings from the environment (with a prefix of BOOKSTORE)
        config_builder =
            config_builder.add_source(Environment::with_prefix(ENV_PREFIX).separator("__"));

        // Set derived properties
        let config = config_builder.build()?;
        let mut config_builder = Config::builder();
        if let Ok(node_host_ip) = config.get::<IpAddr>(NODE_HOST_IP_KEY) {
            config_builder = config_builder
                .set_default(NODE_WORKER_NUMBER_KEY, get_worker_number(node_host_ip))?;
        }

        Ok(config_builder
            .add_source(config)
            .build()?
            .try_deserialize()?)
    }
}

fn get_worker_number(ip: IpAddr) -> u16 {
    let ip_v4 = match ip {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => panic!("IPv6 is not supported"),
    };
    let octets = ip_v4.octets();
    (u16::from(octets[2]) << 8u16) | u16::from(octets[3])
}
