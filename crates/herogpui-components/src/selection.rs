//! The shared `selectionMode` semantics.
//!
//! Toggle groups, dropdown menus, tables and pickers all answer the same
//! question when an item is activated: what is the selection now? Keeping one
//! implementation means they cannot disagree.

use gpui::SharedString;
use herogpui_core::SelectionMode;

/// The selection after activating `key`.
///
/// `Single` collapses to just `key` (or clears it, unless
/// `disallow_empty_selection`); `Multiple` toggles membership; `None` is inert.
pub fn next_selection(
    current: &[SharedString],
    key: &SharedString,
    mode: SelectionMode,
    disallow_empty: bool,
) -> Vec<SharedString> {
    let is_selected = current.iter().any(|k| k == key);
    match mode {
        SelectionMode::None => current.to_vec(),
        SelectionMode::Single => {
            if is_selected && !disallow_empty {
                Vec::new()
            } else {
                vec![key.clone()]
            }
        }
        SelectionMode::Multiple => {
            if is_selected {
                if disallow_empty && current.len() == 1 {
                    return current.to_vec();
                }
                current.iter().filter(|k| *k != key).cloned().collect()
            } else {
                let mut next = current.to_vec();
                next.push(key.clone());
                next
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(items: &[&str]) -> Vec<SharedString> {
        items
            .iter()
            .map(|s| SharedString::from(s.to_string()))
            .collect()
    }

    #[test]
    fn single_replaces() {
        let next = next_selection(
            &keys(&["a"]),
            &SharedString::from("b"),
            SelectionMode::Single,
            false,
        );
        assert_eq!(next, keys(&["b"]));
    }

    #[test]
    fn single_clears_on_reselect() {
        let next = next_selection(
            &keys(&["a"]),
            &SharedString::from("a"),
            SelectionMode::Single,
            false,
        );
        assert!(next.is_empty());
    }

    #[test]
    fn single_keeps_when_empty_disallowed() {
        let next = next_selection(
            &keys(&["a"]),
            &SharedString::from("a"),
            SelectionMode::Single,
            true,
        );
        assert_eq!(next, keys(&["a"]));
    }

    #[test]
    fn multiple_toggles() {
        let added = next_selection(
            &keys(&["a"]),
            &SharedString::from("b"),
            SelectionMode::Multiple,
            false,
        );
        assert_eq!(added, keys(&["a", "b"]));
        let removed = next_selection(
            &keys(&["a", "b"]),
            &SharedString::from("a"),
            SelectionMode::Multiple,
            false,
        );
        assert_eq!(removed, keys(&["b"]));
    }

    #[test]
    fn multiple_keeps_last_when_empty_disallowed() {
        let next = next_selection(
            &keys(&["a"]),
            &SharedString::from("a"),
            SelectionMode::Multiple,
            true,
        );
        assert_eq!(next, keys(&["a"]));
        let next = next_selection(
            &keys(&["a", "b"]),
            &SharedString::from("a"),
            SelectionMode::Multiple,
            true,
        );
        assert_eq!(next, keys(&["b"]));
    }

    #[test]
    fn none_is_inert() {
        let next = next_selection(
            &keys(&["a"]),
            &SharedString::from("b"),
            SelectionMode::None,
            false,
        );
        assert_eq!(next, keys(&["a"]));
    }
}
