"""Add the per-demo state pools the ported examples need."""
import io

P = 'gallery/src/app.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""    pub color_field_state: Entity<h::InputState>,
}""",
    """    pub color_field_state: Entity<h::InputState>,

    // -- per-demo state -----------------------------------------------------
    //
    // v3's examples each own their state: its "Controlled" demo and its
    // "Disabled State" demo are separate fields. Sharing one entity across a
    // page would make typing in one demo change every other, so these are
    // keyed by demo id and created on first render -- a page only pays for the
    // demos it actually shows.
    pub demo_text: HashMap<&'static str, Entity<h::InputState>>,
    pub demo_number: HashMap<&'static str, Entity<h::NumberState>>,
    pub demo_flags: HashMap<&'static str, bool>,
    pub demo_choice: HashMap<&'static str, Option<usize>>,
    pub demo_keys: HashMap<&'static str, Vec<SharedString>>,
    pub demo_values: HashMap<&'static str, f32>,
}""")

rep("""impl Gallery {
    pub fn new(cx: &mut Context<'_, Self>) -> Self {""",
    """impl Gallery {
    /// The text state for one demo, created on first use.
    ///
    /// `initial` seeds it the way v3's `defaultValue` does, and only on the
    /// first call -- later renders return the state the user has been editing.
    pub fn demo_text(
        &mut self,
        key: &'static str,
        initial: &str,
        cx: &mut App,
    ) -> Entity<h::InputState> {
        if let Some(state) = self.demo_text.get(key) {
            return state.clone();
        }
        let initial = initial.to_owned();
        let state = cx.new(|cx| h::InputState::with_value(cx, initial));
        self.demo_text.insert(key, state.clone());
        state
    }

    /// The numeric state for one demo, created on first use.
    pub fn demo_number(
        &mut self,
        key: &'static str,
        value: f64,
        min: f64,
        max: f64,
        step: f64,
        cx: &mut App,
    ) -> Entity<h::NumberState> {
        if let Some(state) = self.demo_number.get(key) {
            return state.clone();
        }
        let state = cx.new(|cx| {
            let mut n = h::NumberState::new(cx, value);
            n.set_range(min, max);
            n.set_step(step);
            n
        });
        self.demo_number.insert(key, state.clone());
        state
    }

    /// A boolean a demo owns (selected, open, checked).
    pub fn demo_flag(&self, key: &str, default: bool) -> bool {
        self.demo_flags.get(key).copied().unwrap_or(default)
    }

    pub fn set_demo_flag(&mut self, key: &'static str, v: bool) {
        self.demo_flags.insert(key, v);
    }

    /// A single-selection index a demo owns.
    pub fn demo_choice(&self, key: &str, default: Option<usize>) -> Option<usize> {
        self.demo_choice.get(key).copied().unwrap_or(default)
    }

    pub fn set_demo_choice(&mut self, key: &'static str, v: Option<usize>) {
        self.demo_choice.insert(key, v);
    }

    /// A multi-selection a demo owns.
    pub fn demo_keys(&self, key: &str) -> Vec<SharedString> {
        self.demo_keys.get(key).cloned().unwrap_or_default()
    }

    pub fn set_demo_keys(&mut self, key: &'static str, v: Vec<SharedString>) {
        self.demo_keys.insert(key, v);
    }

    /// A numeric value a demo owns (slider, progress).
    pub fn demo_value(&self, key: &str, default: f32) -> f32 {
        self.demo_values.get(key).copied().unwrap_or(default)
    }

    pub fn set_demo_value(&mut self, key: &'static str, v: f32) {
        self.demo_values.insert(key, v);
    }

    pub fn new(cx: &mut Context<'_, Self>) -> Self {""")

rep("use std::collections::HashSet;",
    "use std::collections::{HashMap, HashSet};")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched app.rs')
