"""DateField: the description and disabled/read-only props its examples use.

v3's DateField prop table omits `isDisabled` and lists no `Description`, but
its own "Disabled State" and "With Description" examples use both -- they come
from React Aria's DateField, which the table says the component inherits.
"""
import io

P = 'crates/herogpui-components/src/date_picker.rs'
s = io.open(P, encoding='utf-8', newline='').read()


def rep(old, new):
    global s
    assert s.count(old) == 1, (s.count(old), old[:70])
    s = s.replace(old, new)


rep("""    state: Entity<crate::input::InputState>,
    label: Option<SharedString>,
    on_change: Option<OnChange>,
}

impl DateField {""",
    """    state: Entity<crate::input::InputState>,
    label: Option<SharedString>,
    /// `Description` — composed inside the field in v3's own example.
    description: Option<SharedString>,
    is_disabled: bool,
    is_read_only: bool,
    on_change: Option<OnChange>,
}

impl DateField {""")

rep("""    pub fn is_required(mut self, v: bool) -> Self {""",
    """    /// `Description` — help text under the field.
    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// `isDisabled` — greys the field out and stops it answering keys.
    pub fn is_disabled(mut self, v: bool) -> Self {
        self.is_disabled = v;
        self
    }

    /// `isReadOnly` — shows the value but refuses edits.
    pub fn is_read_only(mut self, v: bool) -> Self {
        self.is_read_only = v;
        self
    }

    pub fn is_required(mut self, v: bool) -> Self {""")

io.open(P, 'w', encoding='utf-8', newline='').write(s)
print('patched DateField fields/builders')
