//! User configuration, persisted as TOML.
//!
//! v0.0.1 has no real settings yet — the `version` field is present from day
//! one so future migrations can branch on it without a flag day.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::paths::{self, PathsError};

pub const CURRENT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Settings {}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("could not resolve config path")]
    Path(#[from] PathsError),
    #[error("could not read config at {path}")]
    Read {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("could not parse config at {path}")]
    Parse {
        path: std::path::PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("could not write config at {path}")]
    Write {
        path: std::path::PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("could not serialize config")]
    Serialize(#[from] toml::ser::Error),
}

impl Config {
    /// Loads the config from the default path, returning `Config::default()`
    /// if the file does not exist yet.
    pub fn load_or_default() -> Result<Self, ConfigError> {
        let path = paths::config_path()?;
        match fs::read_to_string(&path) {
            Ok(text) => toml::from_str(&text).map_err(|source| ConfigError::Parse { path, source }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(source) => Err(ConfigError::Read { path, source }),
        }
    }

    /// Writes the config atomically: serialize to a sibling temp file, then
    /// rename over the target. On Unix the temp file is created with mode 0600.
    pub fn save_atomic(&self) -> Result<(), ConfigError> {
        let path = paths::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| ConfigError::Write {
                path: path.clone(),
                source,
            })?;
        }
        let body = toml::to_string_pretty(self)?;
        write_atomic(&path, body.as_bytes()).map_err(|source| ConfigError::Write { path, source })
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let tmp = path.with_extension("toml.tmp");
    {
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&tmp)?;
            io::Write::write_all(&mut f, bytes)?;
            f.sync_all()?;
        }
        #[cfg(not(unix))]
        {
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&tmp)?;
            io::Write::write_all(&mut f, bytes)?;
            f.sync_all()?;
        }
    }
    fs::rename(&tmp, path)
}
