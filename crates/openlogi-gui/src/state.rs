//! App-wide UI state stored as a GPUI global.
//!
//! Anything that more than one view needs to read (current device, currently
//! armed button, the DPI value the panel and the dot-preview share) lives
//! here. Per-component scratch state (hover index, gesture point buffer) stays
//! in the owning entity.

#![allow(
    dead_code,
    reason = "fields are read once their owning component lands in UI.md phases 2–4"
)]

use std::collections::HashMap;

use gpui::Global;

use crate::data::mouse_buttons::{Action, ButtonId, default_binding};

/// Default DPI value applied to a fresh AppState. Matches a common Logitech
/// mid-range mouse and keeps the dot-preview visually obvious from frame one.
pub const DEFAULT_DPI: u32 = 1600;

pub struct AppState {
    /// Index into the carousel's device list. May briefly point past the end
    /// while devices are being enumerated; views must bounds-check.
    pub current_device: usize,
    /// The hotspot the user most recently armed by clicking. Drives the
    /// "selected button" outline on the mouse model and the popover content.
    pub active_button: Option<ButtonId>,
    pub button_bindings: HashMap<ButtonId, Action>,
    pub dpi: u32,
}

impl AppState {
    pub fn new() -> Self {
        let bindings = ButtonId::ALL
            .iter()
            .copied()
            .map(|b| (b, default_binding(b)))
            .collect();
        Self {
            current_device: 0,
            active_button: None,
            button_bindings: bindings,
            dpi: DEFAULT_DPI,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl Global for AppState {}
