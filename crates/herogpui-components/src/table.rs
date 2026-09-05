//! Table — port of `@heroui/table` (v3).
//!
//! `variant` selects between the surface container (`primary`) and the flat,
//! transparent-row layout (`secondary`).
//!
//! Selection and sorting are controlled, as they are in v3: the table reports
//! what the user asked for and the caller owns the data. In particular the
//! table never reorders its own rows — it cannot, since a row is a list of
//! rendered cells, not a record.

use gpui::{
    prelude::*, px, AnyElement, App, ClickEvent, InteractiveElement, IntoElement, ParentElement,
    Pixels, RenderOnce, SharedString, Styled, Window,
};
use herogpui_core::SelectionMode;
use herogpui_theme::ActiveTheme;

use crate::{checkbox::Checkbox, icons};

type OnRowClick = std::sync::Arc<dyn Fn(usize, &ClickEvent, &mut Window, &mut App) + 'static>;
type OnSelectionChange = std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>;
type OnSortChange = std::sync::Arc<dyn Fn(SortDescriptor, &mut Window, &mut App) + 'static>;
type OnLoadMore = std::sync::Arc<dyn Fn(&mut Window, &mut App) + 'static>;
type OnResize = std::sync::Arc<dyn Fn(&[(SharedString, Pixels)], &mut Window, &mut App) + 'static>;
type VirtualRowKey = std::sync::Arc<dyn Fn(usize) -> SharedString + 'static>;
type VirtualRow = std::sync::Arc<dyn Fn(usize) -> TableRow + 'static>;
type VirtualRowText = std::sync::Arc<dyn Fn(usize) -> SharedString + 'static>;
type VirtualTree = std::sync::Arc<dyn Fn(usize) -> VirtualTreeMetadata + 'static>;

const DEFAULT_COLUMN_MIN_WIDTH: f32 = 75.;

/// Visual variant of a table (`variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TableVariant {
    /// A surface container behind the rows.
    #[default]
    Primary,
    /// Flat, with transparent rows and no container.
    Secondary,
}

impl TableVariant {
    pub const ALL: [TableVariant; 2] = [TableVariant::Primary, TableVariant::Secondary];

    pub fn label(self) -> &'static str {
        match self {
            TableVariant::Primary => "Primary",
            TableVariant::Secondary => "Secondary",
        }
    }
}

/// `sortDirection` on `Table.SortableColumnHeader`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    /// The direction a second click on the same column asks for.
    pub fn flipped(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    fn indicator(self) -> &'static str {
        match self {
            SortDirection::Ascending => icons::CHEVRON_UP,
            SortDirection::Descending => icons::CHEVRON_DOWN,
        }
    }
}

type Indicator = std::sync::Arc<dyn Fn(SortDirection) -> AnyElement + 'static>;

/// `sortDescriptor` — which column is sorted, and which way.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SortDescriptor {
    pub column: SharedString,
    pub direction: SortDirection,
}

impl SortDescriptor {
    pub fn new(column: impl Into<SharedString>, direction: SortDirection) -> Self {
        Self {
            column: column.into(),
            direction,
        }
    }

    /// The descriptor a click on `column` should produce.
    ///
    /// Clicking the sorted column flips it; clicking any other column starts
    /// ascending, which is what React Aria does.
    pub fn next(current: Option<&SortDescriptor>, column: impl Into<SharedString>) -> Self {
        let column = column.into();
        match current {
            Some(d) if d.column == column => Self {
                column,
                direction: d.direction.flipped(),
            },
            _ => Self {
                column,
                direction: SortDirection::Ascending,
            },
        }
    }
}

/// One column (`Table.Column`).
#[derive(Clone, Debug)]
pub struct TableColumn {
    label: SharedString,
    allows_sorting: bool,
    is_row_header: bool,
    /// `allowsResizing` — whether a handle on this column's trailing edge
    /// resizes it.
    allows_resizing: bool,
    /// `width` — a caller-owned controlled column width.
    width: Option<Pixels>,
    /// `defaultWidth` — the uncontrolled starting width. Without one the
    /// column shares the row evenly, which is what `flex-1` does.
    default_width: Option<Pixels>,
    /// `minWidth` — the column's floor.
    min_width: Option<Pixels>,
    /// `maxWidth` — the column's ceiling.
    max_width: Option<Pixels>,
}

impl TableColumn {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            allows_sorting: false,
            is_row_header: false,
            allows_resizing: false,
            width: None,
            default_width: None,
            min_width: None,
            max_width: None,
        }
    }

    /// `allowsResizing` — put a drag handle on this column's trailing edge.
    pub fn allows_resizing(mut self, v: bool) -> Self {
        self.allows_resizing = v;
        self
    }

    /// `defaultWidth` — the column's starting width, which a resize handle then
    /// moves.
    pub fn default_width(mut self, width: impl Into<Pixels>) -> Self {
        self.default_width = Some(width.into());
        self
    }

    /// `width` — the caller-owned controlled width of this column.
    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = Some(width.into());
        self
    }

    /// `minWidth` — the width this column will not go below.
    pub fn min_width(mut self, width: impl Into<Pixels>) -> Self {
        self.min_width = Some(width.into());
        self
    }

    /// `maxWidth` — the width this column will not exceed.
    pub fn max_width(mut self, width: impl Into<Pixels>) -> Self {
        self.max_width = Some(width.into());
        self
    }

    /// `allowsSorting` — makes the header a sort control.
    pub fn allows_sorting(mut self, v: bool) -> Self {
        self.allows_sorting = v;
        self
    }

    /// `isRowHeader` — this column names the row.
    ///
    /// With no accessibility layer to expose it to, the effect here is visual:
    /// the column's cells carry medium weight so the identifying value reads
    /// first.
    pub fn is_row_header(mut self, v: bool) -> Self {
        self.is_row_header = v;
        self
    }
}

impl From<SharedString> for TableColumn {
    fn from(label: SharedString) -> Self {
        TableColumn::new(label)
    }
}

impl From<&str> for TableColumn {
    fn from(label: &str) -> Self {
        TableColumn::new(label.to_owned())
    }
}

/// One row (`Table.Row`).
pub struct TableRow {
    key: Option<SharedString>,
    text_value: Option<SharedString>,
    cells: Vec<AnyElement>,
    /// The rows nested under this one. v3 calls them the row's `children`, and
    /// `expandedKeys` decides which parents show theirs.
    children: Vec<TableRow>,
}

/// Cheap tree structure for one preorder item in a virtual table.
///
/// The row factory still runs only for items the viewport builds. This metadata
/// is projected for the whole collection so expansion can derive the visible
/// indices without eagerly constructing any cells.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VirtualTreeMetadata {
    pub depth: usize,
    pub parent_key: Option<SharedString>,
    pub has_children: bool,
}

impl TableRow {
    pub fn new(cells: Vec<AnyElement>) -> Self {
        Self {
            key: None,
            text_value: None,
            cells,
            children: Vec::new(),
        }
    }

    /// The rows nested under this one.
    pub fn children(mut self, rows: Vec<TableRow>) -> Self {
        self.children = rows;
        self
    }

    /// The selection key. Defaults to the row's index.
    pub fn key(mut self, key: impl Into<SharedString>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// The plain-text value used by the table's typeahead.
    ///
    /// Cells are opaque `AnyElement`s, so rows without this value do not
    /// participate in typeahead even when a cell visibly draws text.
    pub fn text_value(mut self, value: impl Into<SharedString>) -> Self {
        self.text_value = Some(value.into());
        self
    }
}

type OnExpandedChange = std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowActivation {
    Pointer,
    Enter,
    Space,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RowIntent {
    Action,
    Selection,
    None,
}

#[derive(Clone, Debug, Default)]
struct TableTypeahead {
    query: String,
    last: Option<web_time::Instant>,
}

#[derive(Clone, Debug, Default)]
struct TableSelectionRange {
    anchor: Option<SharedString>,
    current: Option<SharedString>,
    is_all: bool,
}

impl TableTypeahead {
    fn is_active(&self, now: web_time::Instant) -> bool {
        !self.query.is_empty()
            && self
                .last
                .is_some_and(|last| now.duration_since(last) <= crate::list_nav::TYPEAHEAD_TIMEOUT)
    }

    fn push(&mut self, key: &str, now: web_time::Instant) -> String {
        if self
            .last
            .is_none_or(|last| now.duration_since(last) > crate::list_nav::TYPEAHEAD_TIMEOUT)
        {
            self.query.clear();
        }
        self.last = Some(now);
        self.query.push_str(key);
        self.query.clone()
    }

    fn clear(&mut self) {
        self.query.clear();
        self.last = None;
    }
}

#[derive(Clone)]
struct TableTypeaheadNavigation {
    state: gpui::Entity<TableTypeahead>,
    labels: Vec<String>,
    virtual_indices: Option<Vec<usize>>,
    virtual_text: Option<VirtualRowText>,
    stops: Vec<usize>,
    keys: Vec<SharedString>,
    cursor: gpui::Entity<Option<SharedString>>,
    fixed_virtual: bool,
    fixed_scroll: gpui::UniformListScrollHandle,
    variable_scroll: Option<gpui::ListState>,
}

impl TableTypeaheadNavigation {
    fn push(
        &self,
        character: &str,
        now: web_time::Instant,
        clear_on_failure: bool,
        cx: &mut App,
    ) -> bool {
        let from = self
            .cursor
            .read(cx)
            .as_ref()
            .and_then(|key| self.keys.iter().position(|row| row == key))
            .filter(|index| self.stops.contains(index));
        let mut query = String::new();
        self.state.update(cx, |state, _| {
            query = state.push(character, now);
        });
        let projected_labels = self
            .virtual_text
            .as_ref()
            .zip(self.virtual_indices.as_ref())
            .map(|(text_value, indices)| {
                indices
                    .iter()
                    .map(|index| text_value(*index).to_string())
                    .collect::<Vec<_>>()
            });
        let searchable_labels = projected_labels.as_ref().unwrap_or(&self.labels);
        let Some(next) =
            crate::list_nav::typeahead(searchable_labels, &self.stops, from, &query, false)
        else {
            if clear_on_failure {
                self.state.update(cx, |state, _| state.clear());
            }
            return false;
        };
        self.cursor.update(cx, |value, cx| {
            *value = self.keys.get(next).cloned();
            cx.notify();
        });
        if self.fixed_virtual {
            self.fixed_scroll
                .scroll_to_item(next, gpui::ScrollStrategy::Center);
        } else if let Some(state) = &self.variable_scroll {
            state.scroll_to(gpui::ListOffset {
                item_ix: next,
                offset_in_item: px(0.),
            });
        }
        true
    }
}

#[derive(Clone, PartialEq, Eq)]
enum LoadMoreCollection {
    Rows(Vec<SharedString>),
    Virtual {
        count: usize,
        identity: SharedString,
    },
}

fn row_intent(
    activation: RowActivation,
    mode: SelectionMode,
    selection_is_empty: bool,
    has_action: bool,
) -> RowIntent {
    let primary_action = has_action && (mode == SelectionMode::None || selection_is_empty);
    match activation {
        RowActivation::Pointer if primary_action => RowIntent::Action,
        RowActivation::Pointer if mode != SelectionMode::None => RowIntent::Selection,
        RowActivation::Enter if primary_action => RowIntent::Action,
        RowActivation::Enter if !has_action && mode != SelectionMode::None => RowIntent::Selection,
        RowActivation::Space if mode != SelectionMode::None => RowIntent::Selection,
        _ => RowIntent::None,
    }
}

fn select_all_flags(selectable: &[SharedString], selected: &[SharedString]) -> (bool, bool) {
    let selected_count = selectable
        .iter()
        .filter(|key| selected.contains(key))
        .count();
    let all_selected = !selectable.is_empty() && selected_count == selectable.len();
    let indeterminate = !selected.is_empty() && !all_selected;
    (all_selected, indeterminate)
}

fn same_selection(left: &[SharedString], right: &[SharedString]) -> bool {
    left.len() == right.len()
        && left.iter().collect::<std::collections::HashSet<_>>()
            == right.iter().collect::<std::collections::HashSet<_>>()
}

fn extend_selection_range(
    current: &[SharedString],
    collection: &[SharedString],
    selectable: &[SharedString],
    range: &TableSelectionRange,
    target: &SharedString,
) -> Vec<SharedString> {
    if range.is_all {
        return vec![target.clone()];
    }
    let anchor = range.anchor.as_ref().unwrap_or(target);
    let previous = range.current.as_ref().unwrap_or(target);
    let mut anchor_at = None;
    let mut previous_at = None;
    let mut target_at = None;
    for (index, key) in collection.iter().enumerate() {
        anchor_at = anchor_at.or_else(|| (key == anchor).then_some(index));
        previous_at = previous_at.or_else(|| (key == previous).then_some(index));
        target_at = target_at.or_else(|| (key == target).then_some(index));
    }
    let between = |from: Option<usize>, to: Option<usize>| {
        from.zip(to)
            .map(|(from, to)| if from <= to { from..=to } else { to..=from })
    };
    let mut next = current.to_vec();
    if let Some(previous_range) = between(anchor_at, previous_at) {
        let removed: std::collections::HashSet<&SharedString> =
            previous_range.map(|index| &collection[index]).collect();
        next.retain(|key| !removed.contains(key));
    }
    if let Some(target_range) = between(anchor_at, target_at) {
        let selectable: std::collections::HashSet<&SharedString> = selectable.iter().collect();
        let mut selected: std::collections::HashSet<SharedString> = next.iter().cloned().collect();
        for index in target_range {
            let key = &collection[index];
            if selectable.contains(key) && selected.insert(key.clone()) {
                next.push(key.clone());
            }
        }
    }
    next
}

/// Pinned React Aria 3.51.0 `useSelectableCollection` registers Home and End
/// only for the chords each platform's handler admits: none, Shift, Alt, and
/// Alt+Shift on macOS -- no Meta or Control handler exists -- and none,
/// Shift, Control, and Control+Shift on Windows and Linux. The upstream
/// matcher reads exactly the browser's canonical modifier flags -- Alt,
/// Control, Meta, Shift -- so GPUI's `function` flag is ignored here: a
/// browser exposes no Fn state for it to read, so vetoing on the flag would
/// claim a pinned guard that does not exist, and the framework delivers an
/// Fn-bearing press with every matched modifier flag still false. A chord
/// outside the registration is entirely inert: no focus move, no selection,
/// no preventDefault. `macos` is simulated explicitly so every platform's
/// unit tests can prove both maps.
fn home_end_registered(modifiers: gpui::Modifiers, macos: bool) -> bool {
    if macos {
        !modifiers.control && !modifiers.platform
    } else {
        !modifiers.alt && !modifiers.platform
    }
}

/// Pinned `useSelectableCollection` (`isCtrlKeyPressed`): a Shift move
/// extends the range on the collection's navigation keys, while Home and
/// End extend only from Control+Shift on Windows and Linux. macOS registers
/// no Home/End extension at all -- its Shift and Alt+Shift chords move the
/// focus alone -- so the platform is an explicit bool rather than a `cfg!`.
fn shift_home_end_extends(key_name: &str, control: bool, macos: bool) -> bool {
    !matches!(key_name, "home" | "end") || (!macos && control)
}

/// A column with a `defaultWidth` takes it; the rest split what is left.
///
/// `flex_basis(0)` is the part that matters: a bare `flex_1` sizes a cell by its
/// content, so the tree column's indent shifted every column after it, and a
/// fraction of the *whole* row cannot account for the fixed columns.
fn flex_cell(el: gpui::Div) -> gpui::Div {
    el.flex_basis(px(0.)).flex_1()
}

/// HeroUI Table.
#[derive(IntoElement)]
pub struct Table {
    /// Distinguishes one table's keyed state from another's: the resized column
    /// widths, the drag in progress, the focus handle and the row cursor all
    /// hang off it. v3 needs no such prop -- React keys by position in the tree
    /// -- so the default is a name shared by every table, which only goes wrong
    /// when a page shows two of them.
    id: SharedString,
    /// `indicator` — v3's render prop for the sort chevron, handed the
    /// `sortDirection` the column is sorted in.
    indicator: Option<Indicator>,
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    /// `treeColumn` — which column carries the expand chevron.
    tree_column: usize,
    /// `expandedKeys` — the parent rows showing their children.
    expanded_keys: Vec<SharedString>,
    on_expanded_change: Option<OnExpandedChange>,
    /// `TableLayout`'s `rowHeight`. Together with [`Table::virtual_rows`] it
    /// virtualizes the body: a fixed row height is what lets the scroll geometry
    /// be computed rather than laid out.
    row_height: Option<Pixels>,
    /// `TableLayout`'s `estimatedRowHeight` — virtualizes rows whose heights
    /// differ, where `rowHeight` needs them all the same.
    estimated_row_height: Option<Pixels>,
    /// `TableLayout`'s `loaderHeight` — the fixed height of the load-more row.
    loader_height: Option<Pixels>,
    /// `scrollOffset` on `Table.LoadMore`, in viewport heights.
    load_more_offset: f32,
    /// How tall the virtual body is. v3 sets it with a `className`.
    max_h: Option<Pixels>,
    /// `TableLayout`'s `gap` and `padding`, both 0 in v3: rows meet, separated
    /// by their border rather than by space.
    gap: Option<Pixels>,
    padding: Option<Pixels>,
    /// The row factory a virtual table needs. Cells are `AnyElement`, which
    /// cannot be built up front and then handed out again on the next scroll,
    /// so a virtual table asks for its rows one at a time.
    virtual_rows: Option<(usize, SharedString, VirtualRowKey, VirtualRow)>,
    virtual_text_value: Option<VirtualRowText>,
    virtual_tree_metadata: Option<VirtualTree>,
    variant: TableVariant,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    disabled_keys: Vec<SharedString>,
    is_selection_controlled: bool,
    sort_descriptor: Option<SortDescriptor>,
    show_indicator: bool,
    is_pending: bool,
    empty_state: Option<AnyElement>,
    footer: Option<AnyElement>,
    on_row_click: Option<OnRowClick>,
    on_selection_change: Option<OnSelectionChange>,
    on_sort_change: Option<OnSortChange>,
    on_load_more: Option<OnLoadMore>,
    on_resize_start: Option<OnResize>,
    on_resize: Option<OnResize>,
    on_resize_end: Option<OnResize>,
}

impl Table {
    pub fn new(columns: Vec<SharedString>) -> Self {
        Self {
            id: SharedString::from("table"),
            indicator: None,
            columns: columns.into_iter().map(TableColumn::new).collect(),
            rows: Vec::new(),
            tree_column: 0,
            expanded_keys: Vec::new(),
            on_expanded_change: None,
            row_height: None,
            estimated_row_height: None,
            loader_height: None,
            load_more_offset: 1.0,
            max_h: None,
            gap: None,
            padding: None,
            virtual_rows: None,
            virtual_text_value: None,
            virtual_tree_metadata: None,
            variant: TableVariant::Primary,
            selection_mode: SelectionMode::None,
            selected_keys: Vec::new(),
            disabled_keys: Vec::new(),
            is_selection_controlled: false,
            sort_descriptor: None,
            show_indicator: true,
            is_pending: false,
            empty_state: None,
            footer: None,
            on_row_click: None,
            on_selection_change: None,
            on_sort_change: None,
            on_load_more: None,
            on_resize_start: None,
            on_resize: None,
            on_resize_end: None,
        }
    }

    /// The id this table's own state is keyed by.
    ///
    /// Two tables on one page share their resized widths, their row cursor and
    /// their focus without it, because `use_keyed_state` keys by the id it is
    /// given and nothing else.
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = id.into();
        self
    }

    /// `indicator` — replaces the sort chevron.
    ///
    /// The closure receives `sortDirection`, the value v3 passes into the same
    /// render prop, so a caller can draw an arrow, a caret or a label without
    /// re-deriving which way the column is sorted.
    pub fn indicator(mut self, render: impl Fn(SortDirection) -> AnyElement + 'static) -> Self {
        self.indicator = Some(std::sync::Arc::new(render));
        self
    }

    /// Replaces the columns with fully configured ones.
    pub fn columns(mut self, columns: Vec<TableColumn>) -> Self {
        self.columns = columns;
        self
    }

    /// Adds one configured column.
    pub fn column(mut self, column: impl Into<TableColumn>) -> Self {
        self.columns.push(column.into());
        self
    }

    pub fn variant(mut self, variant: TableVariant) -> Self {
        self.variant = variant;
        self
    }

    /// `TableLayout`'s `rowHeight` — and, with [`Table::virtual_rows`], what
    /// virtualizes the body.
    ///
    /// v3 wraps the table in `<Virtualizer layout={TableLayout}
    /// layoutOptions={{rowHeight: 40}}>`; the wrapper has no separate identity
    /// here, so the option that defines the layout carries it. gpui's
    /// `uniform_list` builds only the rows the viewport shows, and it can do
    /// that because every row is this tall.
    pub fn row_height(mut self, height: impl Into<Pixels>) -> Self {
        self.row_height = Some(height.into());
        self
    }

    /// How tall the scrolling body is. v3's example sets it with `h-[400px]` on
    /// the table itself.
    pub fn max_h(mut self, height: impl Into<Pixels>) -> Self {
        self.max_h = Some(height.into());
        self
    }

    /// `TableLayout`'s `gap` -- space between rows, which v3 leaves at 0.
    pub fn gap(mut self, gap: impl Into<Pixels>) -> Self {
        self.gap = Some(gap.into());
        self
    }

    /// `TableLayout`'s `padding` -- space around the rows, 0 in v3.
    pub fn padding(mut self, padding: impl Into<Pixels>) -> Self {
        self.padding = Some(padding.into());
        self
    }

    /// The rows of a virtualized table, built on demand.
    ///
    /// v3 writes `<Table items={users}>{(user) => <Table.Row>…}</Table>` and the
    /// Virtualizer calls that function for the rows in view. Cells here are
    /// `AnyElement`, which is built once and consumed once, so a virtual table
    /// cannot be handed its rows up front -- it has to be able to ask again
    /// every time the viewport moves.
    /// `key` projects the collection key without building the row. Keyboard
    /// navigation and select-all need the full collection, but calling `row`
    /// for every item would defeat virtualization.
    /// `identity` stays stable for one collection and changes when the caller
    /// replaces it, so a visible load-more sentinel can re-arm without building
    /// every row just to compare their keys.
    /// `TableLayout`'s `estimatedRowHeight` — virtualize rows that are not all
    /// one height.
    ///
    /// `rowHeight` maps to `uniform_list`, which measures one row and multiplies;
    /// this maps to gpui's `list`, which measures every row it builds. The
    /// estimate is what it renders beyond the viewport while it learns the real
    /// heights.
    pub fn estimated_row_height(mut self, height: impl Into<Pixels>) -> Self {
        self.estimated_row_height = Some(height.into());
        self
    }

    /// `TableLayout`'s `loaderHeight` — the height of the load-more row.
    pub fn loader_height(mut self, height: impl Into<Pixels>) -> Self {
        self.loader_height = Some(height.into());
        self
    }

    /// Virtual rows for a collection of `count` items.
    ///
    /// `identity` names the collection: it must stay stable for one collection
    /// and change when the caller replaces it. The key and row closures are
    /// treated as stable for that identity — their output is cached between
    /// frames and replayed whenever count, identity, tree mode and the
    /// expanded set are unchanged. If `key` (or [`Table::virtual_tree_metadata`]'s
    /// `metadata`) would return different output for the same indices without
    /// the collection being replaced, the caller must change `identity` so the
    /// cached projection is rebuilt.
    pub fn virtual_rows(
        mut self,
        count: usize,
        identity: impl Into<SharedString>,
        key: impl Fn(usize) -> SharedString + 'static,
        row: impl Fn(usize) -> TableRow + 'static,
    ) -> Self {
        self.virtual_rows = Some((
            count,
            identity.into(),
            std::sync::Arc::new(key),
            std::sync::Arc::new(row),
        ));
        self
    }

    /// Projects each virtual row's `textValue` without building its cells.
    /// Virtual rows do not participate in typeahead unless this is supplied.
    pub fn virtual_text_value(
        mut self,
        text_value: impl Fn(usize) -> SharedString + 'static,
    ) -> Self {
        self.virtual_text_value = Some(std::sync::Arc::new(text_value));
        self
    }

    /// Supplies preorder tree structure for [`Table::virtual_rows`].
    ///
    /// `count` remains the size of the underlying collection. The projection
    /// identifies each item's parent, depth and expandability; controlled
    /// `expanded_keys` then decides which source indices are visible. The
    /// metadata closure is treated as stable for the [`Table::virtual_rows`]
    /// collection identity: if its output changes for the same indices, the
    /// caller must change that identity so the cached projection is rebuilt.
    pub fn virtual_tree_metadata(
        mut self,
        metadata: impl Fn(usize) -> VirtualTreeMetadata + 'static,
    ) -> Self {
        self.virtual_tree_metadata = Some(std::sync::Arc::new(metadata));
        self
    }

    /// Adds one row of cells (`Table.Row` cells).
    /// `treeColumn` — the column whose cells carry the expand chevron.
    pub fn tree_column(mut self, index: usize) -> Self {
        self.tree_column = index;
        self
    }

    /// `expandedKeys` — the parent rows showing their children.
    pub fn expanded_keys(mut self, keys: impl IntoIterator<Item = SharedString>) -> Self {
        self.expanded_keys = keys.into_iter().collect();
        self
    }

    /// Reports the expanded set after a chevron is pressed.
    pub fn on_expanded_change(
        mut self,
        f: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_expanded_change = Some(std::sync::Arc::new(f));
        self
    }

    /// Adds a row, with any nested rows it carries.
    pub fn tree_row(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn row(mut self, cells: Vec<AnyElement>) -> Self {
        self.rows.push(TableRow::new(cells));
        self
    }

    /// Adds one row under an explicit selection key.
    pub fn keyed_row(mut self, key: impl Into<SharedString>, cells: Vec<AnyElement>) -> Self {
        self.rows.push(TableRow::new(cells).key(key));
        self
    }

    /// `selectionMode` — adds the selection column when not `None`.
    pub fn selection_mode(mut self, mode: SelectionMode) -> Self {
        self.selection_mode = mode;
        self
    }

    /// `selectedKeys` — the controlled selection.
    pub fn selected_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.selected_keys = keys.into_iter().map(Into::into).collect();
        self.is_selection_controlled = true;
        self
    }

    /// React Aria's inherited `disabledKeys` — rows excluded from selection,
    /// actions and the roving keyboard stops.
    pub fn disabled_keys(
        mut self,
        keys: impl IntoIterator<Item = impl Into<SharedString>>,
    ) -> Self {
        self.disabled_keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// `onSelectionChange` — the whole selection after a row is toggled.
    pub fn on_selection_change(
        mut self,
        f: impl Fn(&[SharedString], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_selection_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `sortDescriptor` — the current sort, shown in the header.
    pub fn sort_descriptor(mut self, descriptor: SortDescriptor) -> Self {
        self.sort_descriptor = Some(descriptor);
        self
    }

    /// `onSortChange` — a sortable header was clicked.
    ///
    /// The table does not reorder its rows; a row is already-rendered cells, so
    /// the caller sorts its data and rebuilds.
    pub fn on_sort_change(
        mut self,
        f: impl Fn(SortDescriptor, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort_change = Some(std::sync::Arc::new(f));
        self
    }

    /// `showIndicator` — whether a sorted column draws its direction chevron.
    pub fn show_indicator(mut self, v: bool) -> Self {
        self.show_indicator = v;
        self
    }

    /// `isLoading` on `Table.LoadMore` — shows the loading sentinel row.
    pub fn is_pending(mut self, v: bool) -> Self {
        self.is_pending = v;
        self
    }

    /// `scrollOffset` on `Table.LoadMore` — how many viewport heights before
    /// the sentinel enters view the load should start. React Aria defaults to
    /// one; v3's async example sets zero for an exact intersection.
    pub fn scroll_offset(mut self, offset: f32) -> Self {
        self.load_more_offset = offset;
        self
    }

    /// `onLoadMore` — the sentinel row entered the scroll viewport.
    pub fn on_load_more(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_load_more = Some(std::sync::Arc::new(f));
        self
    }

    /// `onResizeStart` on `Table.ResizableContainer` — reports the current
    /// pixel widths when pointer or keyboard resizing begins.
    pub fn on_resize_start(
        mut self,
        f: impl Fn(&[(SharedString, Pixels)], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resize_start = Some(std::sync::Arc::new(f));
        self
    }

    /// `onResize` on `Table.ResizableContainer` — reports each proposed width
    /// map. Feed the values back through [`TableColumn::width`] for controlled
    /// resizing.
    pub fn on_resize(
        mut self,
        f: impl Fn(&[(SharedString, Pixels)], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resize = Some(std::sync::Arc::new(f));
        self
    }

    /// `onResizeEnd` on `Table.ResizableContainer` — reports the final pixel
    /// widths after pointer or keyboard resizing ends.
    pub fn on_resize_end(
        mut self,
        f: impl Fn(&[(SharedString, Pixels)], &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_resize_end = Some(std::sync::Arc::new(f));
        self
    }

    /// `Table.Footer` — a row under the body, which is where v3 puts a table's
    /// pagination.
    pub fn footer(mut self, content: impl IntoElement) -> Self {
        self.footer = Some(content.into_any_element());
        self
    }

    /// `renderEmptyState` — shown in place of rows when there are none.
    pub fn empty_state(mut self, content: impl IntoElement) -> Self {
        self.empty_state = Some(content.into_any_element());
        self
    }

    pub fn on_row_click(
        mut self,
        f: impl Fn(usize, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_click = Some(std::sync::Arc::new(f));
        self
    }

    /// Every row's selection key, in order.
    fn row_keys(&self) -> Vec<SharedString> {
        fn collect(rows: &[TableRow], path: &str, out: &mut Vec<SharedString>) {
            for (index, row) in rows.iter().enumerate() {
                let key = match &row.key {
                    Some(key) => key.clone(),
                    None if path.is_empty() => SharedString::from(index.to_string()),
                    None => SharedString::from(format!("{path}-{index}")),
                };
                out.push(key.clone());
                collect(&row.children, key.as_ref(), out);
            }
        }

        let mut keys = Vec::new();
        collect(&self.rows, "", &mut keys);
        keys
    }
}

/// A virtual table's projected keys and visible rows, cached between frames.
///
/// The projection depends only on what [`Table::virtual_rows`] already names —
/// the collection's identity and count — plus the expanded set and whether
/// tree metadata is projected. Any frame where none of those changed reuses
/// the cached projection instead of calling the key and tree closures again;
/// the closures are free to return different values only when the caller
/// replaces the collection, which is what `identity` is for.
#[derive(Default)]
struct VirtualProjection {
    count: usize,
    identity: SharedString,
    expanded: std::sync::Arc<std::collections::HashSet<SharedString>>,
    tree: bool,
    full_keys: std::sync::Arc<Vec<SharedString>>,
    /// Behind an `Arc` because the list builders hold it across the frame.
    visible: std::sync::Arc<Vec<(usize, SharedString, VirtualTreeMetadata)>>,
    selectable_collection_keys: Option<std::sync::Arc<Vec<SharedString>>>,
    selectable_disabled_keys: Option<std::sync::Arc<Vec<SharedString>>>,
}

fn filtered_selectable_keys(
    full_keys: &[SharedString],
    disabled_keys: &[SharedString],
) -> std::sync::Arc<Vec<SharedString>> {
    std::sync::Arc::new(
        full_keys
            .iter()
            .filter(|key| !disabled_keys.contains(key))
            .cloned()
            .collect(),
    )
}

fn resolved_column_widths(
    columns: &[TableColumn],
    resized: &[Option<Pixels>],
    measured: &[Option<Pixels>],
    proposal: Option<(usize, Pixels)>,
) -> Vec<(SharedString, Pixels)> {
    columns
        .iter()
        .enumerate()
        .filter_map(|(index, column)| {
            let width = proposal
                .filter(|(proposed_index, _)| *proposed_index == index)
                .map(|(_, width)| width)
                .or(column.width)
                .or_else(|| resized.get(index).copied().flatten())
                .or(column.default_width)
                .or_else(|| measured.get(index).copied().flatten())?;
            let width = if column.allows_resizing {
                let min = column.min_width.map_or(DEFAULT_COLUMN_MIN_WIDTH, f32::from);
                let max = column.max_width.map_or(f32::MAX, f32::from);
                px(f32::from(width).floor().min(max).max(min))
            } else {
                width
            };
            Some((column.label.clone(), width))
        })
        .collect()
}

fn final_column_widths(
    columns: &[TableColumn],
    resized: &[Option<Pixels>],
    measured: &[Option<Pixels>],
    column: usize,
) -> Vec<(SharedString, Pixels)> {
    let proposal = resized
        .get(column)
        .copied()
        .flatten()
        .map(|width| (column, width));
    resolved_column_widths(columns, resized, measured, proposal)
}

fn clear_controlled_resize_proposal(
    columns: &[TableColumn],
    resized: &gpui::Entity<Vec<Option<Pixels>>>,
    column: usize,
    cx: &mut App,
) {
    if columns
        .get(column)
        .is_some_and(|column| column.width.is_some())
    {
        resized.update(cx, |values, cx| {
            if values
                .get_mut(column)
                .is_some_and(|width| width.take().is_some())
            {
                cx.notify();
            }
        });
    }
}

impl RenderOnce for Table {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Column widths a resize handle has moved, and the drag in progress.
        // `use_keyed_state` takes `cx` mutably, so both precede the tokens.
        let resized = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-resized", self.id).into()),
            cx,
            |_, _| Vec::<Option<Pixels>>::new(),
        );
        let resized_now = resized.read(cx).clone();
        let measured_widths = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-measured-widths", self.id).into()),
            cx,
            |_, _| Vec::<Option<Pixels>>::new(),
        );
        let measured_widths_now = measured_widths.read(cx).clone();
        let dragging = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-resizing", self.id).into()),
            cx,
            |_, _| None::<(usize, f32, f32)>,
        );
        let drag_now = *dragging.read(cx);
        let keyboard_resizing = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-keyboard-resizing", self.id).into()),
            cx,
            |_, _| None::<usize>,
        );
        let keyboard_resize_now = *keyboard_resizing.read(cx);
        let load_more_state = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-load-more-state", self.id).into()),
            cx,
            |_, _| (false, None::<LoadMoreCollection>),
        );
        // The body is one tab stop with a cursor inside it, the way a list is.
        let table_focus = crate::util::tab_stop_handle(
            gpui::ElementId::Name(format!("{}-focus", self.id).into()),
            window,
            cx,
        );
        let row_cursor = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-cursor", self.id).into()),
            cx,
            |_, _| None::<SharedString>,
        );
        let selection_range = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-selection-range", self.id).into()),
            cx,
            |_, _| TableSelectionRange::default(),
        );
        let typeahead = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-typeahead", self.id).into()),
            cx,
            |_, _| TableTypeahead::default(),
        );
        let (selected_keys, selection_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{}-selected", self.id).into()),
            self.is_selection_controlled
                .then(|| self.selected_keys.clone()),
            Vec::new(),
        );
        self.selected_keys = selected_keys;
        let needs_selectable_keys = self.selection_mode == SelectionMode::Multiple;
        let virtual_projection: Option<std::sync::Arc<VirtualProjection>> = self
            .virtual_rows
            .as_ref()
            .map(|(count, identity, key_for_row, _)| {
                let cache = window.use_keyed_state(
                    gpui::ElementId::Name(format!("{}-virtual-projection", self.id).into()),
                    cx,
                    |_, _| None::<std::sync::Arc<VirtualProjection>>,
                );
                let cached = cache.read(cx).clone();
                let tree = self.virtual_tree_metadata.is_some();
                match cached {
                    Some(cached)
                        if cached.count == *count
                            && cached.identity == *identity
                            && cached.tree == tree
                            && (!tree
                                || (cached.expanded.len() == self.expanded_keys.len()
                                    && self
                                        .expanded_keys
                                        .iter()
                                        .all(|key| cached.expanded.contains(key)))) =>
                    {
                        if !needs_selectable_keys
                            || cached
                                .selectable_disabled_keys
                                .as_ref()
                                .is_some_and(|disabled| {
                                    disabled.as_slice() == self.disabled_keys.as_slice()
                                })
                        {
                            cached
                        } else {
                            let projection = std::sync::Arc::new(VirtualProjection {
                                count: cached.count,
                                identity: cached.identity.clone(),
                                expanded: std::sync::Arc::clone(&cached.expanded),
                                tree: cached.tree,
                                full_keys: std::sync::Arc::clone(&cached.full_keys),
                                visible: std::sync::Arc::clone(&cached.visible),
                                selectable_collection_keys: Some(filtered_selectable_keys(
                                    cached.full_keys.as_slice(),
                                    &self.disabled_keys,
                                )),
                                selectable_disabled_keys: Some(std::sync::Arc::new(
                                    self.disabled_keys.clone(),
                                )),
                            });
                            cache.update(cx, |slot, _| {
                                *slot = Some(std::sync::Arc::clone(&projection));
                            });
                            projection
                        }
                    }
                    _ => {
                        let mut full_keys = Vec::with_capacity(*count);
                        let mut visible = Vec::with_capacity(*count);
                        if let Some(project) = &self.virtual_tree_metadata {
                            let mut visible_by_key =
                                std::collections::HashMap::with_capacity(*count);
                            for source_index in 0..*count {
                                let key = key_for_row(source_index);
                                let metadata = project(source_index);
                                let is_visible =
                                    metadata.parent_key.as_ref().is_none_or(|parent| {
                                        visible_by_key.get(parent).copied().unwrap_or(false)
                                            && self.expanded_keys.contains(parent)
                                    });
                                visible_by_key.insert(key.clone(), is_visible);
                                full_keys.push(key.clone());
                                if is_visible {
                                    visible.push((source_index, key, metadata));
                                }
                            }
                        } else {
                            for source_index in 0..*count {
                                let key = key_for_row(source_index);
                                full_keys.push(key.clone());
                                visible.push((source_index, key, VirtualTreeMetadata::default()));
                            }
                        }
                        let full_keys = std::sync::Arc::new(full_keys);
                        let selectable_collection_keys = needs_selectable_keys.then(|| {
                            filtered_selectable_keys(full_keys.as_slice(), &self.disabled_keys)
                        });
                        let selectable_disabled_keys = needs_selectable_keys
                            .then(|| std::sync::Arc::new(self.disabled_keys.clone()));
                        let projection = std::sync::Arc::new(VirtualProjection {
                            count: *count,
                            identity: identity.clone(),
                            expanded: if tree {
                                std::sync::Arc::new(self.expanded_keys.iter().cloned().collect())
                            } else {
                                std::sync::Arc::default()
                            },
                            tree,
                            full_keys,
                            visible: std::sync::Arc::new(visible),
                            selectable_collection_keys,
                            selectable_disabled_keys,
                        });
                        cache.update(cx, |slot, _| {
                            *slot = Some(std::sync::Arc::clone(&projection));
                        });
                        projection
                    }
                }
            });
        let virtual_visible_count = virtual_projection
            .as_ref()
            .map_or(0, |projection| projection.visible.len());
        let virtual_visible_keys = virtual_projection.as_ref().map(|projection| {
            projection
                .visible
                .iter()
                .map(|(_, key, _)| key.clone())
                .collect::<Vec<_>>()
        });
        let virtual_row_heights = match (
            &self.virtual_rows,
            self.estimated_row_height,
            &virtual_visible_keys,
        ) {
            (Some((_, identity, _, _)), Some(_), Some(keys)) if self.row_height.is_none() => {
                let count = keys.len();
                let state = window.use_keyed_state(
                    gpui::ElementId::Name(format!("{}-row-heights-{identity}", self.id).into()),
                    cx,
                    |_, _| (Vec::<SharedString>::new(), Vec::<Option<Pixels>>::new()),
                );
                if state.read(cx).0.as_slice() != keys.as_slice() {
                    let stored_keys = keys.clone();
                    state.update(cx, |stored, _| {
                        *stored = (stored_keys, vec![None; count]);
                    });
                }
                Some(state)
            }
            _ => None,
        };
        let virtual_scroll = window.use_keyed_state(
            gpui::ElementId::Name(format!("{}-virtual-scroll", self.id).into()),
            cx,
            |_, _| gpui::UniformListScrollHandle::new(),
        );
        let virtual_scroll_now = virtual_scroll.read(cx).clone();
        let load_more_virtual_scroll = (self.row_height.is_some() && self.virtual_rows.is_some())
            .then(|| virtual_scroll_now.clone());
        let virtual_list_state = match (
            self.row_height,
            self.estimated_row_height,
            &self.virtual_rows,
        ) {
            (None, Some(estimate), Some((_, identity, _, _))) => {
                let count = virtual_visible_count;
                let overdraw = self.max_h.unwrap_or(px(400.)).max(estimate * 3.);
                let state = window
                    .use_keyed_state(
                        gpui::ElementId::Name(format!("{}-list-state-{identity}", self.id).into()),
                        cx,
                        move |_, _| gpui::ListState::new(count, gpui::ListAlignment::Top, overdraw),
                    )
                    .read(cx)
                    .clone();
                if state.item_count() != count {
                    state.reset(count);
                }
                Some(state)
            }
            _ => None,
        };
        let load_more_variable_scroll = virtual_list_state
            .clone()
            .zip(self.estimated_row_height)
            .map(|(state, estimate)| (state, virtual_visible_count, estimate));
        // A sortable header had a click listener and no focus, so sorting was
        // mouse-only. v3's grid roves one tab stop across its cells; this port
        // gives each sortable header its own stop, which is the part that
        // matters. Created before the theme: `use_keyed_state` takes `cx`
        // mutably and `cx.colors()` holds a borrow.
        let sortable = self.on_sort_change.is_some();
        // Every column header is focusable, not just the sortable ones: v3's
        // PageUp leaves the body for the first column header whatever it is.
        // Only a sortable header is a *tab stop*, though -- it has to be, so
        // Enter and Space can sort it -- which keeps every table's Tab order
        // exactly as it was while giving the keyboard somewhere to land.
        let header_focus: Vec<gpui::FocusHandle> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                let id = gpui::ElementId::Name(format!("{}-sort-{i}-focus", self.id).into());
                if column.allows_sorting && sortable {
                    crate::util::tab_stop_handle(id, window, cx)
                } else {
                    window
                        .use_keyed_state(id, cx, |_, cx| cx.focus_handle())
                        .read(cx)
                        .clone()
                }
            })
            .collect();
        let ring_visible = crate::util::focus_visible(cx);
        let header_focused: Vec<bool> = header_focus
            .iter()
            .map(|h| h.is_focused(window) && ring_visible)
            .collect();
        let resize_focus: Vec<Option<gpui::FocusHandle>> = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                column.allows_resizing.then(|| {
                    crate::util::tab_stop_handle(
                        gpui::ElementId::Name(format!("{}-resize-{i}-focus", self.id).into()),
                        window,
                        cx,
                    )
                })
            })
            .collect();
        let resize_focused: Vec<bool> = resize_focus
            .iter()
            .map(|h| h.as_ref().is_some_and(|h| h.is_focused(window)) && ring_visible)
            .collect();
        if keyboard_resize_now.is_some_and(|column_index| {
            resize_focus
                .get(column_index)
                .and_then(Option::as_ref)
                .is_none()
        }) {
            keyboard_resizing.update(cx, |active, cx| {
                *active = None;
                cx.notify();
            });
        }

        let resizable = self.columns.iter().any(|c| c.allows_resizing);
        let resize_limits: Vec<(f32, f32)> = self
            .columns
            .iter()
            .map(|column| {
                (
                    column.min_width.map_or(DEFAULT_COLUMN_MIN_WIDTH, f32::from),
                    column.max_width.map_or(f32::MAX, f32::from),
                )
            })
            .collect();
        let effective_widths: Vec<Option<Pixels>> = self
            .columns
            .iter()
            .enumerate()
            .map(|(column_index, column)| {
                let width = column.width.or_else(|| {
                    resized_now
                        .get(column_index)
                        .copied()
                        .flatten()
                        .or(column.default_width)
                });
                if column.allows_resizing {
                    let (min, max) = resize_limits[column_index];
                    width.map(|width| px(f32::from(width).floor().min(max).max(min)))
                } else {
                    width
                }
            })
            .collect();
        let layout_width_bounds: Vec<(Option<Pixels>, Option<Pixels>)> = self
            .columns
            .iter()
            .enumerate()
            .map(|(column_index, column)| {
                if column.allows_resizing {
                    if effective_widths[column_index].is_some() {
                        (None, None)
                    } else {
                        (Some(px(resize_limits[column_index].0)), column.max_width)
                    }
                } else {
                    (column.min_width, column.max_width)
                }
            })
            .collect();
        let resize_columns = std::sync::Arc::new(self.columns.clone());
        let resize_measurements = std::sync::Arc::new(measured_widths_now.clone());
        let colors = cx.colors();
        // Copies of the tokens the tail needs: the row builder borrows `cx`
        // mutably, which ends the borrow `cx.colors()` holds.
        let muted = colors.muted;
        let secondary = self.variant == TableVariant::Secondary;
        let selectable = self.selection_mode != SelectionMode::None;
        // Only multiple selection consumes the full enabled-key set. The
        // virtual projection shares it between frames; None and Single do not
        // materialize a collection that no consumer reads.
        let non_virtual_keys = virtual_projection.is_none().then(|| self.row_keys());
        let selectable_collection_keys: Option<std::sync::Arc<Vec<SharedString>>> =
            needs_selectable_keys.then(|| match virtual_projection.as_ref() {
                Some(projection) => projection
                    .selectable_collection_keys
                    .as_ref()
                    .expect("multiple virtual Tables cache selectable keys")
                    .clone(),
                None => filtered_selectable_keys(
                    non_virtual_keys
                        .as_ref()
                        .expect("multiple non-virtual Tables have row keys")
                        .as_slice(),
                    &self.disabled_keys,
                ),
            });
        let load_more_collection = match &self.virtual_rows {
            Some((count, identity, _, _)) => LoadMoreCollection::Virtual {
                count: *count,
                identity: identity.clone(),
            },
            None => LoadMoreCollection::Rows(non_virtual_keys.unwrap_or_default()),
        };

        let mut wrapper = gpui::div()
            .w_full()
            .track_focus(&table_focus)
            .overflow_hidden()
            .rounded(crate::util::container_radius(cx))
            .text_color(colors.foreground);

        // `.table-root--primary` is a `bg-surface-secondary px-1 pb-1` tray with
        // `border-radius: min(32px, --radius * 2.5)`, and the rows sit in a
        // `bg-surface` block inside it; `secondary` is flat. This used to draw
        // one white card with a border, which is the block without its tray.
        if !secondary {
            let radius = cx.layout().radius_lg() * 2.5;
            wrapper = wrapper
                .bg(colors.surface_secondary)
                .rounded(radius.min(px(32.)))
                .px(px(4.))
                .pb(px(4.));
        }

        // The content column, whose width is what the scroller at the bottom of
        // the render measures against. A `w_full` child commits to the
        // scroller's width, which is exactly the scroller's own -- the scroll
        // maxima are then zero and a wide table clips at the tray edge instead
        // of sliding. `min_w_full` keeps the column at the viewport when no
        // column pins a width, and `flex_shrink_0` keeps it at the columns'
        // width when they exceed the viewport (a shrinking row only ever fits).
        let mut table = gpui::div()
            .flex()
            .flex_col()
            .flex_shrink_0()
            .min_w_full()
            .text_size(px(14.))
            .line_height(px(20.))
            .when_some(self.gap, |el, g| el.gap(g))
            .when_some(self.padding, |el, p| el.p(p));

        // ---- header ------------------------------------------------------
        // `.table__header`, whose cells are `.table__column`s and whose
        // sortable ones wrap in `.table__sortable-column-header`.
        let mut header = gpui::div()
            .flex()
            .border_b_1()
            .border_color(colors.separator.alpha(0.5))
            .when(!secondary, |h| h.bg(colors.surface_secondary));

        if selectable {
            // The select-all box only makes sense for a multiple selection; a
            // single-selection table keeps the column for alignment.
            let mut cell = gpui::div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(44.))
                .py(px(10.));
            if self.selection_mode == SelectionMode::Multiple {
                // The `Mod+A` keydown handler reads the same set, so it gets a
                // clone rather than the variable itself.
                let all = selectable_collection_keys
                    .as_ref()
                    .expect("multiple Tables have selectable keys")
                    .clone();
                let (all_selected, indeterminate) =
                    select_all_flags(all.as_slice(), &self.selected_keys);
                let mut box_el = Checkbox::new(gpui::ElementId::Name(
                    format!("{}-select-all", self.id).into(),
                ))
                .is_selected(all_selected)
                .is_indeterminate(indeterminate);
                let cb = self.on_selection_change.clone();
                if cb.is_some() || selection_own.is_some() {
                    let selection_own = selection_own.clone();
                    let selection_range = selection_range.clone();
                    box_el = box_el.on_change(move |_next, window, cx| {
                        // Anything short of everything selects everything.
                        let next: Vec<SharedString> = if all_selected {
                            Vec::new()
                        } else {
                            all.as_slice().to_vec()
                        };
                        if let Some(held) = &selection_own {
                            held.update(cx, |value, cx| {
                                *value = next.clone();
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &cb {
                            cb(&next, window, cx);
                        }
                        selection_range.update(cx, |range, _| {
                            *range = if all_selected {
                                TableSelectionRange::default()
                            } else {
                                TableSelectionRange {
                                    is_all: true,
                                    ..TableSelectionRange::default()
                                }
                            };
                        });
                    });
                }
                cell = cell.child(box_el);
            }
            header = header.child(cell);
        }

        for (column_index, column) in self.columns.iter().enumerate() {
            let sorted = self
                .sort_descriptor
                .as_ref()
                .filter(|d| d.column == column.label);
            // A resized column keeps the width the drag left it at.
            let effective = effective_widths[column_index];
            let (layout_min, layout_max) = layout_width_bounds[column_index];
            let mut cell = gpui::div()
                .when(effective.is_none(), flex_cell)
                .when_some(effective, |c, w| c.w(w))
                .when_some(layout_min, |c, w| c.min_w(w))
                .when_some(layout_max, |c, w| c.max_w(w))
                .flex()
                .items_center()
                .gap(px(4.))
                // `.table__column` is `px-4 py-2.5 text-xs`.
                .px(px(16.))
                .py(px(10.))
                .text_size(px(12.))
                .line_height(px(16.))
                .font_weight(gpui::FontWeight::MEDIUM)
                .text_color(if sorted.is_some() {
                    colors.foreground
                } else {
                    colors.muted
                })
                .child(column.label.clone());

            if let Some(descriptor) = sorted {
                if self.show_indicator {
                    // `indicator` is v3's render prop on
                    // `Table.SortableColumnHeader`: it receives the direction
                    // the column is sorted in and replaces the chevron.
                    cell = cell.child(match &self.indicator {
                        Some(render) => render(descriptor.direction),
                        None => gpui::svg()
                            .size(px(12.))
                            .path(descriptor.direction.indicator())
                            // svg() never inherits text colour.
                            .text_color(colors.foreground)
                            .into_any_element(),
                    });
                }
            }

            // A sortable header wraps the cell in a click target, which is a
            // different element type, so both branches unify to AnyElement.
            let cell = match (column.allows_sorting, self.on_sort_change.clone()) {
                (true, Some(cb)) => {
                    let next =
                        SortDescriptor::next(self.sort_descriptor.as_ref(), column.label.clone());
                    // `.table__column[data-allows-sorting]:hover` recolours the
                    // header text to `--foreground`; it paints no background.
                    let sort_group: SharedString =
                        format!("table-sort-hover-{}", column.label).into();
                    let header_cell = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("table-sort-{}", column.label).into(),
                        ))
                        .group(sort_group.clone())
                        .flex_1()
                        .flex()
                        .cursor_pointer()
                        // The focus is what makes Enter and Space sort: gpui
                        // fires a *focused* element's click listeners for them.
                        .track_focus(&header_focus[column_index])
                        .on_click(move |_, window, cx| cb(next.clone(), window, cx))
                        .child(cell.group_hover(sort_group, |s| s.text_color(colors.foreground)));
                    // `.table__column` rings *inside* itself: the next column
                    // is flush against this one, and a ring drawn outside bled
                    // through the transparent cell and filled it.
                    header_cell
                        .relative()
                        .when(header_focused[column_index], |c| {
                            c.child(crate::util::inset_focus_ring(cx))
                        })
                        .into_any_element()
                }
                // Not sortable, so nothing to press -- but still focusable, so
                // PageUp has a header to land on, and it rings when it does.
                _ => cell
                    .id(gpui::ElementId::Name(
                        format!("table-header-{column_index}").into(),
                    ))
                    .track_focus(&header_focus[column_index])
                    .relative()
                    .when(header_focused[column_index], |c| {
                        c.child(crate::util::inset_focus_ring(cx))
                    })
                    .into_any_element(),
            };

            // `allowsResizing` puts a handle on the column's trailing edge. The
            // wrapper is what keeps the handle inside the column's box.
            // `.table__resizable-container` is the box that keeps the handle
            // inside the column, which is what this wrapper is.
            let cell = if column.allows_resizing {
                let held = dragging.clone();
                let start_width = effective
                    .or_else(|| measured_widths_now.get(column_index).copied().flatten())
                    .unwrap_or(px(160.));
                let keyboard = keyboard_resizing.clone();
                let keyboard_out = keyboard.clone();
                let keyboard_for_pointer = keyboard.clone();
                let widths = resized.clone();
                let widths_for_pointer = widths.clone();
                let widths_for_outside = widths.clone();
                let (min_width, max_width) = resize_limits[column_index];
                let controlled = column.width.is_some();
                let focus_for_mouse = resize_focus[column_index]
                    .as_ref()
                    .expect("resizable columns have a focus handle")
                    .clone();
                let resizer_group: SharedString = format!("table-resizer-{column_index}").into();
                let accent_color = colors.accent.color;
                let focus_color = colors.focus;
                let is_resizing = drag_now.is_some_and(|(index, _, _)| index == column_index)
                    || keyboard_resize_now == Some(column_index);
                let measured = measured_widths.clone();
                let columns_for_pointer = resize_columns.clone();
                let columns_for_outside = resize_columns.clone();
                let columns_for_keys = resize_columns.clone();
                let measurements_for_pointer = resize_measurements.clone();
                let measurements_for_outside = resize_measurements.clone();
                let measurements_for_keys = resize_measurements.clone();
                let resize_start_for_pointer = self.on_resize_start.clone();
                let resize_start_for_keys = self.on_resize_start.clone();
                let resize_for_keys = self.on_resize.clone();
                let resize_end_for_pointer = self.on_resize_end.clone();
                let resize_end_for_outside = self.on_resize_end.clone();
                let resize_end_for_keys = self.on_resize_end.clone();
                gpui::div()
                    .relative()
                    .when(effective.is_none(), flex_cell)
                    .when_some(effective, |c, w| c.w(w))
                    .when(effective.is_none(), |wrapper| {
                        wrapper.child(
                            gpui::canvas(
                                move |bounds: gpui::Bounds<Pixels>, _, cx| {
                                    let width = px(f32::from(bounds.size.width).floor());
                                    measured.update(cx, |values, cx| {
                                        if values.len() <= column_index {
                                            values.resize(column_index + 1, None);
                                        }
                                        if values[column_index] != Some(width) {
                                            values[column_index] = Some(width);
                                            cx.notify();
                                        }
                                    });
                                    bounds
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        )
                    })
                    .child(cell)
                    .child(
                        gpui::div()
                            .id(gpui::ElementId::Name(
                                format!("table-resize-{column_index}").into(),
                            ))
                            .track_focus(
                                resize_focus[column_index]
                                    .as_ref()
                                    .expect("resizable columns have a focus handle"),
                            )
                            .group(resizer_group.clone())
                            .absolute()
                            .top(px(0.))
                            // `px-2` around a `w-px` line, `box-content`: an
                            // 8px grab margin either side of the column edge.
                            .right(px(-8.))
                            .w(px(17.))
                            .h_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                // `h-4 w-px rounded-sm bg-separator`, and
                                // `h-full w-0.5 bg-accent` while hovered.
                                gpui::div()
                                    .w(px(1.))
                                    .h(px(16.))
                                    .rounded(crate::util::hairline_radius(cx))
                                    .bg(colors.separator)
                                    .group_hover(resizer_group.clone(), |s| {
                                        s.w(px(2.)).h_full().bg(accent_color)
                                    })
                                    .when(is_resizing, |s| {
                                        s.w(px(2.)).h_full().bg(accent_color)
                                    })
                                    .when(resize_focused[column_index], |s| {
                                        s.w(px(2.)).h_full().bg(focus_color)
                                    }),
                            )
                            .cursor(gpui::CursorStyle::ResizeLeftRight)
                            .on_mouse_down(gpui::MouseButton::Left, move |ev, window, cx| {
                                window.focus(&focus_for_mouse, cx);
                                if let Some(active_column) = *keyboard_for_pointer.read(cx) {
                                    let current = final_column_widths(
                                        &columns_for_pointer,
                                        widths_for_pointer.read(cx),
                                        &measurements_for_pointer,
                                        active_column,
                                    );
                                    keyboard_for_pointer.update(cx, |active, _| *active = None);
                                    clear_controlled_resize_proposal(
                                        &columns_for_pointer,
                                        &widths_for_pointer,
                                        active_column,
                                        cx,
                                    );
                                    if let Some(callback) = &resize_end_for_pointer {
                                        callback(&current, window, cx);
                                    }
                                }
                                clear_controlled_resize_proposal(
                                    &columns_for_pointer,
                                    &widths_for_pointer,
                                    column_index,
                                    cx,
                                );
                                if let Some(callback) = &resize_start_for_pointer {
                                    let current = resolved_column_widths(
                                        &columns_for_pointer,
                                        widths_for_pointer.read(cx),
                                        &measurements_for_pointer,
                                        None,
                                    );
                                    callback(&current, window, cx);
                                }
                                let x = f32::from(ev.position.x);
                                held.update(cx, |v, _| {
                                    *v = Some((column_index, x, f32::from(start_width)));
                                });
                            })
                            .on_mouse_down_out(move |_, window, cx| {
                                if *keyboard_out.read(cx) == Some(column_index) {
                                    let current = final_column_widths(
                                        &columns_for_outside,
                                        widths_for_outside.read(cx),
                                        &measurements_for_outside,
                                        column_index,
                                    );
                                    keyboard_out.update(cx, |active, cx| {
                                        *active = None;
                                        cx.notify();
                                    });
                                    clear_controlled_resize_proposal(
                                        &columns_for_outside,
                                        &widths_for_outside,
                                        column_index,
                                        cx,
                                    );
                                    if let Some(callback) = &resize_end_for_outside {
                                        callback(&current, window, cx);
                                    }
                                }
                            })
                            .on_key_down(move |event, window, cx| {
                                let key = event.keystroke.key.as_str();
                                let editing = *keyboard.read(cx) == Some(column_index);
                                match key {
                                    "enter" => {
                                        if editing {
                                            let current = final_column_widths(
                                                &columns_for_keys,
                                                widths.read(cx),
                                                &measurements_for_keys,
                                                column_index,
                                            );
                                            keyboard.update(cx, |active, cx| {
                                                *active = None;
                                                cx.notify();
                                            });
                                            clear_controlled_resize_proposal(
                                                &columns_for_keys,
                                                &widths,
                                                column_index,
                                                cx,
                                            );
                                            if let Some(callback) = &resize_end_for_keys {
                                                callback(&current, window, cx);
                                            }
                                        } else {
                                            clear_controlled_resize_proposal(
                                                &columns_for_keys,
                                                &widths,
                                                column_index,
                                                cx,
                                            );
                                            let current = resolved_column_widths(
                                                &columns_for_keys,
                                                widths.read(cx),
                                                &measurements_for_keys,
                                                None,
                                            );
                                            keyboard.update(cx, |active, cx| {
                                                *active = Some(column_index);
                                                cx.notify();
                                            });
                                            if let Some(callback) = &resize_start_for_keys {
                                                callback(&current, window, cx);
                                            }
                                        }
                                        cx.stop_propagation();
                                    }
                                    "escape" | "space" | "tab" if editing => {
                                        let current = final_column_widths(
                                            &columns_for_keys,
                                            widths.read(cx),
                                            &measurements_for_keys,
                                            column_index,
                                        );
                                        keyboard.update(cx, |active, cx| {
                                            *active = None;
                                            cx.notify();
                                        });
                                        clear_controlled_resize_proposal(
                                            &columns_for_keys,
                                            &widths,
                                            column_index,
                                            cx,
                                        );
                                        if let Some(callback) = &resize_end_for_keys {
                                            callback(&current, window, cx);
                                        }
                                        cx.stop_propagation();
                                    }
                                    "right" | "up" | "left" | "down" if editing => {
                                        let delta = if matches!(key, "right" | "up") {
                                            10.
                                        } else {
                                            -10.
                                        };
                                        let mut proposed = start_width;
                                        widths.update(cx, |values, cx| {
                                            if values.len() <= column_index {
                                                values.resize(column_index + 1, None);
                                            }
                                            let current = if controlled {
                                                start_width
                                            } else {
                                                values[column_index].unwrap_or(start_width)
                                            };
                                            let next = (f32::from(current) + delta)
                                                .floor()
                                                .min(max_width)
                                                .max(min_width);
                                            proposed = px(next);
                                            values[column_index] = Some(proposed);
                                            cx.notify();
                                        });
                                        if let Some(callback) = &resize_for_keys {
                                            let current = resolved_column_widths(
                                                &columns_for_keys,
                                                widths.read(cx),
                                                &measurements_for_keys,
                                                Some((column_index, proposed)),
                                            );
                                            callback(&current, window, cx);
                                        }
                                        cx.stop_propagation();
                                    }
                                    _ => {}
                                }
                            }),
                    )
                    .into_any_element()
            } else {
                cell
            };
            header = header.child(cell);
        }
        table = table.child(header);

        // `.table__body` rounds to `min(32px, --radius-2xl)` and its cells are
        // `bg-surface`: the white block inside the tray.
        let mut body = gpui::div().flex().flex_col().w_full();
        if !secondary {
            body = body
                .bg(colors.surface.background)
                .rounded(cx.layout().radius_2xl().min(px(32.)))
                .overflow_hidden();
        }

        // The drag itself: the pointer can leave the table, so paint-time
        // window listeners own the move and release until the drag ends.
        if resizable {
            let held = dragging.clone();
            let held_up = dragging;
            let widths = resized;
            let widths_up = widths.clone();
            let columns_for_move = resize_columns.clone();
            let columns_for_up = resize_columns;
            let measurements_for_move = resize_measurements.clone();
            let measurements_for_up = resize_measurements;
            let resize_callback = self.on_resize.clone();
            let resize_end_callback = self.on_resize_end.clone();
            table = table.relative().child(
                gpui::canvas(
                    |bounds, _, _| bounds,
                    move |_, _, window, _| {
                        let held = held.clone();
                        let widths = widths.clone();
                        let columns_for_move = columns_for_move.clone();
                        let measurements_for_move = measurements_for_move.clone();
                        let resize_callback = resize_callback.clone();
                        window.on_mouse_event(
                            move |event: &gpui::MouseMoveEvent, phase, window, cx| {
                                if phase != gpui::DispatchPhase::Capture
                                    || event.pressed_button != Some(gpui::MouseButton::Left)
                                {
                                    return;
                                }
                                let Some((column, from_x, from_w)) = *held.read(cx) else {
                                    return;
                                };
                                let raw = (from_w + f32::from(event.position.x) - from_x).floor();
                                let min = columns_for_move[column]
                                    .min_width
                                    .map_or(DEFAULT_COLUMN_MIN_WIDTH, f32::from);
                                let max = columns_for_move[column]
                                    .max_width
                                    .map_or(f32::MAX, f32::from);
                                let proposed = px(raw.min(max).max(min));
                                widths.update(cx, |values, cx| {
                                    if values.len() <= column {
                                        values.resize(column + 1, None);
                                    }
                                    values[column] = Some(proposed);
                                    cx.notify();
                                });
                                if let Some(callback) = &resize_callback {
                                    let current = resolved_column_widths(
                                        &columns_for_move,
                                        widths.read(cx),
                                        &measurements_for_move,
                                        Some((column, proposed)),
                                    );
                                    callback(&current, window, cx);
                                }
                            },
                        );

                        let held_up = held_up.clone();
                        let widths_up = widths_up.clone();
                        let columns_for_up = columns_for_up.clone();
                        let measurements_for_up = measurements_for_up.clone();
                        let resize_end_callback = resize_end_callback.clone();
                        window.on_mouse_event(
                            move |event: &gpui::MouseUpEvent, phase, window, cx| {
                                if phase != gpui::DispatchPhase::Capture
                                    || event.button != gpui::MouseButton::Left
                                {
                                    return;
                                }
                                let drag = *held_up.read(cx);
                                if let Some((column, _, _)) = drag {
                                    let current = final_column_widths(
                                        &columns_for_up,
                                        widths_up.read(cx),
                                        &measurements_for_up,
                                        column,
                                    );
                                    held_up.update(cx, |value, cx| {
                                        *value = None;
                                        cx.notify();
                                    });
                                    clear_controlled_resize_proposal(
                                        &columns_for_up,
                                        &widths_up,
                                        column,
                                        cx,
                                    );
                                    if let Some(callback) = &resize_end_callback {
                                        callback(&current, window, cx);
                                    }
                                }
                            },
                        );
                    },
                )
                .absolute()
                .inset_0(),
            );
        }
        // ---- rows --------------------------------------------------------
        // Depth-first, and only through the parents that are open: a nested row
        // is not rendered at all until its parent is expanded.
        let mut flat: Vec<(TableRow, usize, bool, SharedString, Option<SharedString>)> = Vec::new();
        fn flatten(
            rows: Vec<TableRow>,
            depth: usize,
            path: &str,
            parent: Option<&SharedString>,
            expanded: &[SharedString],
            out: &mut Vec<(TableRow, usize, bool, SharedString, Option<SharedString>)>,
        ) {
            for (index, mut row) in rows.into_iter().enumerate() {
                let key = match &row.key {
                    Some(k) => k.clone(),
                    None if path.is_empty() => SharedString::from(index.to_string()),
                    None => SharedString::from(format!("{path}-{index}")),
                };
                let children = std::mem::take(&mut row.children);
                let has_children = !children.is_empty();
                let is_open = expanded.contains(&key);
                out.push((row, depth, has_children, key.clone(), parent.cloned()));
                if has_children && is_open {
                    flatten(children, depth + 1, key.as_ref(), Some(&key), expanded, out);
                }
            }
        }
        flatten(
            std::mem::take(&mut self.rows),
            0,
            "",
            None,
            &self.expanded_keys,
            &mut flat,
        );
        let tree_rows: Vec<(bool, Option<SharedString>)> = virtual_projection.as_ref().map_or_else(
            || {
                flat.iter()
                    .map(|(_, _, has_children, _, parent)| (*has_children, parent.clone()))
                    .collect()
            },
            |projection| {
                projection
                    .visible
                    .iter()
                    .map(|(_, _, metadata)| (metadata.has_children, metadata.parent_key.clone()))
                    .collect()
            },
        );
        let visible_collection_keys: Vec<SharedString> = virtual_visible_keys
            .unwrap_or_else(|| flat.iter().map(|(_, _, _, key, _)| key.clone()).collect());
        let typeahead_labels: Vec<String> = virtual_projection.as_ref().map_or_else(
            || {
                flat.iter()
                    .map(|(row, _, _, _, _)| {
                        row.text_value
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_default()
                    })
                    .collect()
            },
            |_| Vec::new(),
        );
        let virtual_typeahead_indices = virtual_projection.as_ref().map(|projection| {
            projection
                .visible
                .iter()
                .map(|(source_index, _, _)| *source_index)
                .collect::<Vec<_>>()
        });
        let virtual_text_value = self.virtual_text_value.clone();
        let cursor_key = row_cursor.read(cx).clone();
        let cursor_valid = cursor_key.as_ref().is_none_or(|key| {
            visible_collection_keys.contains(key) && !self.disabled_keys.contains(key)
        });
        if !cursor_valid {
            row_cursor.update(cx, |key, cx| {
                *key = None;
                cx.notify();
            });
        }
        let cursor_at = if window.is_window_active() && table_focus.is_focused(window) {
            cursor_key
                .as_ref()
                .and_then(|key| visible_collection_keys.iter().position(|row| row == key))
                .filter(|index| {
                    !self
                        .disabled_keys
                        .contains(&visible_collection_keys[*index])
                })
                .filter(|_| crate::util::focus_visible(cx))
        } else {
            None
        };
        // Whether any row in this table is expandable at all: a flat table
        // keeps its cells flush, rather than reserving a chevron's width.
        let tree_column_has_children = virtual_projection.as_ref().map_or_else(
            || flat.iter().any(|(_, _, has, _, _)| *has),
            |projection| {
                projection
                    .visible
                    .iter()
                    .any(|(_, _, metadata)| metadata.has_children)
            },
        );
        let expanded_keys = std::rc::Rc::new(std::mem::take(&mut self.expanded_keys));

        // Everything a row needs, held once. The virtual path builds its rows
        // inside a `'static` closure, so it cannot borrow `self` -- and having
        // one row builder for both paths is what keeps a virtual table drawing
        // the same row as a short one.
        let table_id = self.id.clone();
        let ctx = std::rc::Rc::new(RowCtx {
            id: self.id.clone(),
            widths: self
                .columns
                .iter()
                .enumerate()
                .map(|(c, _)| {
                    let (layout_min, layout_max) = layout_width_bounds[c];
                    (
                        effective_widths.get(c).copied().flatten(),
                        layout_min,
                        layout_max,
                    )
                })
                .collect(),
            row_header_columns: self.columns.iter().map(|c| c.is_row_header).collect(),
            tree_column: self.tree_column,
            tree_column_has_children,
            selectable,
            selection_mode: self.selection_mode,
            selected_keys: self.selected_keys.clone(),
            row_keys: visible_collection_keys,
            disabled_keys: self.disabled_keys.clone(),
            expanded: expanded_keys.clone(),
            on_expanded_change: self.on_expanded_change.clone(),
            on_selection_change: self.on_selection_change.clone(),
            selection_own: selection_own.clone(),
            selection_range: selection_range.clone(),
            on_row_click: self.on_row_click.clone(),
            focus: table_focus.clone(),
            cursor_own: row_cursor.clone(),
            cursor: cursor_at,
            secondary,
        });

        // v3 gives a table a roving row focus: the arrows walk it, Home and End
        // jump, and Enter activates the row -- the same resolver every list here
        // uses, over the rows that exist.
        {
            let stops: Vec<usize> = ctx
                .row_keys
                .iter()
                .enumerate()
                .filter_map(|(index, key)| (!self.disabled_keys.contains(key)).then_some(index))
                .collect();
            let held = row_cursor;
            let typeahead = typeahead;
            let labels = typeahead_labels;
            let virtual_indices = virtual_typeahead_indices;
            let virtual_text = virtual_text_value;
            let on_row_click = self.on_row_click.clone();
            let keys = ctx.row_keys.clone();
            let table_focus_for_keys = table_focus;
            let selection = self.on_selection_change.clone();
            let selection_own_for_keys = selection_own;
            let selection_range_for_keys = selection_range;
            let selected_now = self.selected_keys.clone();
            let mode = self.selection_mode;
            let plain_rows = self.virtual_rows.is_none();
            let fixed_virtual = self.row_height.is_some() && self.virtual_rows.is_some();
            let fixed_page_step = self.row_height.filter(|_| fixed_virtual).map(|row_height| {
                let viewport_height = f32::from(self.max_h.unwrap_or(px(400.)));
                ((viewport_height / f32::from(row_height)).ceil() as usize).saturating_sub(1)
            });
            let fixed_scroll = virtual_scroll_now.clone();
            let variable_scroll = virtual_list_state.clone();
            let variable_heights = virtual_row_heights.clone();
            let variable_estimate = self.estimated_row_height;
            let expanded = expanded_keys;
            let on_expanded = self.on_expanded_change.clone();
            if !keys.is_empty() {
                let typeahead_navigation = std::rc::Rc::new(TableTypeaheadNavigation {
                    state: typeahead,
                    labels,
                    virtual_indices,
                    virtual_text,
                    stops: stops.clone(),
                    keys: keys.clone(),
                    cursor: held.clone(),
                    fixed_virtual,
                    fixed_scroll: fixed_scroll.clone(),
                    variable_scroll: variable_scroll.clone(),
                });
                let capture_typeahead = typeahead_navigation.clone();
                let capture_typeahead_up = typeahead_navigation.clone();
                let capture_focus = table_focus_for_keys.clone();
                let capture_focus_up = table_focus_for_keys.clone();
                wrapper = wrapper.capture_key_down(move |event, window, cx| {
                    if !capture_focus.contains_focused(window, cx) {
                        return;
                    }
                    let key_name = event.keystroke.key.as_str();
                    let typed = event.keystroke.key_char.as_deref().unwrap_or(key_name);
                    let modifiers = &event.keystroke.modifiers;
                    let now = web_time::Instant::now();
                    let is_space = key_name == "space" || typed == " ";
                    if is_space
                        && capture_typeahead.state.read(cx).is_active(now)
                        && !modifiers.control
                        && !modifiers.platform
                        && !modifiers.alt
                    {
                        cx.stop_propagation();
                        capture_typeahead.push(" ", now, false, cx);
                    }
                });
                wrapper = wrapper.capture_key_up(move |event, window, cx| {
                    if !capture_focus_up.contains_focused(window, cx) {
                        return;
                    }
                    let key_name = event.keystroke.key.as_str();
                    let typed = event.keystroke.key_char.as_deref().unwrap_or(key_name);
                    let modifiers = &event.keystroke.modifiers;
                    let is_space = key_name == "space" || typed == " ";
                    if is_space
                        && capture_typeahead_up
                            .state
                            .read(cx)
                            .is_active(web_time::Instant::now())
                        && !modifiers.control
                        && !modifiers.platform
                        && !modifiers.alt
                    {
                        cx.stop_propagation();
                    }
                });
                let key_typeahead = typeahead_navigation;
                // The header PageUp hands the focus to. Cloned out because the
                // handler outlives this frame's `header_focus`.
                let page_up_header = header_focus.first().cloned();
                let headers_for_keys = header_focus;
                wrapper = wrapper.on_key_down(move |event, window, cx| {
                    if !table_focus_for_keys.contains_focused(window, cx) {
                        return;
                    }
                    let from = held
                        .read(cx)
                        .as_ref()
                        .and_then(|key| keys.iter().position(|row| row == key))
                        .filter(|index| stops.contains(index));
                    // Pinned React Aria `useTableRow`: horizontal keys belong
                    // to the focused tree row before the list resolver sees
                    // them. Right opens; Left closes or returns to the parent.
                    let key_name = event.keystroke.key.as_str();
                    let typed = event.keystroke.key_char.as_deref().unwrap_or(key_name);
                    let modifiers = &event.keystroke.modifiers;
                    let now = web_time::Instant::now();
                    let is_space = key_name == "space" || typed == " ";
                    let is_character = {
                        let mut chars = key_name.chars();
                        chars.next().is_some() && chars.next().is_none() && !is_space
                    };
                    let is_typeahead =
                        is_character && !modifiers.control && !modifiers.platform && !modifiers.alt;
                    if is_typeahead {
                        if key_typeahead.push(typed, now, true, cx) {
                            cx.stop_propagation();
                        }
                        return;
                    }
                    // Pinned React Aria 3.51 `useSelectableCollection` binds
                    // `Mod+A` -- the platform Mod, Control here -- to
                    // `selectAll`, and only when the selection mode is
                    // multiple. The shortcut matches its modifiers exactly,
                    // so any extra modifier lets the event fall through.
                    if key_name == "a"
                        && event.keystroke.modifiers.secondary()
                        && !event.keystroke.modifiers.shift
                        && !event.keystroke.modifiers.alt
                        && !event.keystroke.modifiers.function
                        && if cfg!(target_os = "macos") {
                            !event.keystroke.modifiers.control
                        } else {
                            !event.keystroke.modifiers.platform
                        }
                        && mode == SelectionMode::Multiple
                    {
                        let selectable_collection_keys = selectable_collection_keys
                            .as_ref()
                            .expect("multiple Tables have selectable keys");
                        let selectable_collection_keys = selectable_collection_keys.clone();
                        let selectable_collection_keys = selectable_collection_keys.as_slice();
                        let (all_selected, _) =
                            select_all_flags(selectable_collection_keys, &selected_now);
                        let materializes_all =
                            same_selection(selectable_collection_keys, &selected_now);
                        let already_all = all_selected
                            || (selection_range_for_keys.read(cx).is_all && materializes_all);
                        // Pinned React Stately's `selectAll` is idempotent once
                        // the whole selectable collection is already selected.
                        if !already_all {
                            let next = selectable_collection_keys.to_vec();
                            if let Some(held) = &selection_own_for_keys {
                                held.update(cx, |value, cx| {
                                    *value = next.clone();
                                    cx.notify();
                                });
                            }
                            if let Some(cb) = &selection {
                                cb(&next, window, cx);
                            }
                            selection_range_for_keys.update(cx, |range, _| {
                                *range = TableSelectionRange {
                                    is_all: true,
                                    ..TableSelectionRange::default()
                                };
                            });
                        }
                        cx.stop_propagation();
                        return;
                    }
                    // Other collection keys belong only to the body's roving
                    // focus stop. A nested cell action must keep its own Enter
                    // and Space handling even though Mod+A bubbles to the root.
                    if plain_rows
                        && headers_for_keys
                            .iter()
                            .any(|header| header.is_focused(window))
                    {
                        let next = match key_name {
                            "down" => stops.first(),
                            "pagedown" => stops.last(),
                            _ => None,
                        };
                        if let Some(next) = next {
                            held.update(cx, |value, cx| {
                                *value = Some(keys[*next].clone());
                                cx.notify();
                            });
                            window.focus(&table_focus_for_keys, cx);
                            cx.stop_propagation();
                        }
                        return;
                    }
                    if !table_focus_for_keys.is_focused(window) {
                        return;
                    }
                    if key_name == "escape"
                        && mode != SelectionMode::None
                        && !selected_now.is_empty()
                    {
                        let next = Vec::new();
                        if let Some(held) = &selection_own_for_keys {
                            held.update(cx, |value, cx| {
                                *value = next.clone();
                                cx.notify();
                            });
                        }
                        if let Some(cb) = &selection {
                            cb(&next, window, cx);
                        }
                        selection_range_for_keys.update(cx, |range, _| {
                            *range = TableSelectionRange::default();
                        });
                        cx.stop_propagation();
                        return;
                    }
                    if let Some(index) = from {
                        let focused_key = &keys[index];
                        if let Some((has_children, parent)) = tree_rows.get(index) {
                            if key_name == "right"
                                && *has_children
                                && !expanded.contains(focused_key)
                            {
                                if let Some(cb) = &on_expanded {
                                    let mut next = expanded.as_ref().clone();
                                    next.push(focused_key.clone());
                                    cb(&next, window, cx);
                                }
                                crate::util::set_focus_visible(true, cx);
                                cx.stop_propagation();
                                return;
                            } else if key_name == "left" {
                                if *has_children && expanded.contains(focused_key) {
                                    if let Some(cb) = &on_expanded {
                                        let mut next = expanded.as_ref().clone();
                                        next.retain(|key| key != focused_key);
                                        cb(&next, window, cx);
                                    }
                                    crate::util::set_focus_visible(true, cx);
                                    cx.stop_propagation();
                                    return;
                                } else if let Some(parent) = parent {
                                    if let Some(parent_index) =
                                        keys.iter().position(|key| key == parent)
                                    {
                                        held.update(cx, |value, cx| {
                                            *value = Some(parent.clone());
                                            cx.notify();
                                        });
                                        if fixed_virtual {
                                            fixed_scroll.scroll_to_item(
                                                parent_index,
                                                gpui::ScrollStrategy::Center,
                                            );
                                        } else if let Some(state) = &variable_scroll {
                                            state.scroll_to(gpui::ListOffset {
                                                item_ix: parent_index,
                                                offset_in_item: px(0.),
                                            });
                                        }
                                        crate::util::set_focus_visible(true, cx);
                                        cx.stop_propagation();
                                        return;
                                    }
                                }
                            }
                        }
                    }
                    let page_by_step = |from: usize, step: usize| match key_name {
                        "pagedown" => {
                            let boundary = from.saturating_add(step).min(keys.len() - 1);
                            stops
                                .iter()
                                .copied()
                                .find(|stop| *stop >= boundary)
                                .or_else(|| stops.last().copied())
                        }
                        "pageup" => {
                            let boundary = from.saturating_sub(step);
                            stops
                                .iter()
                                .rev()
                                .copied()
                                .find(|stop| *stop <= boundary)
                                .or_else(|| stops.first().copied())
                        }
                        _ => None,
                    };
                    let fixed_page_move = from
                        .zip(fixed_page_step)
                        .and_then(|(from, step)| page_by_step(from, step));
                    let variable_page_move = from.and_then(|from| {
                        let viewport_height =
                            variable_scroll.as_ref()?.viewport_bounds().size.height;
                        let heights = variable_heights.as_ref()?.read(cx);
                        let estimate = variable_estimate?;
                        let height_at = |index: usize| {
                            heights.1.get(index).copied().flatten().unwrap_or(estimate)
                        };
                        let mut distance = height_at(from);
                        let mut target = from;
                        match key_name {
                            "pagedown" => {
                                if distance >= viewport_height {
                                    return Some(target);
                                }
                                for next in stops.iter().copied().filter(|next| *next > from) {
                                    for index in target + 1..=next {
                                        distance += height_at(index);
                                    }
                                    target = next;
                                    if distance >= viewport_height {
                                        break;
                                    }
                                }
                                Some(target)
                            }
                            "pageup" => {
                                if distance >= viewport_height {
                                    return Some(target);
                                }
                                for previous in stops
                                    .iter()
                                    .rev()
                                    .copied()
                                    .filter(|previous| *previous < from)
                                {
                                    for index in previous..target {
                                        distance += height_at(index);
                                    }
                                    target = previous;
                                    if distance >= viewport_height {
                                        break;
                                    }
                                }
                                Some(target)
                            }
                            _ => None,
                        }
                    });
                    let is_variable_page =
                        fixed_page_move.is_none() && variable_page_move.is_some();
                    // Pinned TableKeyboardDelegate sends PageDown to the last
                    // enabled row, and PageUp out of the body entirely, into
                    // the first column header. The header is focusable whether
                    // or not it sorts, so this leaves the body rather than
                    // stopping at its first row.
                    if plain_rows && key_name == "pageup" {
                        if let Some(header) = &page_up_header {
                            window.focus(header, cx);
                            cx.stop_propagation();
                            return;
                        }
                    }
                    let plain_page_move =
                        from.filter(|_| plain_rows).and_then(|_| match key_name {
                            "pagedown" => stops.last().copied(),
                            _ => None,
                        });
                    let page_move = fixed_page_move
                        .or(variable_page_move)
                        .or(plain_page_move)
                        .filter(|next| Some(*next) != from);
                    // The pinned registrations install no Home/End handler
                    // for an unregistered chord -- Cmd- or Ctrl-bearing on
                    // macOS, Alt- or platform-bearing elsewhere -- so the
                    // whole event stays inert: no focus move, no selection,
                    // no preventDefault, and no first-press settle either.
                    if matches!(key_name, "home" | "end")
                        && !home_end_registered(*modifiers, cfg!(target_os = "macos"))
                    {
                        return;
                    }
                    let initial_home_end_extends = shift_home_end_extends(
                        "home",
                        modifiers.control,
                        cfg!(target_os = "macos"),
                    );
                    let initial_shift_settle = from.is_none()
                        && modifiers.shift
                        && (key_name == "up"
                            || (matches!(key_name, "home" | "end") && !initial_home_end_extends)
                            || (key_name == "down" && stops.len() < 2));
                    let navigation = if from.is_none() && modifiers.shift && key_name == "down" {
                        stops
                            .get(1)
                            .or_else(|| stops.first())
                            .copied()
                            .map_or(crate::list_nav::Move::Ignore, crate::list_nav::Move::To)
                    } else if initial_shift_settle {
                        (if key_name == "end" {
                            stops.last()
                        } else {
                            stops.first()
                        })
                        .copied()
                        .map_or(crate::list_nav::Move::Ignore, crate::list_nav::Move::To)
                    } else {
                        page_move.map_or_else(
                            || crate::list_nav::resolve(&stops, from, key_name, false),
                            crate::list_nav::Move::To,
                        )
                    };
                    match navigation {
                        crate::list_nav::Move::To(next) => {
                            // Pinned `useSelectableCollection`: Shift extends a
                            // multiple selection from the anchor with no other
                            // chord, so plain Shift navigation is exact.
                            let exact_shift_navigation = if cfg!(target_os = "macos") {
                                !modifiers.control && !modifiers.platform && !modifiers.function
                            } else {
                                !modifiers.alt && !modifiers.platform && !modifiers.function
                            };
                            let extends_selection = modifiers.shift
                                && mode == SelectionMode::Multiple
                                && exact_shift_navigation
                                && !initial_shift_settle
                                && Some(next) != from
                                && shift_home_end_extends(
                                    key_name,
                                    modifiers.control,
                                    cfg!(target_os = "macos"),
                                );
                            if extends_selection {
                                if let Some(target) = keys.get(next) {
                                    let range = selection_range_for_keys.read(cx).clone();
                                    let next_selection = extend_selection_range(
                                        &selected_now,
                                        &keys,
                                        selectable_collection_keys
                                            .as_ref()
                                            .expect("multiple Tables have selectable keys")
                                            .as_slice(),
                                        &range,
                                        target,
                                    );
                                    selection_range_for_keys.update(cx, |range, _| {
                                        if range.anchor.is_none() {
                                            range.anchor = Some(target.clone());
                                        }
                                        range.current = Some(target.clone());
                                        range.is_all = false;
                                    });
                                    if !same_selection(&next_selection, &selected_now) {
                                        if let Some(held) = &selection_own_for_keys {
                                            held.update(cx, |value, cx| {
                                                *value = next_selection.clone();
                                                cx.notify();
                                            });
                                        }
                                        if let Some(cb) = &selection {
                                            cb(&next_selection, window, cx);
                                        }
                                    }
                                }
                            }
                            held.update(cx, |v, cx| {
                                *v = keys.get(next).cloned();
                                cx.notify();
                            });
                            if fixed_virtual {
                                fixed_scroll.scroll_to_item(next, gpui::ScrollStrategy::Center);
                            } else if let Some(state) = &variable_scroll {
                                if is_variable_page {
                                    state.scroll_to_reveal_item(next);
                                } else {
                                    // Unmeasured rows have no cached height, so
                                    // `scroll_to_reveal_item` cannot locate a far
                                    // target. Jump by logical index; the next
                                    // layout measures that row at the viewport.
                                    state.scroll_to(gpui::ListOffset {
                                        item_ix: next,
                                        offset_in_item: px(0.),
                                    });
                                }
                            }
                        }
                        crate::list_nav::Move::Activate => {
                            let Some(index) = from else { return };
                            let activation = if event.keystroke.key == "enter" {
                                RowActivation::Enter
                            } else {
                                RowActivation::Space
                            };
                            match row_intent(
                                activation,
                                mode,
                                selected_now.is_empty(),
                                on_row_click.is_some(),
                            ) {
                                RowIntent::Action => {
                                    if let Some(cb) = &on_row_click {
                                        cb(index, &ClickEvent::default(), window, cx);
                                    }
                                }
                                RowIntent::Selection => {
                                    if let Some(key) = keys.get(index) {
                                        let was_selected = selected_now.contains(key);
                                        let extends_selection =
                                            modifiers.shift && mode == SelectionMode::Multiple;
                                        let next = if extends_selection {
                                            let range = selection_range_for_keys.read(cx).clone();
                                            extend_selection_range(
                                                &selected_now,
                                                &keys,
                                                selectable_collection_keys
                                                    .as_ref()
                                                    .expect("multiple Tables have selectable keys")
                                                    .as_slice(),
                                                &range,
                                                key,
                                            )
                                        } else {
                                            crate::selection::next_selection(
                                                &selected_now,
                                                key,
                                                mode,
                                                false,
                                            )
                                        };
                                        let changed = !same_selection(&next, &selected_now);
                                        if changed {
                                            if let Some(held) = &selection_own_for_keys {
                                                held.update(cx, |value, cx| {
                                                    *value = next.clone();
                                                    cx.notify();
                                                });
                                            }
                                            if let Some(cb) = &selection {
                                                cb(&next, window, cx);
                                            }
                                        }
                                        if extends_selection && changed {
                                            selection_range_for_keys.update(cx, |range, _| {
                                                if range.anchor.is_none() {
                                                    range.anchor = Some(key.clone());
                                                }
                                                range.current = Some(key.clone());
                                                range.is_all = false;
                                            });
                                        } else if !extends_selection
                                            && mode == SelectionMode::Multiple
                                            && !was_selected
                                        {
                                            selection_range_for_keys.update(cx, |range, _| {
                                                range.anchor = Some(key.clone());
                                                range.current = Some(key.clone());
                                                range.is_all = false;
                                            });
                                        } else if !extends_selection
                                            && mode == SelectionMode::Multiple
                                        {
                                            selection_range_for_keys.update(cx, |range, _| {
                                                if range.is_all {
                                                    *range = TableSelectionRange::default();
                                                }
                                            });
                                        }
                                    }
                                }
                                RowIntent::None => {}
                            }
                        }
                        crate::list_nav::Move::Ignore => {}
                    }
                });
            }
        }

        let row_count = if self.virtual_rows.is_some()
            && (self.row_height.is_some() || self.estimated_row_height.is_some())
        {
            virtual_visible_count
        } else {
            flat.len()
        };
        let virtual_projection = std::rc::Rc::new(
            virtual_projection
                .map(|projection| std::sync::Arc::clone(&projection.visible))
                .unwrap_or_default(),
        );

        // `estimatedRowHeight` takes the variable-height path: gpui's `list`
        // measures each row it builds, and its state is intrusive, so it lives in
        // the window's keyed store and resets when the count changes.
        if let (Some(row_height), Some((_, _, _, factory))) =
            (self.row_height, self.virtual_rows.clone())
        {
            // The body scrolls inside `uniform_list`, which asks for the rows the
            // viewport shows and no others.
            let height = self.max_h.unwrap_or(px(400.));
            let rows = ctx.clone();
            let projection = virtual_projection;
            body = body.child(
                gpui::uniform_list(
                    gpui::ElementId::Name(format!("{table_id}-virtual-rows").into()),
                    row_count,
                    move |range, _window, cx| {
                        range
                            .map(|i| {
                                let (source_index, key, metadata) = &projection[i];
                                let row_data = factory(*source_index);
                                rows.row(
                                    i,
                                    row_data,
                                    metadata.depth,
                                    metadata.has_children,
                                    key,
                                    Some(row_height),
                                    cx,
                                )
                            })
                            .collect::<Vec<_>>()
                    },
                )
                .track_scroll(&virtual_scroll_now)
                .h(height)
                .w_full(),
            );
        } else if let (Some(state), Some((_, _, _, factory))) =
            (virtual_list_state, self.virtual_rows.clone())
        {
            let height = self.max_h.unwrap_or(px(400.));
            let rows = ctx.clone();
            let projection = virtual_projection;
            let measured_heights =
                virtual_row_heights.expect("estimated row height creates a measurement store");
            body = body.child(
                gpui::list(state, move |i, _window, cx| {
                    let (source_index, key, metadata) = &projection[i];
                    let row_data = factory(*source_index);
                    let row = rows.row(
                        i,
                        row_data,
                        metadata.depth,
                        metadata.has_children,
                        key,
                        None,
                        cx,
                    );
                    let measured = measured_heights.clone();
                    gpui::div()
                        .relative()
                        .w_full()
                        .child(row)
                        .child(
                            gpui::canvas(
                                move |bounds: gpui::Bounds<Pixels>, _, cx| {
                                    measured.update(cx, |(_, heights), cx| {
                                        if heights.get(i).copied().flatten()
                                            != Some(bounds.size.height)
                                        {
                                            heights[i] = Some(bounds.size.height);
                                            cx.notify();
                                        }
                                    });
                                    bounds
                                },
                                |_, _, _, _| {},
                            )
                            .absolute()
                            .inset_0(),
                        )
                        .into_any_element()
                })
                .h(height)
                .w_full(),
            );
        } else {
            for (i, (row_data, depth, has_children, tree_key, _)) in flat.into_iter().enumerate() {
                body = body.child(ctx.row(i, row_data, depth, has_children, &tree_key, None, cx));
            }
        }

        // ---- empty state -------------------------------------------------
        if row_count == 0 {
            if let Some(content) = self.empty_state {
                body = body.child(
                    gpui::div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w_full()
                        .py(px(28.))
                        .text_color(muted)
                        .child(content),
                );
            }
        }

        table = table.child(body);

        // ---- load-more sentinel ------------------------------------------
        if self.is_pending || self.on_load_more.is_some() {
            if let Some(cb) = self.on_load_more.clone() {
                let state = load_more_state;
                let scroll_offset = self.load_more_offset;
                let virtual_scroll = load_more_virtual_scroll;
                let variable_scroll = load_more_variable_scroll;
                table = table.child(
                    gpui::canvas(
                        move |bounds, window, cx| {
                            // Plain rows place this canvas at the real content
                            // end, so ancestor masks provide its viewport
                            // intersection. Virtual rows keep that end inside
                            // their own scroll state instead.
                            let mask = window.content_mask().bounds;
                            let in_view = bounds.right() >= mask.left()
                                && bounds.left() <= mask.right()
                                && bounds.bottom() >= mask.top()
                                && bounds.top() <= mask.bottom() + mask.size.height * scroll_offset;
                            let virtual_end_is_near = if let Some(handle) = &virtual_scroll {
                                let scroll = handle.0.borrow();
                                scroll.last_item_size.is_some_and(|size| {
                                    let remaining = size.contents.height
                                        + scroll.base_handle.offset().y
                                        - size.item.height;
                                    remaining <= size.item.height * scroll_offset
                                })
                            } else if let Some((state, count, estimate)) = &variable_scroll {
                                let viewport = state.viewport_bounds();
                                let viewport_height = viewport.size.height;
                                let top = state.logical_scroll_top();
                                // Unmeasured ListState rows have zero height,
                                // so project the unseen tail from the caller's
                                // layout estimate instead.
                                let remaining = *estimate * count.saturating_sub(top.item_ix)
                                    - top.offset_in_item
                                    - viewport_height;
                                let margin = viewport_height * scroll_offset;
                                let measured_end_is_near = *count == 0
                                    || state.bounds_for_item(count.saturating_sub(1)).is_some_and(
                                        |bounds| bounds.bottom() <= viewport.bottom() + margin,
                                    );
                                measured_end_is_near || (scroll_offset > 0. && remaining <= margin)
                            } else {
                                true
                            };
                            let visible = if virtual_scroll.is_some() || variable_scroll.is_some() {
                                virtual_end_is_near
                            } else {
                                in_view
                            };
                            let previous = state.read(cx).clone();
                            let collection_changed = previous
                                .1
                                .as_ref()
                                .is_none_or(|identity| identity != &load_more_collection);
                            let should_load = visible && (!previous.0 || collection_changed);
                            if previous.0 != visible || should_load {
                                state.update(cx, |value, cx| {
                                    value.0 = visible;
                                    if should_load {
                                        value.1 = Some(load_more_collection.clone());
                                    }
                                    cx.notify();
                                });
                            }
                            if should_load {
                                cb(window, cx);
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .size(px(1.)),
                );
            }

            if self.is_pending {
                let sentinel = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("{}-load-more", self.id).into(),
                ))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(8.))
                .w_full()
                // `.table__load-more-content` is `gap-2 py-2`; `loaderHeight`
                // fixes the row's height instead when the caller sets it.
                .py(px(8.))
                .when_some(self.loader_height, |el, h| el.h(h))
                .text_size(px(13.))
                .text_color(muted)
                    .child(
                        crate::spinner::Spinner::new(gpui::ElementId::Name(
                            format!("{}-load-spinner", self.id).into(),
                        ))
                        .size(herogpui_core::Size::Sm),
                    )
                    .child("Loading\u{2026}");
                table = table.child(sentinel);
            }
        }

        if let Some(footer) = self.footer {
            // `.table__footer` is `flex items-center px-4 py-2.5`.
            table = table.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .w_full()
                    .px(px(16.))
                    .py(px(10.))
                    .child(footer),
            );
        }

        // `.table__scroll-container` is `overflow-x-auto` around the content:
        // a row flex, so its one child (the content column above) is free to be
        // wider than the scroller itself, which is what a table wider than its
        // box scrolls *on*.
        wrapper.child(
            gpui::div()
                .id(gpui::ElementId::Name(
                    format!("{}-scroll-x", self.id).into(),
                ))
                .flex()
                .w_full()
                .overflow_x_scroll()
                .child(table),
        )
    }
}

/// Everything one body row needs, so both the plain and the virtualized path can
/// draw it from the same code.
struct RowCtx {
    /// The table's id, so one table's row ids cannot collide with another's.
    id: SharedString,
    /// `(defaultWidth or the resize, minWidth, maxWidth)` per column.
    widths: Vec<(Option<Pixels>, Option<Pixels>, Option<Pixels>)>,
    row_header_columns: Vec<bool>,
    tree_column: usize,
    tree_column_has_children: bool,
    selectable: bool,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    row_keys: Vec<SharedString>,
    disabled_keys: Vec<SharedString>,
    expanded: std::rc::Rc<Vec<SharedString>>,
    on_expanded_change: Option<OnExpandedChange>,
    on_selection_change: Option<OnSelectionChange>,
    selection_own: Option<gpui::Entity<Vec<SharedString>>>,
    selection_range: gpui::Entity<TableSelectionRange>,
    on_row_click: Option<OnRowClick>,
    focus: gpui::FocusHandle,
    cursor_own: gpui::Entity<Option<SharedString>>,
    /// The row the keyboard is on, which wears `status-focused`.
    cursor: Option<usize>,
    /// The `.table-root--secondary` flat layout, whose row hover is a
    /// different token from the primary's.
    secondary: bool,
}

impl RowCtx {
    /// One body row.
    ///
    /// `fixed_h` is `Some` only on the virtual path, where every row is one
    /// `rowHeight` tall because that is the number the scroll geometry comes
    /// from.
    #[allow(clippy::too_many_arguments)]
    /// One `.table__row`: `relative h-full` with a `border-separator/50`
    /// bottom edge except on the final visible row, and `.table__cell`s inside
    /// it.
    fn row(
        &self,
        i: usize,
        row_data: TableRow,
        depth: usize,
        has_children: bool,
        tree_key: &SharedString,
        fixed_h: Option<Pixels>,
        cx: &mut App,
    ) -> AnyElement {
        let colors = cx.colors();
        let tree_column = self.tree_column;
        let tree_column_has_children = self.tree_column_has_children;
        let key = tree_key.clone();
        let is_selected = self.selected_keys.contains(&key);
        let is_disabled = self.disabled_keys.contains(&key);
        let row_header_columns = &self.row_header_columns;

        let mut row = gpui::div()
            .id(gpui::ElementId::Name(format!("{}-row-{i}", self.id).into()))
            .flex()
            // A virtual row is laid out on its own, so it takes the width it is
            // *given*: without `w_full` the columns bunch at the left edge. That
            // is true of both virtual paths, so it is not conditional on the
            // fixed height any more -- `gpui::list`'s rows have none.
            .w_full()
            .when_some(fixed_h, |e, h| e.h(h));
        if i + 1 < self.row_keys.len() {
            row = row.border_b_1().border_color(colors.separator.alpha(0.5));
        }

        if self.selectable {
            let mut cell = gpui::div()
                .id(gpui::ElementId::Name(
                    format!("{}-select-cell-{i}", self.id).into(),
                ))
                .flex()
                .items_center()
                .justify_center()
                .w(px(44.))
                .py(px(10.))
                .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                });
            let mut box_el = Checkbox::new(gpui::ElementId::Name(
                format!("{}-select-{i}", self.id).into(),
            ))
            .is_selected(is_selected)
            .is_disabled(is_disabled);
            let cb = self.on_selection_change.clone();
            if !is_disabled && (cb.is_some() || self.selection_own.is_some()) {
                let current = self.selected_keys.clone();
                let key2 = key.clone();
                let mode = self.selection_mode;
                let selection_own = self.selection_own.clone();
                let selection_range = self.selection_range.clone();
                box_el = box_el.on_change(move |_next, window, cx| {
                    cx.stop_propagation();
                    let next = crate::selection::next_selection(&current, &key2, mode, false);
                    if let Some(held) = &selection_own {
                        held.update(cx, |value, cx| {
                            *value = next.clone();
                            cx.notify();
                        });
                    }
                    if let Some(cb) = &cb {
                        cb(&next, window, cx);
                    }
                    if mode == SelectionMode::Multiple && !is_selected {
                        selection_range.update(cx, |range, _| {
                            range.anchor = Some(key2.clone());
                            range.current = Some(key2.clone());
                            range.is_all = false;
                        });
                    } else if mode == SelectionMode::Multiple {
                        selection_range.update(cx, |range, _| {
                            if range.is_all {
                                *range = TableSelectionRange::default();
                            }
                        });
                    }
                });
            }
            cell = cell.child(box_el);
            row = row.child(cell);
        }

        // Cells are flex rows so inline children (chips, buttons) size to
        // their content instead of stretching to the column width.
        let widths = &self.widths;
        let is_expanded = self.expanded.iter().any(|k| k == tree_key);
        let toggle_key = tree_key.clone();
        let expanded_before = self.expanded.clone();
        let on_expanded = self.on_expanded_change.clone();
        row = row.children(row_data.cells.into_iter().enumerate().map(|(c, cell)| {
            let (width, min_width, max_width) =
                widths.get(c).copied().unwrap_or((None, None, None));
            let mut cell_el = gpui::div()
                .when(width.is_none(), flex_cell)
                .when_some(width, |e, w| e.w(w))
                .when_some(min_width, |e, w| e.min_w(w))
                .when_some(max_width, |e, w| e.max_w(w))
                .flex()
                .items_center()
                .gap(px(6.))
                // `.table__cell` is `px-4 py-3`.
                .px(px(16.))
                .py(px(12.))
                .when(row_header_columns.get(c).copied().unwrap_or(false), |e| {
                    e.font_weight(gpui::FontWeight::MEDIUM)
                });
            // The tree column carries the indent and the chevron; a row with
            // no children still indents, so siblings line up.
            if c == tree_column {
                if depth > 0 {
                    cell_el = cell_el.pl(px(12. + 20. * depth as f32));
                }
                if has_children {
                    let mut chevron = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("{}-expand-{toggle_key}", self.id).into(),
                        ))
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(18.))
                        .flex_shrink_0()
                        .when(!is_disabled, |chevron| {
                            chevron.cursor_pointer().on_mouse_down(
                                gpui::MouseButton::Left,
                                |_, _, cx| {
                                    cx.stop_propagation();
                                },
                            )
                        })
                        .child(
                            gpui::svg()
                                .size(px(12.))
                                .path(if is_expanded {
                                    icons::CHEVRON_DOWN
                                } else {
                                    icons::CHEVRON_RIGHT
                                })
                                .text_color(colors.muted),
                        );
                    if !is_disabled {
                        if let Some(cb) = on_expanded.clone() {
                            let key = toggle_key.clone();
                            let before = expanded_before.clone();
                            let focus = self.focus.clone();
                            let cursor = self.cursor_own.clone();
                            chevron = chevron.on_click(move |_, window, cx| {
                                cx.stop_propagation();
                                let mut next = before.as_ref().clone();
                                if let Some(at) = next.iter().position(|k| *k == key) {
                                    next.remove(at);
                                } else {
                                    next.push(key.clone());
                                }
                                cb(&next, window, cx);
                                window.focus(&focus, cx);
                                cursor.update(cx, |value, cx| {
                                    *value = Some(key.clone());
                                    cx.notify();
                                });
                            });
                        }
                    }
                    cell_el = cell_el.child(chevron);
                } else if depth > 0 || tree_column_has_children {
                    // A leaf keeps the chevron's width so its text lines up
                    // with its expandable siblings'.
                    cell_el = cell_el.child(gpui::div().size(px(18.)).flex_shrink_0());
                }
            }
            cell_el.child(cell)
        }));

        // A selected row reads as selected even where the checkbox is off
        // screen, and outranks striping. `.table__row[data-selected]` fills
        // `bg-surface/10`, and the pinned rule follows the hover rule, so a
        // hovered selected row keeps this fill instead of the hover's. gpui
        // allows one hover refinement per element, so the fill rides the
        // single hover below.
        let selected_bg = is_selected.then(|| colors.surface.background.alpha(0.1));
        if let Some(selected_bg) = selected_bg {
            row = row.bg(selected_bg);
        }

        let row_action = self.on_row_click.clone();
        let row_selection = self.on_selection_change.clone();
        let selection_own = self.selection_own.clone();
        if !is_disabled
            && (row_action.is_some()
                || (self.selectable && (row_selection.is_some() || selection_own.is_some())))
        {
            let current = self.selected_keys.clone();
            let mode = self.selection_mode;
            let selection_range = self.selection_range.clone();
            let range_collection = self.row_keys.clone();
            let range_selectable: Vec<SharedString> = self
                .row_keys
                .iter()
                .filter(|key| !self.disabled_keys.contains(key))
                .cloned()
                .collect();
            let moved = self.cursor_own.clone();
            let focus = self.focus.clone();
            let focus_for_click = self.focus.clone();
            let key_for_cursor = key.clone();
            // `.table-root--primary` hovers `bg-surface/40`;
            // `.table-root--secondary` rows hover `bg-default/50`. A selected
            // row keeps its `bg-surface/10` fill instead -- the pinned
            // selected rule wins the cascade over the hover's.
            let secondary = self.secondary;
            row = row
                .cursor_pointer()
                .hover(move |s| {
                    s.bg(selected_bg.unwrap_or(if secondary {
                        colors.default.color.alpha(0.5)
                    } else {
                        colors.surface.background.alpha(0.4)
                    }))
                })
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    if window.default_prevented() {
                        return;
                    }
                    window.focus(&focus, cx);
                    moved.update(cx, |value, cx| {
                        *value = Some(key_for_cursor.clone());
                        cx.notify();
                    });
                })
                .on_click(move |ev, w, cx| {
                    if !focus_for_click.is_focused(w) {
                        return;
                    }
                    match row_intent(
                        RowActivation::Pointer,
                        mode,
                        current.is_empty(),
                        row_action.is_some(),
                    ) {
                        RowIntent::Action => {
                            if let Some(cb) = &row_action {
                                cb(i, ev, w, cx);
                            }
                        }
                        RowIntent::Selection => {
                            let was_selected = current.contains(&key);
                            let extends_selection =
                                ev.modifiers().shift && mode == SelectionMode::Multiple;
                            let next = if extends_selection {
                                let range = selection_range.read(cx).clone();
                                extend_selection_range(
                                    &current,
                                    &range_collection,
                                    &range_selectable,
                                    &range,
                                    &key,
                                )
                            } else {
                                crate::selection::next_selection(&current, &key, mode, false)
                            };
                            let changed = !same_selection(&next, &current);
                            if changed {
                                if let Some(held) = &selection_own {
                                    held.update(cx, |value, cx| {
                                        *value = next.clone();
                                        cx.notify();
                                    });
                                }
                                if let Some(cb) = &row_selection {
                                    cb(&next, w, cx);
                                }
                            }
                            if extends_selection && changed {
                                selection_range.update(cx, |range, _| {
                                    if range.anchor.is_none() {
                                        range.anchor = Some(key.clone());
                                    }
                                    range.current = Some(key.clone());
                                    range.is_all = false;
                                });
                            } else if !extends_selection
                                && mode == SelectionMode::Multiple
                                && !was_selected
                            {
                                selection_range.update(cx, |range, _| {
                                    range.anchor = Some(key.clone());
                                    range.current = Some(key.clone());
                                    range.is_all = false;
                                });
                            } else if !extends_selection && mode == SelectionMode::Multiple {
                                selection_range.update(cx, |range, _| {
                                    if range.is_all {
                                        *range = TableSelectionRange::default();
                                    }
                                });
                            }
                        }
                        RowIntent::None => {}
                    }
                });
        }

        // v3 rings the focused row *inside* itself: the cells each carry an
        // inset shadow, with the first and last three-sided, so the row reads as
        // one continuous outline. One overlay across the row is the same picture
        // and needs no per-cell cases.
        row.when(is_disabled, |row| row.opacity(cx.layout().disabled_opacity))
            .relative()
            .when(self.cursor == Some(i), |r| {
                r.child(crate::util::inset_focus_ring(cx))
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_column_starts_ascending() {
        let next = SortDescriptor::next(None, "name");
        assert_eq!(next, SortDescriptor::new("name", SortDirection::Ascending));
    }

    #[test]
    fn clicking_the_sorted_column_flips_it() {
        let current = SortDescriptor::new("name", SortDirection::Ascending);
        let next = SortDescriptor::next(Some(&current), "name");
        assert_eq!(next.direction, SortDirection::Descending);
        let back = SortDescriptor::next(Some(&next), "name");
        assert_eq!(back.direction, SortDirection::Ascending);
    }

    #[test]
    fn disabled_only_controlled_selection_is_indeterminate() {
        let selectable = vec![SharedString::from("alpha")];
        let selected = vec![SharedString::from("beta")];
        assert_eq!(select_all_flags(&selectable, &selected), (false, true));
    }

    #[test]
    fn switching_column_resets_to_ascending() {
        let current = SortDescriptor::new("name", SortDirection::Descending);
        let next = SortDescriptor::next(Some(&current), "size");
        assert_eq!(next, SortDescriptor::new("size", SortDirection::Ascending));
    }

    #[test]
    fn row_keys_fall_back_to_the_index() {
        let table = Table::new(vec!["A".into()])
            .row(vec![])
            .keyed_row("second", vec![])
            .row(vec![]);
        assert_eq!(
            table.row_keys(),
            vec![
                SharedString::from("0"),
                SharedString::from("second"),
                SharedString::from("2"),
            ]
        );
    }

    /// The Home/End gate takes the platform as an explicit bool, so this
    /// truth table is free of `cfg!` and mechanically proves both maps from
    /// any host: no macOS chord ever extends -- Shift and Alt+Shift move the
    /// focus alone -- while Windows and Linux extend exactly from
    /// Control+Shift.
    #[test]
    fn shift_home_end_extends_only_from_control_outside_macos() {
        for key in ["home", "end"] {
            assert!(
                !shift_home_end_extends(key, true, true),
                "macOS registers no Home/End extension"
            );
            assert!(!shift_home_end_extends(key, false, true));
            assert!(
                shift_home_end_extends(key, true, false),
                "Control+Shift+{key} must extend on Windows and Linux"
            );
            assert!(
                !shift_home_end_extends(key, false, false),
                "plain Shift+{key} must only move the focus"
            );
        }
    }

    /// The grid delegate's arrows never consult the Home/End gate: their
    /// forbidden extra chords are rejected earlier, by
    /// `exact_shift_navigation`.
    #[test]
    fn shift_navigation_keys_do_not_consult_the_home_end_gate() {
        for key in ["left", "right", "up", "down"] {
            assert!(shift_home_end_extends(key, false, true));
            assert!(shift_home_end_extends(key, true, false));
        }
    }

    /// The registration gate takes `Modifiers`, so the pinned chord map can
    /// be spelled out: macOS registers none, Shift, Alt, and Alt+Shift and
    /// every Control- or Meta-bearing chord is entirely inert, while
    /// Windows and Linux register none, Shift, Control, and Control+Shift
    /// and reject every Alt- or Meta-bearing chord. The upstream matcher
    /// sees only the browser's Alt/Control/Meta/Shift flags, so GPUI's
    /// `function` flag is ignored: `fn` stays registered on both maps, and
    /// it never rescues a chord the platform itself rejects.
    #[test]
    fn home_end_registration_matches_the_pinned_chord_map() {
        let none = gpui::Modifiers::none();
        let shift = gpui::Modifiers {
            shift: true,
            ..none
        };
        let alt = gpui::Modifiers { alt: true, ..none };
        let alt_shift = gpui::Modifiers { shift: true, ..alt };
        let function = gpui::Modifiers {
            function: true,
            ..none
        };
        let function_alt = gpui::Modifiers {
            alt: true,
            ..function
        };
        for modifiers in [none, shift, alt, alt_shift, function, function_alt] {
            assert!(
                home_end_registered(modifiers, true),
                "macOS must register {modifiers:?}"
            );
        }
        let control = gpui::Modifiers {
            control: true,
            ..none
        };
        let control_shift = gpui::Modifiers {
            shift: true,
            ..control
        };
        let platform = gpui::Modifiers {
            platform: true,
            ..none
        };
        let platform_shift = gpui::Modifiers {
            shift: true,
            ..platform
        };
        for modifiers in [control, control_shift, platform, platform_shift] {
            assert!(
                !home_end_registered(modifiers, true),
                "macOS must not register {modifiers:?}"
            );
        }
        for modifiers in [none, shift, control, control_shift, function] {
            assert!(
                home_end_registered(modifiers, false),
                "Windows and Linux must register {modifiers:?}"
            );
        }
        for modifiers in [alt, alt_shift, function_alt, platform, platform_shift] {
            assert!(
                !home_end_registered(modifiers, false),
                "Windows and Linux must not register {modifiers:?}"
            );
        }
    }

    /// The keystroke spellings real events hand the gate: `ctrl` parses to
    /// the Control field the Windows/Linux registration admits and macOS
    /// vetoes, `cmd` to the platform field macOS vetoes, `alt-shift` to
    /// the chord that stays registered (focus-only) on macOS alone, and
    /// `fn` to the flag the browser matcher never sees, so it registers
    /// exactly like the bare key on both maps.
    #[test]
    fn keystroke_spellings_reach_the_registration_gate() {
        let ctrl_shift_home = gpui::Keystroke::parse("ctrl-shift-home").unwrap();
        assert!(home_end_registered(ctrl_shift_home.modifiers, false));
        assert!(!home_end_registered(ctrl_shift_home.modifiers, true));
        let cmd_shift_home = gpui::Keystroke::parse("cmd-shift-home").unwrap();
        assert!(!home_end_registered(cmd_shift_home.modifiers, true));
        let alt_shift_end = gpui::Keystroke::parse("alt-shift-end").unwrap();
        assert!(home_end_registered(alt_shift_end.modifiers, true));
        assert!(!home_end_registered(alt_shift_end.modifiers, false));
        let fn_home = gpui::Keystroke::parse("fn-home").unwrap();
        assert!(fn_home.modifiers.function);
        assert!(home_end_registered(fn_home.modifiers, true));
        assert!(home_end_registered(fn_home.modifiers, false));
    }

    // The pinned sortable column hover only recolours the header text to
    // `--foreground`; it paints no background. `soft()` is a lighter, wrong
    // wash, so the check is mechanical.
    #[test]
    fn the_sort_header_hover_paints_no_background() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains(".group_hover(sort_group, |s| s.text_color(colors.foreground))"),
            "the sortable header hover must recolour the text to `--foreground` \
             (pinned `.table__column[data-allows-sorting]:hover`)"
        );
        assert!(
            !source.contains("colors.default.soft()"),
            "the sortable header hover must not come back as a soft background"
        );
    }

    // The pinned row hover is variant-specific: `.table-root--primary` rows
    // fill `bg-surface/40`, `.table-root--secondary` rows fill
    // `bg-default/50`.
    #[test]
    fn the_row_hover_is_variant_specific() {
        // Scan the implementation only.
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("colors.default.color.alpha(0.5)")
                && source.contains("colors.surface.background.alpha(0.4)"),
            "the row hover must read `bg-default/50` on secondary and \
             `bg-surface/40` on primary"
        );
    }

    // The pinned selected row fills `bg-surface/10`, and the rule follows the
    // hover rule in the pinned stylesheet, so it wins the cascade: a hovered
    // selected row keeps the selected fill. `accent.soft()` is a role wash
    // the pinned table never paints, so the check is mechanical.
    #[test]
    fn the_selected_row_fills_surface_ten_percent_and_outranks_the_hover() {
        // Scan the implementation only; this test's own text names the
        // forbidden accessor.
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains(
                "let selected_bg = is_selected.then(|| colors.surface.background.alpha(0.1));"
            ),
            "the selected row must fill `bg-surface/10` \
             (pinned `.table__row[data-selected] .table__cell`)"
        );
        assert!(
            source.contains("s.bg(selected_bg.unwrap_or(if secondary {"),
            "the hover must give way to the selected fill -- the pinned \
                 selected rule wins the cascade over `.table__row:hover`"
        );
        assert!(
            !source.contains("accent.soft()"),
            "the selected row must not paint a role soft wash"
        );
    }

    #[test]
    fn table_header_uses_medium_weight() {
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains(
                ".text_size(px(12.))\n                .line_height(px(16.))\n                .font_weight(gpui::FontWeight::MEDIUM)\n                .text_color"
            ),
            "table column headers must use the pinned `font-medium` weight"
        );
    }

    #[test]
    fn table_header_preserves_caller_label_case() {
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains(".child(column.label.clone());"),
            "table column headers must render the caller's label without uppercasing it"
        );
    }

    #[test]
    fn table_header_and_rows_use_half_alpha_separator() {
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.match_indices("colors.separator.alpha(0.5)").count() >= 2,
            "the header and row separators must use the separator token at 50% alpha"
        );
    }

    #[test]
    fn table_last_row_omits_the_bottom_separator() {
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            source.contains("if i + 1 < self.row_keys.len() {")
                && source
                    .contains("row = row.border_b_1().border_color(colors.separator.alpha(0.5));"),
            "the final visible Table row must not receive a bottom separator"
        );
    }

    #[test]
    fn virtual_selectable_keys_are_not_materialized_on_every_render() {
        let source = include_str!("table.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("the implementation section is always present");
        assert!(
            !source.contains("let selectable_collection_keys: Vec<SharedString> ="),
            "virtual Table must not allocate a selectable-key Vec for every render"
        );
    }
}
