//! DPI slider for the right-side config column.
//!
//! Just a label + numeric value + horizontal slider that writes to
//! [`AppState::dpi`]. The earlier "preview dot" was Phase 2 scaffolding
//! to validate `with_animation`; once the rest of the UI shipped it
//! added perpetual motion to a settings surface that should sit still.
//!
//! Wiring the DPI value to the hardware (HID++ `AdjustableDpi` feature
//! 0x2201) is a separate task — today the slider only mutates the in-
//! process [`AppState`], so other panels can react but the mouse itself
//! doesn't change DPI.

use gpui::{
    AppContext as _, BorrowAppContext as _, Context, Entity, IntoElement, ParentElement, Render,
    Styled, Subscription, Window, div, px, rgb,
};
use gpui_component::{
    ActiveTheme, h_flex,
    slider::{Slider, SliderEvent, SliderState},
    v_flex,
};
use tracing::{debug, warn};

use crate::state::AppState;
use crate::theme::ACCENT_BLUE;

/// Identifies which physical device the slider should write DPI to.
/// `receiver_uid` is the Bolt receiver's unique id (so we route writes
/// correctly when more than one receiver is plugged in); `slot` is the
/// device's pairing slot on that receiver.
#[derive(Debug, Clone)]
pub struct DpiTarget {
    pub receiver_uid: String,
    pub slot: u8,
}

/// Slider column width. Matches the right-column layout in `app.rs`.
const PANEL_W: f32 = 300.;

const MIN_DPI: f32 = 200.;
const MAX_DPI: f32 = 6400.;
const STEP_DPI: f32 = 50.;

pub struct DpiPanel {
    slider_state: Entity<SliderState>,
    _slider_sub: Subscription,
    _state_obs: Subscription,
}

impl DpiPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let initial_dpi = dpi_to_f32(
            cx.try_global::<AppState>()
                .map_or(crate::state::DEFAULT_DPI, |s| s.dpi),
        );

        // Order matters: `SliderState` defaults to max=100, and `.min(N)`
        // clamps the value against the current max. Setting max=6400
        // first keeps the intermediate state coherent.
        let slider_state = cx.new(|_| {
            SliderState::new()
                .max(MAX_DPI)
                .min(MIN_DPI)
                .step(STEP_DPI)
                .default_value(initial_dpi)
        });

        let slider_sub =
            cx.subscribe(
                &slider_state,
                |_panel, _slider, event: &SliderEvent, cx| match event {
                    // Continuous Change drives the in-process state so the
                    // numeric label tracks the drag. The HID write happens
                    // once on Release to keep us from spamming the device
                    // with intermediate values.
                    SliderEvent::Change(value) => {
                        let dpi = clamp_dpi(value.start());
                        debug!(dpi, "slider change → AppState.dpi");
                        cx.update_global::<AppState, _>(|state, _| state.dpi = dpi);
                        cx.notify();
                    }
                    SliderEvent::Release(value) => {
                        let dpi = clamp_dpi(value.start());
                        // Resolve the target from AppState at fire-time so
                        // carousel-driven device switches route the write to
                        // the now-current device, not whichever was active
                        // when the panel was constructed.
                        let target = cx
                            .try_global::<AppState>()
                            .and_then(|s| s.current_record().and_then(|r| r.dpi_target.clone()));
                        write_dpi_in_background(target, dpi);
                    }
                },
            );

        // Repaint when the carousel switches devices so the label tracks
        // the new device's last DPI (the slider thumb stays put — sliding
        // to a different value will write to the now-current device).
        let state_obs = cx.observe_global::<AppState>(|_panel, cx| cx.notify());

        Self {
            slider_state,
            _slider_sub: slider_sub,
            _state_obs: state_obs,
        }
    }
}

/// Spawn an OS thread that runs a one-shot tokio runtime, fires the
/// HID++ DPI write, and exits. We don't reuse GPUI's executor because
/// `async-hid` carries macOS-specific transport bits that want a tokio
/// reactor. One thread per slider release is cheap (~100 ms wall time)
/// and avoids a long-lived background runtime.
fn write_dpi_in_background(target: Option<DpiTarget>, dpi: u32) {
    let Some(target) = target else {
        debug!(dpi, "no target device — UI-only DPI update");
        return;
    };
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                warn!(error = %e, "tokio runtime init failed; DPI write skipped");
                return;
            }
        };
        // DPI is bounded by `clamp_dpi` to [200, 6400] so the u16 cast
        // is lossless.
        let dpi_u16 = u16::try_from(dpi).unwrap_or(u16::MAX);
        let result = rt.block_on(openlogi_hid::set_dpi(
            Some(&target.receiver_uid),
            target.slot,
            dpi_u16,
        ));
        match result {
            Ok(()) => debug!(slot = target.slot, dpi = dpi_u16, "DPI written to device"),
            Err(e) => warn!(error = ?e, "DPI write failed"),
        }
    });
}

impl Render for DpiPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dpi = cx
            .try_global::<AppState>()
            .map_or(crate::state::DEFAULT_DPI, |s| s.dpi);
        let theme = cx.theme();

        v_flex()
            .gap_3()
            .w(px(PANEL_W))
            .child(
                h_flex()
                    .justify_between()
                    .items_baseline()
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child("DPI"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(ACCENT_BLUE))
                            .child(format!("{dpi}")),
                    ),
            )
            .child(Slider::new(&self.slider_state).horizontal())
    }
}

/// Snap a raw slider read to the discrete DPI step and clamp into range.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "value is rounded and clamped into [MIN_DPI, MAX_DPI] above the cast"
)]
fn clamp_dpi(raw: f32) -> u32 {
    raw.clamp(MIN_DPI, MAX_DPI).round() as u32
}

/// Widen a DPI count into f32 for slider math. DPI is ≤ 6400 so it fits
/// comfortably in f32's mantissa with no precision loss.
#[allow(
    clippy::cast_precision_loss,
    reason = "DPI ≤ 6400 — well below f32 mantissa precision"
)]
fn dpi_to_f32(dpi: u32) -> f32 {
    dpi as f32
}
