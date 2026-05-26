//! Logical mouse button identifiers and the action vocabulary each one can
//! bind to. Lives in `openlogi-core` because the [`config`](crate::config)
//! schema serializes these directly — the GUI re-exports them.
//!
//! When [`Action`] gains new variants, keep the existing variant names stable:
//! the TOML config keys/values use the enum variant identifiers verbatim, so
//! renames are migration events.

use std::fmt;

use serde::{Deserialize, Serialize};

/// One of the user-rebindable hotspots on a Logi mouse. The order matches the
/// physical layout from front to side; [`ButtonId::ALL`] is consumed by the
/// default-binding generator and the popover trigger list.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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

    /// Human-readable label for popovers and tooltips.
    #[must_use]
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

/// Grouping for popover section headers.
///
/// Used by [`Action::category`] and rendered as a small muted label above
/// each group in the action picker.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    /// Cut, copy, paste, undo, redo, select-all, find, save.
    Editing,
    /// Browser navigation: tabs, page reload, back/forward.
    Browser,
    /// Playback and volume controls.
    Media,
    /// Physical mouse clicks.
    Mouse,
    /// DPI cycle and SmartShift.
    Dpi,
    /// Scroll direction shortcuts.
    Scroll,
    /// Window/app navigation: Mission Control, Launchpad, etc.
    Navigation,
    /// Lock screen, show desktop, system-level actions.
    System,
}

impl Category {
    /// Short label for popover section headers (already uppercase so callers
    /// don't have to transform it).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Category::Editing => "EDITING",
            Category::Browser => "BROWSER",
            Category::Media => "MEDIA",
            Category::Mouse => "MOUSE",
            Category::Dpi => "DPI",
            Category::Scroll => "SCROLL",
            Category::Navigation => "NAVIGATION",
            Category::System => "SYSTEM",
        }
    }
}

/// What pressing a [`ButtonId`] should do.
///
/// Serialization uses serde's default external tagging: unit variants
/// serialize as a bare string (`"BrowserBack"`) and the tuple variant
/// serializes as a single-key table (`{ CustomShortcut = "my chord" }`).
///
/// **Stability contract:** existing variant *names* are frozen — they form the
/// on-disk `config.toml` schema. New variants may be appended freely; removing
/// or renaming a variant requires a `schema_version` bump and a migration.
///
/// `Action::execute` synthesizes the OS-level event for each variant.
/// On macOS it posts the event via `CGEventPost(kCGHIDEventTap, …)`.
/// On other platforms it logs a warning and returns immediately — the binary
/// compiles on all targets.
///
/// # Manual verification
///
/// `execute` is intentionally excluded from the automated test suite because
/// it would need to intercept the OS event queue. Smoke-test it manually:
/// bind a button to any action in the GUI and confirm the expected system event
/// fires when the button is pressed.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // ── Mouse ────────────────────────────────────────────────────────────────
    /// Primary mouse button.
    LeftClick,
    /// Secondary mouse button.
    RightClick,
    /// Middle mouse button (wheel click).
    MiddleClick,

    // ── Editing ──────────────────────────────────────────────────────────────
    /// Copy the current selection (⌘C / Ctrl+C).
    Copy,
    /// Paste from the clipboard (⌘V / Ctrl+V).
    Paste,
    /// Cut the current selection (⌘X / Ctrl+X).
    Cut,
    /// Undo the last action (⌘Z / Ctrl+Z).
    Undo,
    /// Redo the last undone action (⌘⇧Z / Ctrl+Y).
    Redo,
    /// Select all content (⌘A / Ctrl+A).
    SelectAll,
    /// Open the find / search bar (⌘F / Ctrl+F).
    Find,
    /// Save the current document (⌘S / Ctrl+S).
    Save,

    // ── Browser / Navigation ──────────────────────────────────────────────────
    /// Navigate backward in browser history.
    BrowserBack,
    /// Navigate forward in browser history.
    BrowserForward,
    /// Open a new tab (⌘T / Ctrl+T).
    NewTab,
    /// Close the current tab (⌘W / Ctrl+W).
    CloseTab,
    /// Reopen the last closed tab (⌘⇧T / Ctrl+Shift+T).
    ReopenTab,
    /// Switch to the next tab (⌃⇥ / Ctrl+Tab).
    NextTab,
    /// Switch to the previous tab (⌃⇧⇥ / Ctrl+Shift+Tab).
    PrevTab,
    /// Reload the current page (⌘R / Ctrl+R).
    ReloadPage,

    // ── Navigation / Window ───────────────────────────────────────────────────
    /// macOS Mission Control (⌃↑).
    MissionControl,
    /// macOS App Exposé — all windows for the current app (⌃↓).
    AppExpose,
    /// Show the desktop (hide all windows).
    ShowDesktop,
    /// Open Launchpad.
    LaunchpadShow,

    // ── System ────────────────────────────────────────────────────────────────
    /// Lock the screen.
    LockScreen,
    /// Capture a screenshot.
    Screenshot,

    // ── Media ────────────────────────────────────────────────────────────────
    /// Toggle media play/pause.
    PlayPause,
    /// Skip to the next track.
    NextTrack,
    /// Go back to the previous track.
    PrevTrack,
    /// Increase system volume.
    VolumeUp,
    /// Decrease system volume.
    VolumeDown,
    /// Toggle system mute.
    MuteVolume,

    // ── DPI ──────────────────────────────────────────────────────────────────
    /// Step through the configured DPI preset list (P1.7).
    CycleDpiPresets,
    /// Toggle the HID++ SmartShift ratchet/free-spin wheel mode (P1.1).
    ToggleSmartShift,

    // ── Scroll ───────────────────────────────────────────────────────────────
    /// Synthesise a vertical scroll-up tick.
    ScrollUp,
    /// Synthesise a vertical scroll-down tick.
    ScrollDown,
    /// Synthesise a horizontal scroll-left tick.
    HorizontalScrollLeft,
    /// Synthesise a horizontal scroll-right tick.
    HorizontalScrollRight,

    // ── Custom ───────────────────────────────────────────────────────────────
    /// Replay an arbitrary recorded key chord (P1.3).
    ///
    /// The `String` value is a human-readable label, e.g. `"⌘⇧P"`. The
    /// actual key data for `execute` is stored separately in the config once
    /// P1.3 lands; for now this is a placeholder that renders the label.
    CustomShortcut(String),
}

impl Action {
    /// Display label for the popover row.
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Action::LeftClick => "Left Click",
            Action::RightClick => "Right Click",
            Action::MiddleClick => "Middle Click",
            Action::Copy => "Copy",
            Action::Paste => "Paste",
            Action::Cut => "Cut",
            Action::Undo => "Undo",
            Action::Redo => "Redo",
            Action::SelectAll => "Select All",
            Action::Find => "Find",
            Action::Save => "Save",
            Action::BrowserBack => "Browser Back",
            Action::BrowserForward => "Browser Forward",
            Action::NewTab => "New Tab",
            Action::CloseTab => "Close Tab",
            Action::ReopenTab => "Reopen Tab",
            Action::NextTab => "Next Tab",
            Action::PrevTab => "Previous Tab",
            Action::ReloadPage => "Reload Page",
            Action::MissionControl => "Mission Control",
            Action::AppExpose => "App Exposé",
            Action::ShowDesktop => "Show Desktop",
            Action::LaunchpadShow => "Launchpad",
            Action::LockScreen => "Lock Screen",
            Action::Screenshot => "Screenshot",
            Action::PlayPause => "Play / Pause",
            Action::NextTrack => "Next Track",
            Action::PrevTrack => "Previous Track",
            Action::VolumeUp => "Volume Up",
            Action::VolumeDown => "Volume Down",
            Action::MuteVolume => "Mute",
            Action::CycleDpiPresets => "Cycle DPI Presets",
            Action::ToggleSmartShift => "Toggle SmartShift",
            Action::ScrollUp => "Scroll Up",
            Action::ScrollDown => "Scroll Down",
            Action::HorizontalScrollLeft => "Scroll Left",
            Action::HorizontalScrollRight => "Scroll Right",
            Action::CustomShortcut(s) => s.as_str(),
        }
    }

    /// Which [`Category`] this action belongs to, used for popover grouping.
    #[must_use]
    pub fn category(&self) -> Category {
        match self {
            Action::LeftClick | Action::RightClick | Action::MiddleClick => Category::Mouse,
            // CustomShortcut is assigned to Editing so it doesn't need a
            // separate arm (it's not in the picker catalog).
            Action::Copy
            | Action::Paste
            | Action::Cut
            | Action::Undo
            | Action::Redo
            | Action::SelectAll
            | Action::Find
            | Action::Save
            | Action::CustomShortcut(_) => Category::Editing,
            Action::BrowserBack
            | Action::BrowserForward
            | Action::NewTab
            | Action::CloseTab
            | Action::ReopenTab
            | Action::NextTab
            | Action::PrevTab
            | Action::ReloadPage => Category::Browser,
            Action::MissionControl
            | Action::AppExpose
            | Action::ShowDesktop
            | Action::LaunchpadShow => Category::Navigation,
            Action::LockScreen | Action::Screenshot => Category::System,
            Action::PlayPause
            | Action::NextTrack
            | Action::PrevTrack
            | Action::VolumeUp
            | Action::VolumeDown
            | Action::MuteVolume => Category::Media,
            Action::CycleDpiPresets | Action::ToggleSmartShift => Category::Dpi,
            Action::ScrollUp
            | Action::ScrollDown
            | Action::HorizontalScrollLeft
            | Action::HorizontalScrollRight => Category::Scroll,
        }
    }

    /// All pickable actions in a deterministic order.
    ///
    /// [`Action::CustomShortcut`] is intentionally excluded — it is opened via
    /// "Record shortcut…" (P1.3), not selected from the catalog.
    #[must_use]
    pub fn catalog() -> Vec<Action> {
        vec![
            // Mouse
            Action::LeftClick,
            Action::RightClick,
            Action::MiddleClick,
            // Editing
            Action::Copy,
            Action::Paste,
            Action::Cut,
            Action::Undo,
            Action::Redo,
            Action::SelectAll,
            Action::Find,
            Action::Save,
            // Browser
            Action::BrowserBack,
            Action::BrowserForward,
            Action::NewTab,
            Action::CloseTab,
            Action::ReopenTab,
            Action::NextTab,
            Action::PrevTab,
            Action::ReloadPage,
            // Navigation
            Action::MissionControl,
            Action::AppExpose,
            Action::ShowDesktop,
            Action::LaunchpadShow,
            // System
            Action::LockScreen,
            Action::Screenshot,
            // Media
            Action::PlayPause,
            Action::NextTrack,
            Action::PrevTrack,
            Action::VolumeUp,
            Action::VolumeDown,
            Action::MuteVolume,
            // DPI
            Action::CycleDpiPresets,
            Action::ToggleSmartShift,
            // Scroll
            Action::ScrollUp,
            Action::ScrollDown,
            Action::HorizontalScrollLeft,
            Action::HorizontalScrollRight,
        ]
    }

    /// Synthesise the OS-level event for this action.
    ///
    /// On macOS, key events are posted via `CGEventPost(kCGHIDEventTap, …)`
    /// using virtual key codes from the standard US keyboard layout.
    /// Mouse-click variants and actions with no direct CGEvent equivalent
    /// (e.g. `CycleDpiPresets`, `ToggleSmartShift`) are handled at the hook
    /// layer (P0.1) and log a debug trace here instead.
    ///
    /// On other platforms a warning is logged and the function returns
    /// immediately — the binary compiles clean on all targets.
    pub fn execute(&self) {
        #[cfg(target_os = "macos")]
        self.execute_macos();

        #[cfg(not(target_os = "macos"))]
        {
            tracing::warn!(
                action = self.label(),
                "Action::execute unsupported on this platform"
            );
        }
    }

    /// macOS implementation: dispatch to the appropriate event helper.
    #[cfg(target_os = "macos")]
    fn execute_macos(&self) {
        use core_graphics::event::CGEventFlags;

        // Modifier bit shorthands.
        let cmd = CGEventFlags::CGEventFlagCommand;
        let shift = CGEventFlags::CGEventFlagShift;
        let ctrl = CGEventFlags::CGEventFlagControl;
        let none = CGEventFlags::CGEventFlagNull;

        match self {
            // ── Mouse clicks: delegated to the hook layer ─────────────────────
            Action::LeftClick | Action::RightClick | Action::MiddleClick => {
                tracing::debug!(
                    action = self.label(),
                    "mouse-click execute delegated to hook layer"
                );
            }
            // ── Editing ───────────────────────────────────────────────────────
            Action::Copy => macos::post_key(VK_C, cmd),
            Action::Paste => macos::post_key(VK_V, cmd),
            Action::Cut => macos::post_key(VK_X, cmd),
            Action::Undo => macos::post_key(VK_Z, cmd),
            Action::Redo => macos::post_key(VK_Z, cmd | shift),
            Action::SelectAll => macos::post_key(VK_A, cmd),
            Action::Find => macos::post_key(VK_F, cmd),
            Action::Save => macos::post_key(VK_S, cmd),
            // ── Browser / Navigation ──────────────────────────────────────────
            // BrowserBack/Forward: Cmd+[ / Cmd+] as keyboard fallback; hook
            // layer handles the physical mouse buttons directly.
            // kVK_ANSI_LeftBracket = 0x21, kVK_ANSI_RightBracket = 0x1E
            Action::BrowserBack => macos::post_key(0x21, cmd),
            Action::BrowserForward => macos::post_key(0x1E, cmd),
            Action::NewTab => macos::post_key(VK_T, cmd),
            Action::CloseTab => macos::post_key(VK_W, cmd),
            Action::ReopenTab => macos::post_key(VK_T, cmd | shift),
            Action::NextTab => macos::post_key(VK_TAB, ctrl),
            Action::PrevTab => macos::post_key(VK_TAB, ctrl | shift),
            Action::ReloadPage => macos::post_key(VK_R, cmd),
            // ── Navigation / Window ───────────────────────────────────────────
            // Mission Control = Ctrl+Up (kVK_UpArrow = 0x7E)
            Action::MissionControl => macos::post_key(0x7E, ctrl),
            // App Exposé = Ctrl+Down (kVK_DownArrow = 0x7D)
            Action::AppExpose => macos::post_key(0x7D, ctrl),
            // Show Desktop = Cmd+F3 (kVK_F3 = 0x63)
            Action::ShowDesktop => macos::post_key(0x63, cmd),
            // Launchpad = F4 (kVK_F4 = 0x76)
            Action::LaunchpadShow => macos::post_key(0x76, none),
            // ── System ────────────────────────────────────────────────────────
            // Lock screen = Cmd+Ctrl+Q (kVK_ANSI_Q = 0x0C)
            Action::LockScreen => macos::post_key(0x0C, cmd | ctrl),
            // Screenshot = Cmd+Shift+3 (kVK_ANSI_3 = 0x14)
            Action::Screenshot => macos::post_key(0x14, cmd | shift),
            // ── Media ─────────────────────────────────────────────────────────
            // NX_KEYTYPE_PLAY=16, NEXT=17, PREVIOUS=18 via NSSystemDefined stub.
            Action::PlayPause => macos::post_media_key(0),
            Action::NextTrack => macos::post_media_key(1),
            Action::PrevTrack => macos::post_media_key(2),
            // kVK_VolumeUp/Down/Mute = 0x48/0x49/0x4A (ADB codes)
            Action::VolumeUp => macos::post_key(0x48, none),
            Action::VolumeDown => macos::post_key(0x49, none),
            Action::MuteVolume => macos::post_key(0x4A, none),
            // ── DPI / SmartShift: handled at hook/HID layer ───────────────────
            Action::CycleDpiPresets | Action::ToggleSmartShift => {
                tracing::debug!(
                    action = self.label(),
                    "device action handled by hook/HID layer"
                );
            }
            // ── Scroll ────────────────────────────────────────────────────────
            Action::ScrollUp
            | Action::ScrollDown
            | Action::HorizontalScrollLeft
            | Action::HorizontalScrollRight => macos::post_scroll(self),
            // ── Custom ────────────────────────────────────────────────────────
            Action::CustomShortcut(s) => {
                tracing::warn!(
                    chord = s.as_str(),
                    "CustomShortcut::execute not yet implemented (P1.3)"
                );
            }
        }
    }
}

// ── macOS virtual key codes ────────────────────────────────────────────────
// Source: <HIToolbox/Events.h> kVK_* constants. Values are layout-independent
// for the US ANSI keyboard.
#[cfg(target_os = "macos")]
const VK_A: u16 = 0x00;
#[cfg(target_os = "macos")]
const VK_C: u16 = 0x08;
#[cfg(target_os = "macos")]
const VK_F: u16 = 0x03;
#[cfg(target_os = "macos")]
const VK_R: u16 = 0x0F;
#[cfg(target_os = "macos")]
const VK_S: u16 = 0x01;
#[cfg(target_os = "macos")]
const VK_T: u16 = 0x11;
#[cfg(target_os = "macos")]
const VK_V: u16 = 0x09;
#[cfg(target_os = "macos")]
const VK_W: u16 = 0x0D;
#[cfg(target_os = "macos")]
const VK_X: u16 = 0x07;
#[cfg(target_os = "macos")]
const VK_Z: u16 = 0x06;
#[cfg(target_os = "macos")]
const VK_TAB: u16 = 0x30;

/// Platform helpers for synthesising OS-level input events on macOS.
#[cfg(target_os = "macos")]
mod macos {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, ScrollEventUnit};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    use crate::binding::Action;

    /// Post a key-down + key-up pair for `vk` with `flags` set.
    pub(super) fn post_key(vk: u16, flags: CGEventFlags) {
        let Ok(src) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) else {
            tracing::warn!("CGEventSource::new failed");
            return;
        };
        let Ok(down) = CGEvent::new_keyboard_event(src.clone(), vk, true) else {
            tracing::warn!("CGEvent::new_keyboard_event(down) failed");
            return;
        };
        down.set_flags(flags);
        down.post(CGEventTapLocation::HID);
        let Ok(up) = CGEvent::new_keyboard_event(src, vk, false) else {
            tracing::warn!("CGEvent::new_keyboard_event(up) failed");
            return;
        };
        up.set_flags(flags);
        up.post(CGEventTapLocation::HID);
    }

    /// Post a media key event (Play/Pause, Next, Previous).
    ///
    /// `kind`: 0 = play/pause, 1 = next track, 2 = previous track.
    ///
    /// The proper implementation uses an `NSSystemDefined` event (type 14,
    /// subtype 8) which requires AppKit bindings. Until those land this
    /// function logs a debug trace so manual smoke tests can confirm the
    /// correct execution path.
    pub(super) fn post_media_key(kind: i32) {
        // NX_KEYTYPE_PLAY=16, NX_KEYTYPE_NEXT=17, NX_KEYTYPE_PREVIOUS=18.
        let nx_key: i64 = match kind {
            0 => 16,
            1 => 17,
            _ => 18,
        };
        tracing::debug!(
            nx_key,
            "media key event: NSSystemDefined stub — full AppKit impl tracked in P1.x"
        );
    }

    /// Post a synthetic scroll event for `action` (one of the `Scroll*` variants).
    pub(super) fn post_scroll(action: &Action) {
        let Ok(src) = CGEventSource::new(CGEventSourceStateID::HIDSystemState) else {
            tracing::warn!("CGEventSource::new failed for scroll");
            return;
        };
        let (v, h): (i32, i32) = match action {
            Action::ScrollUp => (3, 0),
            Action::ScrollDown => (-3, 0),
            Action::HorizontalScrollLeft => (0, -3),
            Action::HorizontalScrollRight => (0, 3),
            _ => return,
        };
        let Ok(ev) = CGEvent::new_scroll_event(src, ScrollEventUnit::PIXEL, 2, v, h, 0) else {
            tracing::warn!("CGEvent::new_scroll_event failed");
            return;
        };
        ev.post(CGEventTapLocation::HID);
    }
}

/// Sensible defaults for a fresh device so the panel isn't empty on first run.
#[must_use]
pub fn default_binding(button: ButtonId) -> Action {
    match button {
        ButtonId::LeftClick => Action::LeftClick,
        ButtonId::RightClick => Action::RightClick,
        ButtonId::MiddleClick => Action::MiddleClick,
        ButtonId::Back => Action::BrowserBack,
        ButtonId::Forward => Action::BrowserForward,
        ButtonId::DpiToggle => Action::CycleDpiPresets,
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, reason = "expect/unwrap are idiomatic in tests")]
mod tests {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};

    use super::*;

    // ── Roundtrip wrapper: defined here so it precedes any `let` statements ──

    /// Minimal TOML-serializable wrapper used by `roundtrip`.
    /// Defined at module scope to satisfy `clippy::items_after_statements`.
    #[derive(Serialize, Deserialize)]
    struct RoundtripWrapper {
        binding: BTreeMap<ButtonId, Action>,
    }

    // ── Catalog tests ─────────────────────────────────────────────────────────

    #[test]
    fn catalog_has_at_least_29_entries() {
        let catalog = Action::catalog();
        assert!(
            catalog.len() >= 29,
            "catalog has {} entries, need ≥ 29",
            catalog.len()
        );
    }

    #[test]
    fn catalog_excludes_custom_shortcut() {
        let catalog = Action::catalog();
        for action in &catalog {
            assert!(
                !matches!(action, Action::CustomShortcut(_)),
                "catalog must not contain CustomShortcut"
            );
        }
    }

    // ── TOML roundtrip ────────────────────────────────────────────────────────

    /// Serialize then deserialize `action` through TOML, using a wrapper
    /// struct because TOML requires a top-level table.
    fn roundtrip(action: &Action) -> Action {
        let mut map: BTreeMap<ButtonId, Action> = BTreeMap::new();
        map.insert(ButtonId::Back, action.clone());
        let w = RoundtripWrapper { binding: map };
        let s = toml::to_string(&w).expect("serialize");
        let back: RoundtripWrapper = toml::from_str(&s).expect("deserialize");
        back.binding
            .into_values()
            .next()
            .expect("binding present after roundtrip")
    }

    #[test]
    fn all_catalog_variants_roundtrip_toml() {
        for action in Action::catalog() {
            let back = roundtrip(&action);
            assert_eq!(action, back, "TOML roundtrip failed for {action:?}");
        }
    }

    #[test]
    fn custom_shortcut_roundtrips_toml() {
        let action = Action::CustomShortcut("⌘⇧P".into());
        assert_eq!(roundtrip(&action), action);
    }

    // ── Category tests ────────────────────────────────────────────────────────

    #[test]
    fn category_editing_variants() {
        assert_eq!(Action::Copy.category(), Category::Editing);
        assert_eq!(Action::Undo.category(), Category::Editing);
        assert_eq!(Action::SelectAll.category(), Category::Editing);
        assert_eq!(Action::Find.category(), Category::Editing);
        assert_eq!(Action::Save.category(), Category::Editing);
        assert_eq!(Action::Cut.category(), Category::Editing);
        assert_eq!(Action::Redo.category(), Category::Editing);
        assert_eq!(Action::Paste.category(), Category::Editing);
    }

    #[test]
    fn category_browser_variants() {
        assert_eq!(Action::BrowserBack.category(), Category::Browser);
        assert_eq!(Action::BrowserForward.category(), Category::Browser);
        assert_eq!(Action::NewTab.category(), Category::Browser);
        assert_eq!(Action::CloseTab.category(), Category::Browser);
        assert_eq!(Action::ReopenTab.category(), Category::Browser);
        assert_eq!(Action::NextTab.category(), Category::Browser);
        assert_eq!(Action::PrevTab.category(), Category::Browser);
        assert_eq!(Action::ReloadPage.category(), Category::Browser);
    }

    #[test]
    fn category_media_variants() {
        assert_eq!(Action::PlayPause.category(), Category::Media);
        assert_eq!(Action::NextTrack.category(), Category::Media);
        assert_eq!(Action::PrevTrack.category(), Category::Media);
        assert_eq!(Action::VolumeUp.category(), Category::Media);
        assert_eq!(Action::VolumeDown.category(), Category::Media);
        assert_eq!(Action::MuteVolume.category(), Category::Media);
    }

    #[test]
    fn category_mouse_variants() {
        assert_eq!(Action::LeftClick.category(), Category::Mouse);
        assert_eq!(Action::RightClick.category(), Category::Mouse);
        assert_eq!(Action::MiddleClick.category(), Category::Mouse);
    }

    #[test]
    fn category_dpi_variants() {
        assert_eq!(Action::CycleDpiPresets.category(), Category::Dpi);
        assert_eq!(Action::ToggleSmartShift.category(), Category::Dpi);
    }

    #[test]
    fn category_scroll_variants() {
        assert_eq!(Action::ScrollUp.category(), Category::Scroll);
        assert_eq!(Action::ScrollDown.category(), Category::Scroll);
        assert_eq!(Action::HorizontalScrollLeft.category(), Category::Scroll);
        assert_eq!(Action::HorizontalScrollRight.category(), Category::Scroll);
    }

    #[test]
    fn category_navigation_variants() {
        assert_eq!(Action::MissionControl.category(), Category::Navigation);
        assert_eq!(Action::AppExpose.category(), Category::Navigation);
        assert_eq!(Action::ShowDesktop.category(), Category::Navigation);
        assert_eq!(Action::LaunchpadShow.category(), Category::Navigation);
    }

    #[test]
    fn category_system_variants() {
        assert_eq!(Action::LockScreen.category(), Category::System);
        assert_eq!(Action::Screenshot.category(), Category::System);
    }

    // ── Category label smoke test ─────────────────────────────────────────────

    #[test]
    fn category_labels_are_nonempty() {
        let categories = [
            Category::Editing,
            Category::Browser,
            Category::Media,
            Category::Mouse,
            Category::Dpi,
            Category::Scroll,
            Category::Navigation,
            Category::System,
        ];
        for cat in categories {
            assert!(!cat.label().is_empty(), "label empty for {cat:?}");
        }
    }

    // ── Default binding ───────────────────────────────────────────────────────

    #[test]
    fn dpi_toggle_default_is_cycle_dpi_presets() {
        assert_eq!(
            default_binding(ButtonId::DpiToggle),
            Action::CycleDpiPresets
        );
    }
}
