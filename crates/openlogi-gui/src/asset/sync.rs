//! Startup-time HTTP sync against `assets.openlogi.org`.
//!
//! Runs **before** the GUI opens. For each connected device with a
//! [`DeviceModelInfo`], resolves the matching depot from the freshly-
//! fetched `index.json`, then downloads any per-device files we don't
//! already have cached (or whose sha256 differs). Failures are logged
//! and swallowed — the GUI falls back to whatever's currently on disk
//! and ultimately to the synthetic silhouette.

use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use openlogi_core::device::DeviceModelInfo;
use sha2::{Digest, Sha256};
use tracing::{debug, info, warn};

use super::AssetCache;
use super::index::{DeviceEntry, Index};

/// Default origin for asset fetches. Overridable via `OPENLOGI_ASSETS`
/// so dev / staging deployments can point elsewhere without a rebuild.
pub const DEFAULT_BASE: &str = "https://assets.openlogi.org";

/// Files the GUI actually opens. We only fetch these; the rest of each
/// depot stays remote until a feature needs it.
const FETCH_FILES: &[&str] = &["front_core.png", "core_metadata.json"];

const INDEX_PATH: &str = "index.json";

/// Refresh the local cache for every model the host can plausibly want.
///
/// `models` is what `openlogi_hid::enumerate` reported — usually one
/// entry, occasionally more. The `OPENLOGI_FORCE_DEPOT` override is
/// honoured so devs without the physical hardware can still test.
pub fn sync(server: &str, models: &[DeviceModelInfo]) -> Result<()> {
    let cache_root = AssetCache::new().cache_root().to_path_buf();
    fs::create_dir_all(&cache_root)
        .with_context(|| format!("create cache root {}", cache_root.display()))?;

    let index = match refresh_index(server, &cache_root) {
        Ok(idx) => idx,
        Err(e) => {
            warn!(error = ?e, "index.json fetch failed — proceeding with cached files");
            return Ok(());
        }
    };

    let mut targets: Vec<(String, DeviceEntry)> = Vec::new();
    if let Ok(forced) = std::env::var("OPENLOGI_FORCE_DEPOT")
        && let Some(entry) = index.devices.get(&forced)
    {
        targets.push((forced, entry.clone()));
    }
    for model in models {
        if let Some((depot, entry)) = super::resolve_in_index(&index, model) {
            targets.push((depot.to_string(), entry.clone()));
        }
    }
    targets.sort_by(|a, b| a.0.cmp(&b.0));
    targets.dedup_by(|a, b| a.0 == b.0);

    if targets.is_empty() {
        debug!("sync: no matching depots for connected devices");
        return Ok(());
    }

    for (depot, entry) in &targets {
        if let Err(e) = sync_depot(server, &cache_root, depot, entry) {
            warn!(depot, error = %e, "depot sync failed");
        }
    }
    info!(devices = targets.len(), "asset sync complete");
    Ok(())
}

fn refresh_index(server: &str, cache_root: &Path) -> Result<Index> {
    let url = format!("{}/{INDEX_PATH}", server.trim_end_matches('/'));
    debug!(%url, "fetching index.json");
    let body = http_get_bytes(&url)?;
    let local = cache_root.join(INDEX_PATH);
    fs::write(&local, &body).with_context(|| format!("write {}", local.display()))?;
    let parsed: Index = serde_json::from_slice(&body).context("parse fetched index.json")?;
    debug!(devices = parsed.devices.len(), "index.json refreshed");
    Ok(parsed)
}

fn sync_depot(
    server: &str,
    cache_root: &Path,
    depot: &str,
    entry: &DeviceEntry,
) -> Result<()> {
    let dir = cache_root.join(depot);
    fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;

    for name in FETCH_FILES {
        let Some(file_entry) = entry.files.iter().find(|f| f.name == *name) else {
            warn!(depot, file = name, "registry lists no entry for required file");
            continue;
        };
        let dst: PathBuf = dir.join(name);
        if cached_matches(&dst, &file_entry.sha256) {
            debug!(depot, file = name, "cache hit");
            continue;
        }
        let url = format!(
            "{}/{}{}",
            server.trim_end_matches('/'),
            entry.asset_path.trim_start_matches('/'),
            name
        );
        debug!(%url, depot, "fetching");
        let bytes = http_get_bytes(&url)?;
        fs::write(&dst, &bytes).with_context(|| format!("write {}", dst.display()))?;
        info!(depot, file = name, bytes = bytes.len(), "downloaded");
    }
    Ok(())
}

fn cached_matches(path: &Path, expected_sha: &str) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(_) => return false,
        }
    }
    let actual = format!("{:x}", hasher.finalize());
    actual.eq_ignore_ascii_case(expected_sha)
}

fn http_get_bytes(url: &str) -> Result<Vec<u8>> {
    let mut response = ureq::get(url)
        .call()
        .with_context(|| format!("GET {url}"))?;
    let mut body = Vec::new();
    response
        .body_mut()
        .as_reader()
        .read_to_end(&mut body)
        .with_context(|| format!("read body {url}"))?;
    Ok(body)
}
