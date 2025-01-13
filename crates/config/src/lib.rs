#[cfg(feature = "network")]
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use hyveos_core::DAEMON_NAME;
use serde::{Deserialize, Serialize};
use tracing::level_filters::LevelFilter;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(default)]
    pub interfaces: Option<Vec<String>>,
    #[cfg(feature = "batman")]
    #[serde(default)]
    pub batman_interface: Option<String>,
    #[serde(default)]
    pub wifi_interface: Option<String>,
    #[serde(default)]
    pub store_directory: Option<PathBuf>,
    #[serde(default)]
    pub db_file: Option<PathBuf>,
    #[serde(default)]
    pub key_file: Option<PathBuf>,
    #[serde(default)]
    pub random_directory: bool,
    #[serde(default)]
    pub script_management: Option<ScriptManagementConfig>,
    #[serde(default)]
    pub log_dir: Option<PathBuf>,
    #[serde(default)]
    pub log_level: LogFilter,
    #[serde(default)]
    pub cli_socket_path: Option<PathBuf>,
    #[cfg(feature = "network")]
    #[serde(default, deserialize_with = "deserialize_socket_addr")]
    pub cli_socket_addr: Option<SocketAddr>,
}

impl Config {
    pub fn load(path: Option<impl AsRef<Path>>) -> anyhow::Result<Self> {
        if let Some(path) = path {
            std::fs::read_to_string(path)
                .map(|s| toml::from_str(&s))?
                .map_err(Into::into)
        } else {
            let base_paths = [Path::new("/etc"), Path::new("/usr/lib")];

            for base_path in &base_paths {
                let path = base_path.join(DAEMON_NAME).join("config.toml");
                match std::fs::read_to_string(&path) {
                    Ok(s) => {
                        let config = toml::from_str(&s)?;
                        tracing::debug!("Loaded config file from {}", path.display());
                        return Ok(config);
                    }
                    Err(_) => {
                        tracing::info!("Failed to load config file from {}", path.display());
                    }
                }
            }

            Ok(Self::default())
        }
    }

    pub fn save(&self, path: Option<impl AsRef<Path>>) -> anyhow::Result<()> {
        if let Some(path) = path {
            let s = toml::to_string(self)?;
            std::fs::write(path, s)?
        } else {
            let base_path = Path::new("/etc");

            let path = base_path.join(DAEMON_NAME).join("config.toml");
            let s = toml::to_string(self)?;
            std::fs::write(&path, s)?;
            tracing::debug!("Saved config file to {}", path.display());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum ScriptManagementConfig {
    Allow,
    #[default]
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum LogFilter {
    None,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogFilter> for LevelFilter {
    fn from(value: LogFilter) -> Self {
        match value {
            LogFilter::None => Self::OFF,
            LogFilter::Error => Self::ERROR,
            LogFilter::Warn => Self::WARN,
            LogFilter::Info => Self::INFO,
            LogFilter::Debug => Self::DEBUG,
            LogFilter::Trace => Self::TRACE,
        }
    }
}

#[cfg(feature = "network")]
fn deserialize_socket_addr<'de, D>(deserializer: D) -> Result<Option<SocketAddr>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    let s: Option<&str> = Deserialize::deserialize(deserializer)?;

    if let Some(s) = s {
        parse_socket_addr(s)
            .map_err(serde::de::Error::custom)
            .map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(feature = "network")]
pub fn parse_socket_addr(s: &str) -> Result<SocketAddr, anyhow::Error> {
    use std::str::FromStr;

    use hyveos_ifaddr::IfAddr;

    if let Some(s) = s.strip_prefix("[") {
        let mut parts = s.splitn(2, "]:");
        let host = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Missing host"))
            .and_then(|addr| IfAddr::from_str(addr).map_err(Into::into))?;
        let port = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Missing port"))
            .and_then(|port| port.parse().map_err(Into::into))?;

        host.with_port(port).map_err(Into::into)
    } else {
        s.parse().map_err(Into::into)
    }
}
