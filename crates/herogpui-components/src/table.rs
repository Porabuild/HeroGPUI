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
type VirtualRowKey = std::sync::Arc<dyn Fn(usize) -> SharedString + 'static>;
type VirtualRow = std::sync::Arc<dyn Fn(usize) -> TableRow + 'static>;
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
    /// `defaultWidth` — a fixed column width. Without one the column shares the
    /// row evenly, which is what `flex-1` does.
    width: Option<Pixels>,
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

    /// Supplies preorder tree structure for [`Table::virtual_rows`].
    ///
    /// `count` remains the size of the underlying collection. The projection
    /// identifies each item's parent, depth and expandability; controlled
    /// `expanded_keys` then decides which source indices are visible.
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
        let (selected_keys, selection_own) = crate::util::controlled(
            window,
            cx,
            gpui::ElementId::Name(format!("{}-selected", self.id).into()),
            self.is_selection_controlled
                .then(|| self.selected_keys.clone()),
            Vec::new(),
        );
        self.selected_keys = selected_keys;
        let virtual_projection = self
            .virtual_rows
            .as_ref()
            .map(|(count, _, key_for_row, _)| {
                let mut full_keys = Vec::with_capacity(*count);
                let mut visible = Vec::with_capacity(*count);
                if let Some(project) = &self.virtual_tree_metadata {
                    let mut visible_by_key = std::collections::HashMap::with_capacity(*count);
                    for source_index in 0..*count {
                        let key = key_for_row(source_index);
                        let metadata = project(source_index);
                        let is_visible = metadata.parent_key.as_ref().is_none_or(|parent| {
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
                (full_keys, visible)
            });
        let virtual_keys = virtual_projection
            .as_ref()
            .map(|(full_keys, _)| full_keys.clone());
        let virtual_visible_count = virtual_projection
            .as_ref()
            .map_or(0, |(_, visible)| visible.len());
        let virtual_visible_keys = virtual_projection.as_ref().map(|(_, visible)| {
            visible
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
        let sort_focus: Vec<Option<gpui::FocusHandle>> = self
            .columns
            .iter()
            .map(|c| c.allows_sorting && sortable)
            .collect::<Vec<_>>()
            .into_iter()
            .enumerate()
            .map(|(i, is_sortable)| {
                is_sortable.then(|| {
                    crate::util::tab_stop_handle(
                        gpui::ElementId::Name(format!("{}-sort-{i}-focus", self.id).into()),
                        window,
                        cx,
                    )
                })
            })
            .collect();
        let ring_visible = crate::util::focus_visible(cx);
        let sort_focused: Vec<bool> = sort_focus
            .iter()
            .map(|h| h.as_ref().is_some_and(|h| h.is_focused(window)) && ring_visible)
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
                let width = resized_now
                    .get(column_index)
                    .copied()
                    .flatten()
                    .or(column.width);
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
        let colors = cx.colors();
        // Copies of the tokens the tail needs: the row builder borrows `cx`
        // mutably, which ends the borrow `cx.colors()` holds.
        let muted = colors.muted;
        let secondary = self.variant == TableVariant::Secondary;
        let selectable = self.selection_mode != SelectionMode::None;
        let full_collection_keys = virtual_keys.unwrap_or_else(|| self.row_keys());
        let selectable_collection_keys: Vec<SharedString> = full_collection_keys
            .iter()
            .filter(|key| !self.disabled_keys.contains(key))
            .cloned()
            .collect();
        let load_more_collection = match &self.virtual_rows {
            Some((count, identity, _, _)) => LoadMoreCollection::Virtual {
                count: *count,
                identity: identity.clone(),
            },
            None => LoadMoreCollection::Rows(full_collection_keys),
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
            .when_some(self.gap, |el, g| el.gap(g))
            .when_some(self.padding, |el, p| el.p(p));

        // ---- header ------------------------------------------------------
        // `.table__header`, whose cells are `.table__column`s and whose
        // sortable ones wrap in `.table__sortable-column-header`.
        let mut header = gpui::div()
            .flex()
            .border_b_1()
            .border_color(colors.separator)
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
                let all = selectable_collection_keys.clone();
                let (all_selected, indeterminate) = select_all_flags(&all, &self.selected_keys);
                let mut box_el = Checkbox::new(gpui::ElementId::Name(
                    format!("{}-select-all", self.id).into(),
                ))
                .is_selected(all_selected)
                .is_indeterminate(indeterminate);
                let cb = self.on_selection_change.clone();
                if cb.is_some() || selection_own.is_some() {
                    let selection_own = selection_own.clone();
                    box_el = box_el.on_change(move |_next, window, cx| {
                        // Anything short of everything selects everything.
                        let next: Vec<SharedString> = if all_selected {
                            Vec::new()
                        } else {
                            all.clone()
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
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(if sorted.is_some() {
                    colors.foreground
                } else {
                    colors.muted
                })
                .child(column.label.to_uppercase());

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
                    let hover = colors.default.soft();
                    let header_cell = gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("table-sort-{}", column.label).into(),
                        ))
                        .flex_1()
                        .flex()
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover))
                        // The focus is what makes Enter and Space sort: gpui
                        // fires a *focused* element's click listeners for them.
                        .when_some(sort_focus[column_index].as_ref(), |c, handle| {
                            c.track_focus(handle)
                        })
                        .on_click(move |_, window, cx| cb(next.clone(), window, cx))
                        .child(cell);
                    // `.table__column` rings *inside* itself: the next column
                    // is flush against this one, and a ring drawn outside bled
                    // through the transparent cell and filled it.
                    header_cell
                        .relative()
                        .when(sort_focused[column_index], |c| {
                            c.child(crate::util::inset_focus_ring(cx))
                        })
                        .into_any_element()
                }
                _ => cell.into_any_element(),
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
                let widths = resized.clone();
                let (min_width, max_width) = resize_limits[column_index];
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
                                window.focus(&focus_for_mouse);
                                let x = f32::from(ev.position.x);
                                held.update(cx, |v, _| {
                                    *v = Some((column_index, x, f32::from(start_width)));
                                });
                            })
                            .on_mouse_down_out(move |_, _, cx| {
                                if *keyboard_out.read(cx) == Some(column_index) {
                                    keyboard_out.update(cx, |active, cx| {
                                        *active = None;
                                        cx.notify();
                                    });
                                }
                            })
                            .on_key_down(move |event, _window, cx| {
                                let key = event.keystroke.key.as_str();
                                let editing = *keyboard.read(cx) == Some(column_index);
                                match key {
                                    "enter" => {
                                        keyboard.update(cx, |active, cx| {
                                            *active = if editing { None } else { Some(column_index) };
                                            cx.notify();
                                        });
                                        cx.stop_propagation();
                                    }
                                    "escape" | "space" if editing => {
                                        keyboard.update(cx, |active, cx| {
                                            *active = None;
                                            cx.notify();
                                        });
                                        cx.stop_propagation();
                                    }
                                    "tab" if editing => {
                                        keyboard.update(cx, |active, cx| {
                                            *active = None;
                                            cx.notify();
                                        });
                                        cx.stop_propagation();
                                    }
                                    "right" | "up" | "left" | "down" if editing => {
                                        let delta = if matches!(key, "right" | "up") {
                                            10.
                                        } else {
                                            -10.
                                        };
                                        widths.update(cx, |values, cx| {
                                            if values.len() <= column_index {
                                                values.resize(column_index + 1, None);
                                            }
                                            let current = values[column_index].unwrap_or(start_width);
                                            let next = (f32::from(current) + delta)
                                                .floor()
                                                .min(max_width)
                                                .max(min_width);
                                            values[column_index] = Some(px(next));
                                            cx.notify();
                                        });
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

        // The drag itself: the pointer can leave the handle, so the table
        // watches the move and the release.
        if resizable {
            let held = dragging.clone();
            let held_up = dragging;
            let widths = resized;
            table = table
                .on_mouse_move(move |ev, _window, cx| {
                    let Some((column, from_x, from_w)) = *held.read(cx) else {
                        return;
                    };
                    let raw = (from_w + f32::from(ev.position.x) - from_x).floor();
                    widths.update(cx, |v, cx| {
                        if v.len() <= column {
                            v.resize(column + 1, None);
                        }
                        v[column] = Some(px(raw));
                        cx.notify();
                    });
                })
                .on_mouse_up(gpui::MouseButton::Left, move |_, _window, cx| {
                    if held_up.read(cx).is_some() {
                        held_up.update(cx, |v, cx| {
                            *v = None;
                            cx.notify();
                        });
                    }
                });
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
            |(_, visible)| {
                visible
                    .iter()
                    .map(|(_, _, metadata)| (metadata.has_children, metadata.parent_key.clone()))
                    .collect()
            },
        );
        let visible_collection_keys: Vec<SharedString> = virtual_visible_keys
            .unwrap_or_else(|| flat.iter().map(|(_, _, _, key, _)| key.clone()).collect());
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
            |(_, visible)| visible.iter().any(|(_, _, metadata)| metadata.has_children),
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
            on_row_click: self.on_row_click.clone(),
            focus: table_focus.clone(),
            cursor_own: row_cursor.clone(),
            cursor: cursor_at,
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
            let on_row_click = self.on_row_click.clone();
            let keys = ctx.row_keys.clone();
            let table_focus_for_keys = table_focus;
            let selection = self.on_selection_change.clone();
            let selection_own_for_keys = selection_own;
            let selected_now = self.selected_keys.clone();
            let mode = self.selection_mode;
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
                        let (all_selected, _) =
                            select_all_flags(&selectable_collection_keys, &selected_now);
                        // Pinned React Stately's `selectAll` is idempotent once
                        // the whole selectable collection is already selected.
                        if !all_selected {
                            let next = selectable_collection_keys.clone();
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
                        cx.stop_propagation();
                        return;
                    }
                    // Other collection keys belong only to the body's roving
                    // focus stop. A nested cell action must keep its own Enter
                    // and Space handling even though Mod+A bubbles to the root.
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
                    let page_move = fixed_page_move
                        .or(variable_page_move)
                        .filter(|next| Some(*next) != from);
                    let navigation = page_move.map_or_else(
                        || crate::list_nav::resolve(&stops, from, key_name, false),
                        crate::list_nav::Move::To,
                    );
                    match navigation {
                        crate::list_nav::Move::To(next) => {
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
                                        let next = crate::selection::next_selection(
                                            &selected_now,
                                            key,
                                            mode,
                                            false,
                                        );
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
                .map(|(_, visible)| visible)
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
                .track_scroll(virtual_scroll_now)
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
    on_row_click: Option<OnRowClick>,
    focus: gpui::FocusHandle,
    cursor_own: gpui::Entity<Option<SharedString>>,
    /// The row the keyboard is on, which wears `status-focused`.
    cursor: Option<usize>,
}

impl RowCtx {
    /// One body row.
    ///
    /// `fixed_h` is `Some` only on the virtual path, where every row is one
    /// `rowHeight` tall because that is the number the scroll geometry comes
    /// from.
    #[allow(clippy::too_many_arguments)]
    /// One `.table__row`: `relative h-full` with a `border-separator/50`
    /// bottom edge, and `.table__cell`s inside it.
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
        let accent = colors.accent;
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
            .when_some(fixed_h, |e, h| e.h(h))
            .border_b_1()
            .border_color(colors.separator);

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
                                window.focus(&focus);
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
        // screen, and outranks striping.
        if is_selected {
            row = row.bg(accent.soft());
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
            let moved = self.cursor_own.clone();
            let focus = self.focus.clone();
            let focus_for_click = self.focus.clone();
            let key_for_cursor = key.clone();
            row = row
                .cursor_pointer()
                .hover(move |s| s.bg(colors.default.soft()))
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    if window.default_prevented() {
                        return;
                    }
                    window.focus(&focus);
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
                            let next =
                                crate::selection::next_selection(&current, &key, mode, false);
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
}
