//! Logical mouse buttons + the action vocabulary each one can bind to.
//!
//! Hotspot bounds are authored in mouse-model-local pixels (the SVG canvas is
//! 420×560 — see [`MOUSE_MODEL_SIZE`]). They are stored as plain `f32` tuples
//! so this module stays purely data and doesn't drag in `gpui` types.

#![allow(
    dead_code,
    reason = "scaffolding consumed by UI.md phases 3–6 (carousel, popover, hotspots)"
)]

use std::fmt;

/// The size of the mouse model canvas. Hotspot coords are relative to this.
pub const MOUSE_MODEL_SIZE: (f32, f32) = (420., 560.);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonId {
    LeftClick,
    RightClick,
    MiddleClick,
    Back,
    Forward,
    DpiToggle,
}

impl ButtonId {
    pub const ALL: [ButtonId; 6] = [
        ButtonId::LeftClick,
        ButtonId::RightClick,
        ButtonId::MiddleClick,
        ButtonId::Back,
        ButtonId::Forward,
        ButtonId::DpiToggle,
    ];

    pub fn label(self) -> &'static str {
        match self {
            ButtonId::LeftClick => "Left Click",
            ButtonId::RightClick => "Right Click",
            ButtonId::MiddleClick => "Middle Click",
            ButtonId::Back => "Back",
            ButtonId::Forward => "Forward",
            ButtonId::DpiToggle => "DPI Toggle",
        }
    }
}

impl fmt::Display for ButtonId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Hotspot rectangle in mouse-model-local coordinates.
#[derive(Clone, Copy, Debug)]
pub struct Hotspot {
    pub id: ButtonId,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Hotspot {
    /// Returns the center point — convenient for leader lines.
    #[inline]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w * 0.5, self.y + self.h * 0.5)
    }
}

/// Default hotspot layout. Coordinates approximate a simple top-down mouse
/// shape and will be tuned against the real SVG in Phase 6.
pub fn default_hotspots() -> Vec<Hotspot> {
    vec![
        Hotspot {
            id: ButtonId::LeftClick,
            x: 40.,
            y: 60.,
            w: 160.,
            h: 200.,
        },
        Hotspot {
            id: ButtonId::RightClick,
            x: 220.,
            y: 60.,
            w: 160.,
            h: 200.,
        },
        Hotspot {
            id: ButtonId::MiddleClick,
            x: 180.,
            y: 110.,
            w: 60.,
            h: 90.,
        },
        Hotspot {
            id: ButtonId::Back,
            x: 0.,
            y: 220.,
            w: 40.,
            h: 60.,
        },
        Hotspot {
            id: ButtonId::Forward,
            x: 0.,
            y: 290.,
            w: 40.,
            h: 60.,
        },
        Hotspot {
            id: ButtonId::DpiToggle,
            x: 175.,
            y: 230.,
            w: 70.,
            h: 40.,
        },
    ]
}

/// An action a button can be bound to. Kept open-ended so Phase 4 can grow
/// new variants (keyboard chords, system commands, app-specific macros).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    LeftClick,
    RightClick,
    MiddleClick,
    Copy,
    Paste,
    Screenshot,
    BrowserBack,
    BrowserForward,
    CustomShortcut(String),
}

impl Action {
    pub fn label(&self) -> &str {
        match self {
            Action::LeftClick => "Left Click",
            Action::RightClick => "Right Click",
            Action::MiddleClick => "Middle Click",
            Action::Copy => "Copy",
            Action::Paste => "Paste",
            Action::Screenshot => "Screenshot",
            Action::BrowserBack => "Browser Back",
            Action::BrowserForward => "Browser Forward",
            Action::CustomShortcut(s) => s.as_str(),
        }
    }

    /// The picker list shown inside the action popover.
    pub fn catalog() -> Vec<Action> {
        vec![
            Action::LeftClick,
            Action::RightClick,
            Action::MiddleClick,
            Action::Copy,
            Action::Paste,
            Action::Screenshot,
            Action::BrowserBack,
            Action::BrowserForward,
        ]
    }
}

/// Sensible defaults for a fresh device so the panel isn't empty on first run.
pub fn default_binding(button: ButtonId) -> Action {
    match button {
        ButtonId::LeftClick => Action::LeftClick,
        ButtonId::RightClick => Action::RightClick,
        ButtonId::MiddleClick => Action::MiddleClick,
        ButtonId::Back => Action::BrowserBack,
        ButtonId::Forward => Action::BrowserForward,
        ButtonId::DpiToggle => Action::CustomShortcut("Toggle DPI".into()),
    }
}
