//! Configuration and credential management for GitLab CLI.
//!
//! Stores and retrieves authentication tokens and instance preferences across multiple
//! GitLab hosts (e.g., `gitlab.com` alongside self-hosted instances).
//! Configuration files are persisted in the user's config directory (typically `~/.config/gitlab-cli/config.toml`).

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Default host used when none is specified.
pub const DEFAULT_GITLAB_HOST: &str = "gitlab.com";

/// Configuration entry for a single GitLab host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HostConfig {
    /// Authentication token (Personal Access Token, Project Token, or OAuth Bearer token).
    pub token: String,
    /// Optional username associated with this token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// Root configuration data structure holding multi-host settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Config {
    /// The default host to use when outside a Git repository and no `--host` flag is provided.
    #[serde(default = "default_host")]
    pub default_host: String,

    /// Mapping of hostnames (e.g., `gitlab.com`, `gitlab.example.com`) to host configurations.
    #[serde(default)]
    pub hosts: HashMap<String, HostConfig>,
}

fn default_host() -> String {
    DEFAULT_GITLAB_HOST.to_string()
}

impl Config {
    /// Returns the standard path to the configuration file (`~/.config/gitlab-cli/config.toml`).
    ///
    /// # Errors
    ///
    /// Returns an error if the user home directory cannot be determined.
    pub fn config_path() -> anyhow::Result<PathBuf> {
        if let Some(dirs) = ProjectDirs::from("com", "gitlab-cli", "gitlab-cli") {
            let config_dir = dirs.config_dir();
            Ok(config_dir.join("config.toml"))
        } else {
            anyhow::bail!("Unable to determine standard configuration directory")
        }
    }

    /// Loads the configuration from the standard configuration file path.
    ///
    /// If the file does not exist, an empty default configuration is returned.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed as TOML.
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path()?;
        Self::load_from_path(&path)
    }

    /// Loads configuration from an explicit file path.
    ///
    /// If the file does not exist, a default configuration is returned.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the TOML configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if reading or deserializing fails.
    pub fn load_from_path(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self {
                default_host: DEFAULT_GITLAB_HOST.to_string(),
                hosts: HashMap::new(),
            });
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Saves the current configuration to the standard configuration file path.
    ///
    /// Creates parent directories if they do not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if serializing or writing to disk fails.
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()?;
        self.save_to_path(&path)
    }

    /// Saves configuration to an explicit file path.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to write the serialized configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to disk fails.
    pub fn save_to_path(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = toml::to_string_pretty(self)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    /// Sets or updates authentication credentials for a specific GitLab host.
    ///
    /// # Arguments
    ///
    /// * `host` - The GitLab instance hostname.
    /// * `token` - The access token.
    /// * `user` - An optional username.
    pub fn set_host_token(
        &mut self,
        host: impl Into<String>,
        token: impl Into<String>,
        user: Option<String>,
    ) {
        let host_str = host.into();
        self.hosts.insert(
            host_str,
            HostConfig {
                token: token.into(),
                user,
            },
        );
    }

    /// Removes authentication credentials for a specific GitLab host.
    ///
    /// # Arguments
    ///
    /// * `host` - The host to remove.
    ///
    /// # Returns
    ///
    /// `true` if the host existed and was removed, `false` otherwise.
    pub fn remove_host(&mut self, host: &str) -> bool {
        self.hosts.remove(host).is_some()
    }

    /// Retrieves the token for a specific host from config, or from the default host if unspecified.
    ///
    /// # Arguments
    ///
    /// * `host` - An optional hostname.
    pub fn get_token(&self, host: Option<&str>) -> Option<String> {
        let target_host = host.unwrap_or(&self.default_host);
        self.hosts.get(target_host).map(|h| h.token.clone())
    }

    /// Resolves the effective host and authentication token.
    ///
    /// Resolution order:
    /// 1. Host: CLI `--host` flag > Git remote host > `GITLAB_HOST` env > `config.default_host` > `gitlab.com`.
    /// 2. Token: `GITLAB_TOKEN` env > Stored token for resolved host.
    ///
    /// # Arguments
    ///
    /// * `cli_host` - Optional host passed via CLI argument.
    /// * `git_host` - Optional host detected from Git remote.
    ///
    /// # Errors
    ///
    /// Returns an error if no authentication token is configured or found in environment variables.
    pub fn resolve_auth(
        &self,
        cli_host: Option<&str>,
        git_host: Option<&str>,
    ) -> anyhow::Result<(String, String)> {
        // Resolve host
        let host = if let Some(h) = cli_host {
            h.to_string()
        } else if let Some(h) = git_host {
            h.to_string()
        } else if let Ok(h) = std::env::var("GITLAB_HOST") {
            if !h.trim().is_empty() {
                h.trim().to_string()
            } else {
                self.default_host.clone()
            }
        } else if !self.default_host.is_empty() {
            self.default_host.clone()
        } else {
            DEFAULT_GITLAB_HOST.to_string()
        };

        // Resolve token: check GITLAB_TOKEN or CI_JOB_TOKEN environment variable first
        if let Ok(env_token) = std::env::var("GITLAB_TOKEN") {
            if !env_token.trim().is_empty() {
                return Ok((host, env_token.trim().to_string()));
            }
        }
        if let Ok(ci_token) = std::env::var("CI_JOB_TOKEN") {
            if !ci_token.trim().is_empty() {
                return Ok((host, ci_token.trim().to_string()));
            }
        }

        // Look up token in stored host config
        if let Some(host_cfg) = self.hosts.get(&host) {
            return Ok((host, host_cfg.token.clone()));
        }

        anyhow::bail!(
            "No authentication token found for GitLab host '{host}'. \
             Please run `gitlab auth login --host {host}` or set the `GITLAB_TOKEN` environment variable."
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let mut config = Config {
            default_host: "gitlab.mycorp.com".to_string(),
            hosts: HashMap::new(),
        };

        config.set_host_token("gitlab.mycorp.com", "token-123", Some("alice".to_string()));
        config.set_host_token("gitlab.com", "token-456", None);

        config.save_to_path(path).unwrap();

        let loaded = Config::load_from_path(path).unwrap();
        assert_eq!(loaded.default_host, "gitlab.mycorp.com");
        assert_eq!(
            loaded.hosts.get("gitlab.mycorp.com").unwrap().token,
            "token-123"
        );
        assert_eq!(
            loaded.hosts.get("gitlab.mycorp.com").unwrap().user,
            Some("alice".to_string())
        );
        assert_eq!(loaded.hosts.get("gitlab.com").unwrap().token, "token-456");
        assert_eq!(loaded.hosts.get("gitlab.com").unwrap().user, None);
    }

    #[test]
    fn test_remove_host() {
        let mut config = Config::default();
        config.set_host_token("gitlab.com", "tok", None);
        assert!(config.remove_host("gitlab.com"));
        assert!(!config.remove_host("gitlab.com"));
        assert_eq!(config.get_token(Some("gitlab.com")), None);
    }

    #[test]
    fn test_resolve_auth_with_env_token() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("GITLAB_TOKEN", "env-secret-token");
        std::env::remove_var("GITLAB_HOST");

        let config = Config::default();
        let (host, token) = config.resolve_auth(None, None).unwrap();
        assert_eq!(host, "gitlab.com");
        assert_eq!(token, "env-secret-token");

        std::env::remove_var("GITLAB_TOKEN");
    }

    #[test]
    fn test_resolve_auth_with_config_token() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("GITLAB_TOKEN");
        std::env::remove_var("CI_JOB_TOKEN");
        std::env::remove_var("GITLAB_HOST");

        let mut config = Config::default();
        config.set_host_token("gitlab.example.com", "cfg-token", None);

        let (host, token) = config
            .resolve_auth(Some("gitlab.example.com"), None)
            .unwrap();
        assert_eq!(host, "gitlab.example.com");
        assert_eq!(token, "cfg-token");
    }

    #[test]
    fn test_resolve_auth_missing_token() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("GITLAB_TOKEN");
        std::env::remove_var("CI_JOB_TOKEN");

        let config = Config::default();
        let res = config.resolve_auth(Some("unknown.gitlab.com"), None);
        assert!(res.is_err());
    }
}
