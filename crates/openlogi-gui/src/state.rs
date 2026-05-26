//! App-wide UI state stored as a GPUI global.
//!
//! Anything that more than one view needs to read (current device, currently
//! armed button, the DPI value the panel and the dot-preview share) lives
//! here. Per-component scratch state (hover index, gesture point buffer) stays
//! in the owning entity.
//!
//! [`AppState::with_runtime`] resolves every paired device's asset + DPI
//! target up front so views can switch instantly when the carousel selection
//! changes — no synchronous I/O during the device switch.

#![allow(
    dead_code,
    reason = "fields are read once their owning component lands in UI.md phases 2–4"
)]

use std::collections::BTreeMap;

use gpui::Global;
use openlogi_core::config::Config;
use openlogi_core::device::DeviceInventory;
use tracing::{debug, warn};

use crate::asset::{AssetCache, ResolvedAsset};
use crate::components::dpi_panel::DpiTarget;
use crate::data::mouse_buttons::{Action, ButtonId, default_binding};

/// Default DPI value applied to a fresh AppState. Matches a common Logitech
/// mid-range mouse and keeps the dot-preview visually obvious from frame one.
pub const DEFAULT_DPI: u32 = 1600;

/// One paired device with everything the UI needs to switch to it in O(1):
/// the config key (for bindings/DPI persistence), a display name, the
/// resolved asset (PNG + metadata, or `None` for the synthetic fallback),
/// and the routing target for HID++ DPI writes.
#[derive(Debug, Clone)]
pub struct DeviceRecord {
    pub config_key: String,
    pub display_name: String,
    pub asset: Option<ResolvedAsset>,
    pub dpi_target: Option<DpiTarget>,
}

pub struct AppState {
    /// Index into [`Self::device_list`] of the currently visible device. May
    /// be out of bounds briefly while inventories re-enumerate; views must
    /// bounds-check via [`Self::current_record`].
    pub current_device: usize,
    /// The hotspot the user most recently armed by clicking. Drives the
    /// "selected button" outline on the mouse model and the popover content.
    pub active_button: Option<ButtonId>,
    /// Bindings for the *currently selected* device. Reloaded whenever the
    /// carousel selection changes.
    pub button_bindings: BTreeMap<ButtonId, Action>,
    pub dpi: u32,
    /// All paired devices, in carousel order. Each entry caches the per-
    /// device data the views need so a switch is a pure index update.
    pub device_list: Vec<DeviceRecord>,
    /// Live config — kept in sync with disk via [`Self::commit_binding`] and
    /// [`Self::set_current_device`] so restarts preserve user bindings and
    /// the last-selected device.
    config: Config,
}

impl AppState {
    /// Build the global from a loaded config + enumerated inventories. The
    /// initial selection prefers [`Config::selected_device`] if it still
    /// matches one of the paired devices; otherwise it falls back to index 0.
    #[must_use]
    pub fn with_runtime(
        config: Config,
        inventories: &[DeviceInventory],
        cache: &AssetCache,
    ) -> Self {
        let device_list = build_device_list(inventories, cache);
        let current_device = pick_initial_device(&device_list, config.selected_device());
        let mut state = Self {
            current_device,
            active_button: None,
            button_bindings: BTreeMap::new(),
            dpi: DEFAULT_DPI,
            device_list,
            config,
        };
        state.button_bindings = state.bindings_for_current();
        state
    }

    /// The active device, or `None` when [`Self::device_list`] is empty or
    /// `current_device` is past the end.
    #[must_use]
    pub fn current_record(&self) -> Option<&DeviceRecord> {
        self.device_list.get(self.current_device)
    }

    /// Switch the carousel to `idx`. Out-of-range indices are silently
    /// ignored so callers can pass them straight through from UI events.
    /// Persists the new selection (by config key, not index — index isn't
    /// stable across restarts) and reloads bindings for the new device.
    pub fn set_current_device(&mut self, idx: usize) {
        if idx >= self.device_list.len() || idx == self.current_device {
            return;
        }
        self.current_device = idx;
        self.button_bindings = self.bindings_for_current();
        let key = self.current_record().map(|r| r.config_key.clone());
        self.config.set_selected_device(key);
        if let Err(e) = self.config.save_atomic() {
            warn!(error = %e, "could not persist selected device");
        }
    }

    /// Update a single binding both in memory and on disk for the currently
    /// selected device.
    ///
    /// Disk failures are logged at `warn` instead of bubbling up: the UI
    /// thread shouldn't crash because the user's home volume is full. A
    /// future retry / banner UI can read the most recent error from
    /// [`tracing`].
    pub fn commit_binding(&mut self, button: ButtonId, action: Action) {
        self.button_bindings.insert(button, action.clone());
        let Some(key) = self.current_record().map(|r| r.config_key.clone()) else {
            debug!(
                ?button,
                "no active device key — binding kept in memory only"
            );
            return;
        };
        self.config.set_binding(&key, button, action);
        if let Err(e) = self.config.save_atomic() {
            warn!(error = %e, "could not persist binding to config.toml");
        }
    }

    fn bindings_for_current(&self) -> BTreeMap<ButtonId, Action> {
        let stored = self
            .current_record()
            .map(|r| self.config.bindings_for(&r.config_key))
            .unwrap_or_default();
        let mut bindings: BTreeMap<ButtonId, Action> = ButtonId::ALL
            .iter()
            .copied()
            .map(|b| (b, default_binding(b)))
            .collect();
        for (k, v) in stored {
            bindings.insert(k, v);
        }
        bindings
    }
}

impl Global for AppState {}

fn build_device_list(inventories: &[DeviceInventory], cache: &AssetCache) -> Vec<DeviceRecord> {
    let mut list = Vec::new();
    for inv in inventories {
        let receiver_uid = inv.receiver.unique_id.clone();
        for paired in &inv.paired {
            let Some(model) = paired.model_info.as_ref() else {
                continue;
            };
            let config_key = model.config_key();
            let asset = cache.resolve(model);
            let display_name = asset
                .as_ref()
                .map(|a| a.display_name.clone())
                .or_else(|| paired.codename.clone())
                .unwrap_or_else(|| format!("Slot {}", paired.slot));
            let dpi_target = receiver_uid.as_ref().map(|uid| DpiTarget {
                receiver_uid: uid.clone(),
                slot: paired.slot,
            });
            list.push(DeviceRecord {
                config_key,
                display_name,
                asset,
                dpi_target,
            });
        }
    }
    list
}

fn pick_initial_device(list: &[DeviceRecord], saved: Option<&str>) -> usize {
    saved
        .and_then(|key| list.iter().position(|r| r.config_key == key))
        .unwrap_or(0)
}
