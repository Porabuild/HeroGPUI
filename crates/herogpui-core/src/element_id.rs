//! How a component mints the [`ElementId`]s it hands to gpui.
//!
//! # The rule
//!
//! **An element's id must be unique among everything drawn in a frame, not
//! just among that element's own parts.**
//!
//! 1. A component that stands on its own takes its id from its caller. Every
//!    interactive component in this port already does: `Button::new(id)`,
//!    `Checkbox::new(id)`, `Modal::new(id)`.
//! 2. Every *part* of that component hangs its id off the component's id with
//!    [`scoped`], and every repeated part adds its position with [`indexed`].
//!    `"dismiss"` is not an id; `scoped(&self.id, "dismiss")` is.
//!
//! A constant — `div().id("close")` — satisfies neither. It is the same id for
//! every instance of the component, so two of them on one page are one element
//! as far as gpui is concerned, and it is only ever correct by accident of an
//! ancestor the component does not control.
//!
//! # Why it matters here
//!
//! gpui keys two separate things on an element's *whole* id path (its
//! `GlobalElementId`):
//!
//! - **Element state.** `Window::with_element_state`, and therefore
//!   `Window::use_keyed_state` and every `util` helper built on it, stores
//!   per-element state under that path. Two elements with the same path share
//!   one slot, silently: open flags, hover state, scroll offsets and — through
//!   `util::tab_stop_handle` — *the focus handle itself* bleed between them.
//! - **Accessibility node ids.** gpui hashes the path into an
//!   `accesskit::NodeId`, so a duplicate path collides two a11y nodes. That
//!   half only bites once an element also reports a role, which is why a
//!   duplicate id can sit in this crate looking harmless today. Scope the ids
//!   first, add the roles second.
//!
//! Most components here are `RenderOnce` builders. A `RenderOnce` is inlined
//! into its parent's element tree and pushes *nothing* onto the id path, and
//! `use_keyed_state` calls at the top of `render` run before the component's
//! own root `div().id(..)` exists. So a component's keys are effectively
//! window-global, and "is this id unique?" cannot be answered by reading the
//! component. Deriving the id is what makes the answer local.
//!
//! # Why the structured variants beat `format!`
//!
//! The obvious spelling is `ElementId::Name(format!("{parent}-{part}").into())`,
//! and it is ambiguous. The separator is not reserved, so parent `"a-b"` with
//! part `"c"` and parent `"a"` with part `"b-c"` flatten to the identical key
//! `"a-b-c"`: two different elements, one state slot, one focus handle, no
//! error. Component ids in this port come from callers and gallery pages, which
//! spell them with hyphens all the time, so that is not a theoretical clash.
//!
//! [`ElementId::NamedChild`] keeps the parent id as *structure* — a separate
//! field, compared and hashed on its own — so those two cases cannot fold
//! together. It also costs no allocation: it clones an `Arc<ElementId>` and a
//! `SharedString` instead of formatting a fresh `String` for every part of
//! every component on every frame.
//!
//! The one thing it deliberately keeps is the `format!` spelling's `Display`:
//! `NamedChild` renders as `{parent}-{part}`, so an id that reaches a log line
//! or a `debug_selector` reads exactly as it did before.

use std::sync::Arc;

use gpui::{ElementId, SharedString};

/// The id of a named part of the element identified by `parent`.
///
/// This is the workhorse. `parent` is the component's caller-supplied id (or
/// another part's id, for a part of a part), and `part` names the piece within
/// it — `"trigger"`, `"panel"`, `"focus"`, `"checked"`.
///
/// ```
/// use gpui::ElementId;
/// use herogpui_core::element_id::scoped;
///
/// let dialog: ElementId = "confirm".into();
/// let close = scoped(&dialog, "close");
///
/// assert_eq!(close.to_string(), "confirm-close");
/// // Nesting is structure, not text: these are three distinct ids.
/// assert_ne!(close, ElementId::Name("confirm-close".into()));
/// assert_ne!(scoped(&scoped(&dialog, "a"), "b"), scoped(&dialog, "a-b"));
/// ```
pub fn scoped(parent: &ElementId, part: impl Into<SharedString>) -> ElementId {
    ElementId::NamedChild(Arc::new(parent.clone()), part.into())
}

/// The id of the `index`th of a repeated part of `parent`.
///
/// For rows, cells, tabs, options, segments — anything a component renders once
/// per item. The position is the only thing separating siblings, and `parent`
/// is what separates two of the collection.
///
/// The index rides as its own segment rather than as
/// [`ElementId::NamedInteger`], because `gpui-pre` 0.3.3 has no variant carrying
/// both a parent `ElementId` and an integer: `NamedInteger`'s name is a
/// `SharedString`, so using it here would mean flattening `parent` back into a
/// string and reintroducing exactly the ambiguity this module exists to avoid.
/// Use [`ElementId::named_usize`] directly for the rarer case of a repeated
/// part whose name is already unique on its own — an id built from an
/// `EntityId`, say — where there is no parent id to keep.
///
/// ```
/// use herogpui_core::element_id::{indexed, scoped};
///
/// let tabs = "settings".into();
///
/// assert_eq!(indexed(&tabs, "tab", 2).to_string(), "settings-tab-2");
/// assert_ne!(indexed(&tabs, "tab", 2), indexed(&tabs, "tab", 3));
/// // A part and the same part indexed are different ids.
/// assert_ne!(indexed(&tabs, "tab", 2), scoped(&tabs, "tab"));
/// ```
pub fn indexed(parent: &ElementId, part: impl Into<SharedString>, index: usize) -> ElementId {
    scoped(&scoped(parent, part), SharedString::from(index.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_parts_follow_their_parent() {
        let left: ElementId = "left".into();
        let right: ElementId = "right".into();

        assert_ne!(scoped(&left, "close"), scoped(&right, "close"));
        assert_ne!(scoped(&left, "close"), scoped(&left, "dismiss"));
        assert_eq!(scoped(&left, "close"), scoped(&left, "close"));
        // The part is a child of the parent id, not a replacement for it.
        assert_ne!(scoped(&left, "close"), left);
    }

    /// The whole point: the `format!` spelling folds these two together.
    #[test]
    fn hyphenated_parents_and_parts_cannot_collide() {
        let hyphenated: ElementId = "a-b".into();
        let plain: ElementId = "a".into();

        assert_eq!(
            format!("{}-{}", hyphenated, "c"),
            format!("{}-{}", plain, "b-c"),
            "the string spelling this module replaces really is ambiguous"
        );
        assert_ne!(scoped(&hyphenated, "c"), scoped(&plain, "b-c"));
    }

    #[test]
    fn indexed_parts_separate_siblings_and_collections() {
        let one: ElementId = "one".into();
        let two: ElementId = "two".into();

        assert_ne!(indexed(&one, "row", 0), indexed(&one, "row", 1));
        assert_ne!(indexed(&one, "row", 0), indexed(&two, "row", 0));
        assert_eq!(indexed(&one, "row", 0), indexed(&one, "row", 0));
    }

    /// `Display` parity with the `format!` spelling, which is what lets an id
    /// keep appearing in a `debug_selector` or a log line unchanged.
    #[test]
    fn display_matches_the_string_spelling() {
        let base: ElementId = "picker".into();

        assert_eq!(scoped(&base, "panel").to_string(), "picker-panel");
        assert_eq!(
            indexed(&scoped(&base, "grid"), "cell", 7).to_string(),
            "picker-grid-cell-7"
        );
    }
}
