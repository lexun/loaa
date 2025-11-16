use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database connection mode
    pub mode: DatabaseMode,
    /// Database URL for remote mode (e.g., "127.0.0.1:8000")
    pub url: Option<String>,
    /// File path for embedded mode (e.g., "./data/loaa.db")
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseMode {
    /// In-memory database (development only, data lost on restart)
    Memory,
    /// Embedded file-based database (RocksDB backend)
    Embedded,
    /// Remote SurrealDB server
    Remote,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Web server host (e.g., "127.0.0.1")
    pub host: String,
    /// Web server port (e.g., 3000)
    pub port: u16,
    /// Whether to run MCP server in the same process
    pub include_mcp: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                mode: DatabaseMode::Memory,
                url: Some("127.0.0.1:8000".to_string()),
                path: None,
            },
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 3000,
                include_mcp: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let db_mode = std::env::var("LOAA_DB_MODE")
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "memory" => Some(DatabaseMode::Memory),
                "embedded" => Some(DatabaseMode::Embedded),
                "remote" => Some(DatabaseMode::Remote),
                _ => None,
            })
            .unwrap_or(DatabaseMode::Memory);

        let db_url = std::env::var("LOAA_DB_URL").ok();
        let db_path = std::env::var("LOAA_DB_PATH").ok().map(PathBuf::from);

        let server_host = std::env::var("LOAA_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = std::env::var("LOAA_PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3000);
        let include_mcp = std::env::var("LOAA_INCLUDE_MCP")
            .ok()
            .map(|s| s.to_lowercase() == "true" || s == "1")
            .unwrap_or(false);

        Self {
            database: DatabaseConfig {
                mode: db_mode,
                url: db_url,
                path: db_path,
            },
            server: ServerConfig {
                host: server_host,
                port: server_port,
                include_mcp,
            },
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        match self.database.mode {
            DatabaseMode::Memory => {
                // Memory mode needs no additional config
            }
            DatabaseMode::Embedded => {
                if self.database.path.is_none() {
                    return Err(
                        "Embedded database mode requires LOAA_DB_PATH to be set".to_string()
                    );
                }
            }
            DatabaseMode::Remote => {
                if self.database.url.is_none() {
                    return Err(
                        "Remote database mode requires LOAA_DB_URL to be set".to_string()
                    );
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.database.mode, DatabaseMode::Memory);
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert!(!config.server.include_mcp);
    }

    #[test]
    fn test_validate_memory_mode() {
        let config = Config {
            database: DatabaseConfig {
                mode: DatabaseMode::Memory,
                url: None,
                path: None,
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_embedded_mode_missing_path() {
        let config = Config {
            database: DatabaseConfig {
                mode: DatabaseMode::Embedded,
                url: None,
                path: None,
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validate_remote_mode_missing_url() {
        let config = Config {
            database: DatabaseConfig {
                mode: DatabaseMode::Remote,
                url: None,
                path: None,
            },
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
