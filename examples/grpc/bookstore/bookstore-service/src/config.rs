use std::{
    env,
    net::{IpAddr, SocketAddr},
    path::Path,
    sync::OnceLock,
};

use config::{Config, Environment, File};
use serde::Deserialize;

use crate::error::AppResult;

/// Application configuration.
///
/// Contains all configuration settings for the bookstore service,
/// including server, authentication, database, and tracing settings.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Distribution metadata configuration
    pub distribution: DistributionConfig,
    /// Node-specific configuration
    pub node: NodeConfig,
    /// Server configuration settings
    pub server: ServerConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Tracing configuration
    pub tracing: TracingConfig,
}

/// Server configuration settings.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    /// gRPC server bind address
    pub grpc_address: SocketAddr,
}

/// Authentication configuration.
///
/// Supports either memory-based authentication or JWT-based authentication.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum AuthConfig {
    /// Memory-based authentication (no authentication)
    Memory,
    /// JWT-based authentication
    Jwt(JwtConfig),
}

/// JWT authentication configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key for token validation
    pub secret: String,
    /// Whether to validate token expiration
    pub validate_expiration: Option<bool>,
}

/// Database configuration.
///
/// Supports either in-memory storage or `PostgreSQL` database.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum DatabaseConfig {
    /// In-memory database storage
    Memory,
    /// `PostgreSQL` database connection
    Postgres(PostgresConfig),
}

/// `PostgreSQL` database configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct PostgresConfig {
    /// `PostgreSQL` connection string
    pub connection: String,
}

/// Tracing configuration.
///
/// Controls how tracing data is output from the service.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum TracingConfig {
    /// In-memory tracing (no output)
    Memory,
    /// Standard output tracing
    Stdout,
}

/// Distribution metadata configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct DistributionConfig {
    /// Distribution name
    pub name: String,
    /// Distribution version
    pub version: Option<String>,
}

/// Node-specific configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct NodeConfig {
    /// Host IP address of the node
    pub host_ip: IpAddr,
    /// Worker number for this node instance
    pub worker_number: u16,
}

const CONFIG_PATH_ENV: &str = "BOOKSTORE_CONFIG_PATH";
const ENV_PREFIX: &str = "BOOKSTORE";
const VERSION: &str = env!("CARGO_PKG_VERSION");

const DISTRIBUTION_VERSION_KEY: &str = "distribution.version";
const NODE_HOST_IP_KEY: &str = "node.host_ip";
const NODE_WORKER_NUMBER_KEY: &str = "node.worker_number";

impl AppConfig {
    /// Gets the global application configuration instance.
    ///
    /// Uses a static `OnceLock` to ensure the configuration is loaded only once.
    ///
    /// # Panics
    ///
    /// Will panic if the configuration cannot be loaded.
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<AppConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::load().unwrap())
    }

    /// Loads configuration from files and environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if configuration files cannot be read or parsed.
    ///
    /// # Panics
    ///
    /// Will panic if the default config path cannot be converted to a string.
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

/// Calculates worker number from IP address.
///
/// Uses the last two octets of an IPv4 address to generate a worker number.
///
/// # Panics
///
/// Will panic if IPv6 address is provided.
fn get_worker_number(ip: IpAddr) -> u16 {
    let ip_v4 = match ip {
        IpAddr::V4(ip) => ip,
        IpAddr::V6(_) => panic!("IPv6 is not supported"),
    };
    let octets = ip_v4.octets();
    (u16::from(octets[2]) << 8u16) | u16::from(octets[3])
}
