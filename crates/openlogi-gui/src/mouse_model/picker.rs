//! Popover content for binding a [`ButtonId`] to an [`Action`].
//!
//! Generic over the entity that should be notified after the binding changes
//! — that lets both the Phase 4 row and the Phase 6 mouse model reuse the
//! same picker.

use std::rc::Rc;

use gpui::{
    AnyElement, BorrowAppContext as _, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, StatefulInteractiveElement as _, Styled, div, px, rgb,
};
use gpui_component::{popover::PopoverState, v_flex};

const POPOVER_W: f32 = 200.;
/// Cap the scrollable action list at ~11 rows tall. The catalog has 29+
/// entries plus section headers, so the full list would otherwise blow
/// past the window height. Header + footer-less list stays sticky.
const POPOVER_LIST_MAX_H: f32 = 360.;

use crate::data::mouse_buttons::{
    Action, ButtonId, Category, GestureDirection, default_gesture_binding,
};
use crate::state::AppState;
use crate::theme::{ACCENT_BLUE, SURFACE, SURFACE_HOVER, TEXT_MUTED, TEXT_PRIMARY};

/// Build the popover body that lets the user re-bind `btn`.
///
/// `observer` is whatever entity wraps the trigger — it'll be notified after
/// the global is updated so the trigger re-renders with the new label.
///
/// Actions are grouped by [`Category`] with a small muted section header
/// above each group.
pub fn action_picker<T: 'static>(
    btn: ButtonId,
    observer: &Entity<T>,
    cx: &mut Context<PopoverState>,
) -> AnyElement {
    let popover = cx.entity().downgrade();
    let current = cx
        .try_global::<AppState>()
        .and_then(|s| s.button_bindings.get(&btn).cloned());

    // Group the catalog by category while preserving catalog order within
    // each group.  We collect (category, items) in first-seen category order
    // so the sections appear in the same order the catalog defines them.
    let catalog = Action::catalog();
    let mut sections: Vec<(Category, Vec<Action>)> = Vec::new();
    for action in catalog {
        let cat = action.category();
        if let Some(sec) = sections.iter_mut().find(|(c, _)| *c == cat) {
            sec.1.push(action);
        } else {
            sections.push((cat, vec![action]));
        }
    }

    // Global item index for stable GPUI element IDs across sections.
    let mut item_idx: usize = 0;
    let mut children: Vec<AnyElement> = Vec::new();

    for (category, actions) in sections {
        // Section header — small muted all-caps label.
        children.push(
            div()
                .w_full()
                .px_2()
                .pt_2()
                .pb_1()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .child(category.label())
                .into_any_element(),
        );

        for action in actions {
            let is_selected = current.as_ref() == Some(&action);
            let label = action.label();
            let observer = observer.clone();
            let popover = popover.clone();
            let action = Rc::new(action);
            let idx = item_idx;
            item_idx += 1;

            children.push(
                div()
                    .id(("action-item", idx))
                    .w_full()
                    .px_3()
                    .py_1p5()
                    .rounded_md()
                    .text_sm()
                    .text_color(rgb(if is_selected {
                        ACCENT_BLUE
                    } else {
                        TEXT_PRIMARY
                    }))
                    .bg(rgb(if is_selected { SURFACE_HOVER } else { SURFACE }))
                    .hover(|s| s.bg(rgb(SURFACE_HOVER)))
                    .child(label)
                    .on_click(move |_event, window, cx| {
                        let action = (*action).clone();
                        cx.update_global::<AppState, _>(|state, _| {
                            state.commit_binding(btn, action);
                        });
                        observer.update(cx, |_, cx| cx.notify());
                        if let Some(p) = popover.upgrade() {
                            p.update(cx, |s, cx| s.dismiss(window, cx));
                        }
                    })
                    .into_any_element(),
            );
        }
    }

    v_flex()
        .min_w(px(POPOVER_W))
        .gap_1()
        .p_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .px_2()
                .pb_1()
                .child(format!("Bind {}", btn.label())),
        )
        // Action list scrolls. The catalog is long enough (29+ actions
        // across half a dozen categories) that an unconstrained popover
        // overflows the window; capping height + scroll keeps the
        // sticky-header pattern that's already familiar to the user.
        .child(
            div()
                .id("picker-scroll")
                .max_h(px(POPOVER_LIST_MAX_H))
                .overflow_y_scroll()
                .children(children),
        )
        .into_any_element()
}

/// Top-level popover content for the gesture button. Two pages, gated on
/// [`AppState::gesture_edit`]:
///
/// 1. `None` → directions list ([`gesture_directions_list`]).
/// 2. `Some(dir)` → action picker for that one direction
///    ([`gesture_action_picker`]). Committing returns to the list.
pub fn gesture_picker<T: 'static>(
    observer: &Entity<T>,
    cx: &mut Context<PopoverState>,
) -> AnyElement {
    let edit = cx
        .try_global::<AppState>()
        .and_then(|s| s.gesture_edit);
    match edit {
        Some(dir) => gesture_action_picker(dir, observer, cx),
        None => gesture_directions_list(observer, cx),
    }
}

/// Five-row page: each direction shows its current binding, click to edit.
fn gesture_directions_list<T: 'static>(
    observer: &Entity<T>,
    cx: &mut Context<PopoverState>,
) -> AnyElement {
    let bindings = cx
        .try_global::<AppState>()
        .map(|s| s.gesture_bindings.clone())
        .unwrap_or_default();

    let rows: Vec<AnyElement> = GestureDirection::ALL
        .iter()
        .copied()
        .enumerate()
        .map(|(idx, dir)| {
            let action = bindings
                .get(&dir)
                .cloned()
                .unwrap_or_else(|| default_gesture_binding(dir));
            let observer = observer.clone();
            div()
                .id(("gesture-row", idx))
                .w_full()
                .px_3()
                .py_2()
                .rounded_md()
                .bg(rgb(SURFACE))
                .hover(|s| s.bg(rgb(SURFACE_HOVER)))
                .child(
                    v_flex()
                        .gap_0p5()
                        .child(
                            div()
                                .text_xs()
                                .text_color(rgb(TEXT_MUTED))
                                .child(format!("{}  {}", dir.glyph(), dir.label())),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(TEXT_PRIMARY))
                                .child(action.label()),
                        ),
                )
                .on_click(move |_event, _window, cx| {
                    cx.update_global::<AppState, _>(|state, _| {
                        state.gesture_edit = Some(dir);
                    });
                    observer.update(cx, |_, cx| cx.notify());
                })
                .into_any_element()
        })
        .collect();

    v_flex()
        .min_w(px(POPOVER_W))
        .gap_1()
        .p_2()
        .child(
            div()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .px_2()
                .pb_1()
                .child("Gesture Button"),
        )
        .children(rows)
        .into_any_element()
}

/// Sub-page: action catalog for `direction`. A `← Back` row returns to the
/// directions list without committing; picking an action commits + returns.
fn gesture_action_picker<T: 'static>(
    direction: GestureDirection,
    observer: &Entity<T>,
    cx: &mut Context<PopoverState>,
) -> AnyElement {
    let current = cx
        .try_global::<AppState>()
        .and_then(|s| s.gesture_bindings.get(&direction).cloned());

    let catalog = Action::catalog();
    let mut sections: Vec<(Category, Vec<Action>)> = Vec::new();
    for action in catalog {
        let cat = action.category();
        if let Some(sec) = sections.iter_mut().find(|(c, _)| *c == cat) {
            sec.1.push(action);
        } else {
            sections.push((cat, vec![action]));
        }
    }

    let mut item_idx: usize = 0;
    let mut children: Vec<AnyElement> = Vec::new();
    for (category, actions) in sections {
        children.push(
            div()
                .w_full()
                .px_2()
                .pt_2()
                .pb_1()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .child(category.label())
                .into_any_element(),
        );
        for action in actions {
            let is_selected = current.as_ref() == Some(&action);
            let label = action.label();
            let observer = observer.clone();
            let action = Rc::new(action);
            let idx = item_idx;
            item_idx += 1;
            children.push(
                div()
                    .id(("gesture-action-item", idx))
                    .w_full()
                    .px_3()
                    .py_1p5()
                    .rounded_md()
                    .text_sm()
                    .text_color(rgb(if is_selected { ACCENT_BLUE } else { TEXT_PRIMARY }))
                    .bg(rgb(if is_selected { SURFACE_HOVER } else { SURFACE }))
                    .hover(|s| s.bg(rgb(SURFACE_HOVER)))
                    .child(label)
                    .on_click(move |_event, _window, cx| {
                        let action = (*action).clone();
                        cx.update_global::<AppState, _>(|state, _| {
                            state.commit_gesture_binding(direction, action);
                            // Return to the directions list — the user can
                            // bind another direction or dismiss themselves.
                            state.gesture_edit = None;
                        });
                        observer.update(cx, |_, cx| cx.notify());
                    })
                    .into_any_element(),
            );
        }
    }

    let observer_back = observer.clone();
    v_flex()
        .min_w(px(POPOVER_W))
        .gap_1()
        .p_2()
        .child(
            div()
                .id("gesture-back")
                .px_2()
                .pb_1()
                .text_xs()
                .text_color(rgb(TEXT_MUTED))
                .hover(|s| s.text_color(rgb(TEXT_PRIMARY)))
                .child(format!("← Gesture {}", direction.label()))
                .on_click(move |_event, _window, cx| {
                    cx.update_global::<AppState, _>(|state, _| {
                        state.gesture_edit = None;
                    });
                    observer_back.update(cx, |_, cx| cx.notify());
                }),
        )
        .child(
            div()
                .id("gesture-picker-scroll")
                .max_h(px(POPOVER_LIST_MAX_H))
                .overflow_y_scroll()
                .children(children),
        )
        .into_any_element()
}
