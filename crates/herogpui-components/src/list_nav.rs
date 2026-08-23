//! Arrow-key navigation shared by every list-shaped control.
//!
//! v3 sits on React Aria, so a listbox, a select's popover, a dropdown menu and
//! a combo box's suggestions all answer the same four keys the same way. Keeping
//! one implementation means they cannot disagree — and a list that only answers
//! the pointer is not the same control.

/// What a keystroke does to the cursor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Move {
    /// Put the cursor on this stop.
    To(usize),
    /// Activate the row the cursor is on.
    Activate,
    /// Not a navigation key.
    Ignore,
}

/// Where `key` moves the cursor, over the rows a keyboard may land on.
///
/// `stops` holds the indices of the selectable rows, in order — sections,
/// separators and disabled rows are left out, so the cursor never stops on
/// something that cannot be chosen. `from` is the cursor's current index into
/// the *item* list, not into `stops`.
///
/// With nothing focused, `down` starts at the top and `up` at the bottom, which
/// is what React Aria does. `wrap` is `shouldFocusWrap`: without it the ends
/// hold instead of joining up.
pub fn resolve(stops: &[usize], from: Option<usize>, key: &str, wrap: bool) -> Move {
    if stops.is_empty() {
        return Move::Ignore;
    }
    let last = stops.len() as i32 - 1;
    let here = from.and_then(|i| stops.iter().position(|s| *s == i));

    let step = |delta: i32| -> Move {
        let next = match here {
            None if delta > 0 => 0,
            None => last,
            Some(pos) => {
                let raw = pos as i32 + delta;
                if raw < 0 {
                    if wrap {
                        last
                    } else {
                        0
                    }
                } else if raw > last {
                    if wrap {
                        0
                    } else {
                        last
                    }
                } else {
                    raw
                }
            }
        };
        stops
            .get(next as usize)
            .copied()
            .map_or(Move::Ignore, Move::To)
    };

    match key {
        "down" => step(1),
        "up" => step(-1),
        "home" => Move::To(stops[0]),
        "end" => Move::To(stops[stops.len() - 1]),
        "enter" | "space" => Move::Activate,
        _ => Move::Ignore,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn down_from_nothing_starts_at_the_top() {
        assert_eq!(resolve(&[1, 2, 4], None, "down", false), Move::To(1));
    }

    #[test]
    fn up_from_nothing_starts_at_the_bottom() {
        assert_eq!(resolve(&[1, 2, 4], None, "up", false), Move::To(4));
    }

    #[test]
    fn skips_the_rows_that_are_not_stops() {
        // 3 is a separator, so Down from 2 lands on 4.
        assert_eq!(resolve(&[1, 2, 4], Some(2), "down", false), Move::To(4));
    }

    #[test]
    fn the_ends_hold_without_wrap() {
        assert_eq!(resolve(&[1, 2, 4], Some(4), "down", false), Move::To(4));
        assert_eq!(resolve(&[1, 2, 4], Some(1), "up", false), Move::To(1));
    }

    #[test]
    fn the_ends_join_up_with_wrap() {
        assert_eq!(resolve(&[1, 2, 4], Some(4), "down", true), Move::To(1));
        assert_eq!(resolve(&[1, 2, 4], Some(1), "up", true), Move::To(4));
    }

    #[test]
    fn home_and_end_jump() {
        assert_eq!(resolve(&[1, 2, 4], Some(2), "home", false), Move::To(1));
        assert_eq!(resolve(&[1, 2, 4], Some(2), "end", false), Move::To(4));
    }

    #[test]
    fn enter_and_space_activate() {
        assert_eq!(resolve(&[1], Some(1), "enter", false), Move::Activate);
        assert_eq!(resolve(&[1], Some(1), "space", false), Move::Activate);
    }

    #[test]
    fn anything_else_is_ignored() {
        assert_eq!(resolve(&[1], Some(1), "a", false), Move::Ignore);
        assert_eq!(resolve(&[], None, "down", false), Move::Ignore);
    }
}
