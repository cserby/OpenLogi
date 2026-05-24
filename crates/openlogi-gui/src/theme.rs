//! Color, radius, and spacing constants used by the OpenLogi UI.
//!
//! Centralized here so phase work doesn't drift away from the Logi Options+
//! inspired palette. Anything pulled from `cx.theme()` still wins — these are
//! for the bespoke surfaces (mouse model, hotspot glows, gesture pad) that
//! aren't standard gpui-component widgets.

#![allow(
    dead_code,
    reason = "palette is wired up progressively across UI.md phases 1–7"
)]

use gpui::{Hsla, hsla, rgb};

/// Window background — slightly cooler than pure black to keep accents legible.
pub const BG_DARK: u32 = 0x001a_1a1d;

/// Card / panel base.
pub const SURFACE: u32 = 0x0022_2227;

/// Card hovered state.
pub const SURFACE_HOVER: u32 = 0x002c_2c33;

/// Subtle border between cards and surface.
pub const BORDER: u32 = 0x002f_2f36;

/// Primary action / selection blue.
pub const ACCENT_BLUE: u32 = 0x003b_82f6;

/// Foreground.
pub const TEXT_PRIMARY: u32 = 0x00e8_e8ec;

/// De-emphasized labels / metadata.
pub const TEXT_MUTED: u32 = 0x008a_8a93;

/// Status colors for the carousel connectivity dot.
pub const STATUS_CONNECTED: u32 = 0x0022_c55e;
pub const STATUS_CONNECTING: u32 = 0x00ea_b308;
pub const STATUS_OFFLINE: u32 = 0x006b_7280;

/// Sizes that several components need to agree on.
pub const HEADER_H: f32 = 80.;
pub const FOOTER_H: f32 = 50.;

/// Returns the canonical pulse-glow color around the accent hue.
///
/// `intensity` is the 0..1 amplitude factor — the shadow alpha is built from
/// it so callers can drive a breathing curve from `with_animation`.
#[inline]
pub fn accent_glow(intensity: f32) -> Hsla {
    hsla(0.6, 0.9, 0.6, 0.3 + intensity * 0.4)
}

/// Convenience wrapper so call sites don't have to import `rgb` separately.
#[inline]
pub fn color(hex: u32) -> impl Into<Hsla> {
    rgb(hex)
}
