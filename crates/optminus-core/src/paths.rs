//! Per-OS application data directories.
//!
//! On macOS this resolves to `~/Library/Application Support/dev.OptMinus.optminus/`,
//! on Linux to `$XDG_CONFIG_HOME/optminus/` (default `~/.config/optminus/`),
//! and on Windows to `%APPDATA%\OptMinus\optminus\config\`.

use std::path::PathBuf;

use directories::ProjectDirs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PathsError {
    #[error("could not resolve a home directory for the current user")]
    HomeNotFound,
}

fn project_dirs() -> Result<ProjectDirs, PathsError> {
    ProjectDirs::from("dev", "OptMinus", "optminus").ok_or(PathsError::HomeNotFound)
}

pub fn config_dir() -> Result<PathBuf, PathsError> {
    Ok(project_dirs()?.config_dir().to_path_buf())
}

pub fn config_path() -> Result<PathBuf, PathsError> {
    Ok(config_dir()?.join("config.toml"))
}
