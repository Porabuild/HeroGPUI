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

    /// The key this row selects under, given its position.
    fn selection_key(&self, index: usize) -> SharedString {
        match &self.key {
            Some(k) => k.clone(),
            None => SharedString::from(index.to_string()),
        }
    }
}

type OnExpandedChange = std::sync::Arc<dyn Fn(&[SharedString], &mut Window, &mut App) + 'static>;

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
    /// How tall the virtual body is. v3 sets it with a `className`.
    max_h: Option<Pixels>,
    /// `TableLayout`'s `gap` and `padding`, both 0 in v3: rows meet, separated
    /// by their border rather than by space.
    gap: Option<Pixels>,
    padding: Option<Pixels>,
    /// The row factory a virtual table needs. Cells are `AnyElement`, which
    /// cannot be built up front and then handed out again on the next scroll,
    /// so a virtual table asks for its rows one at a time.
    virtual_rows: Option<(usize, std::sync::Arc<dyn Fn(usize) -> TableRow + 'static>)>,
    variant: TableVariant,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    sort_descriptor: Option<SortDescriptor>,
    show_indicator: bool,
    is_pending: bool,
    empty_state: Option<AnyElement>,
    on_row_click: Option<OnRowClick>,
    on_selection_change: Option<OnSelectionChange>,
    on_sort_change: Option<OnSortChange>,
    on_load_more: Option<OnLoadMore>,
}

impl Table {
    pub fn new(columns: Vec<SharedString>) -> Self {
        Self {
            indicator: None,
            columns: columns.into_iter().map(TableColumn::new).collect(),
            rows: Vec::new(),
            tree_column: 0,
            expanded_keys: Vec::new(),
            on_expanded_change: None,
            row_height: None,
            max_h: None,
            gap: None,
            padding: None,
            virtual_rows: None,
            variant: TableVariant::Primary,
            selection_mode: SelectionMode::None,
            selected_keys: Vec::new(),
            sort_descriptor: None,
            show_indicator: true,
            is_pending: false,
            empty_state: None,
            on_row_click: None,
            on_selection_change: None,
            on_sort_change: None,
            on_load_more: None,
        }
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
    pub fn virtual_rows(mut self, count: usize, row: impl Fn(usize) -> TableRow + 'static) -> Self {
        self.virtual_rows = Some((count, std::sync::Arc::new(row)));
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

    /// `onLoadMore` — the sentinel row was activated.
    ///
    /// gpui gives a `RenderOnce` element no scroll offset, so this fires on a
    /// click rather than on the sentinel scrolling into view.
    pub fn on_load_more(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_load_more = Some(std::sync::Arc::new(f));
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
        self.rows
            .iter()
            .enumerate()
            .map(|(i, r)| r.selection_key(i))
            .collect()
    }
}

impl RenderOnce for Table {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        // Column widths a resize handle has moved, and the drag in progress.
        // `use_keyed_state` takes `cx` mutably, so both precede the tokens.
        let resized =
            window.use_keyed_state(gpui::ElementId::Name("table-resized".into()), cx, |_, _| {
                Vec::<Option<Pixels>>::new()
            });
        let resized_now = resized.read(cx).clone();
        let dragging = window.use_keyed_state(
            gpui::ElementId::Name("table-resizing".into()),
            cx,
            |_, _| None::<(usize, f32, f32)>,
        );
        let drag_now = *dragging.read(cx);
        let resizable = self.columns.iter().any(|c| c.allows_resizing);
        let colors = cx.colors();
        // Copies of the tokens the tail needs: the row builder borrows `cx`
        // mutably, which ends the borrow `cx.colors()` holds.
        let muted = colors.muted;
        let default_soft = colors.default.soft();
        let secondary = self.variant == TableVariant::Secondary;
        let selectable = self.selection_mode != SelectionMode::None;
        let row_keys = self.row_keys();
        let selected_count = row_keys
            .iter()
            .filter(|k| self.selected_keys.contains(k))
            .count();

        let mut wrapper = gpui::div()
            .w_full()
            .overflow_hidden()
            .rounded(crate::util::container_radius(cx))
            .text_color(colors.foreground);

        // `primary` sits in a surface container; `secondary` is flat.
        if !secondary {
            wrapper = wrapper
                .bg(colors.surface.background)
                .border(cx.layout().border_width)
                .border_color(colors.border);
        }

        let mut table = gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .text_size(px(14.))
            .when_some(self.gap, |el, g| el.gap(g))
            .when_some(self.padding, |el, p| el.p(p));

        // ---- header ------------------------------------------------------
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
                let all = row_keys.clone();
                let none_selected = selected_count == 0;
                let all_selected = !all.is_empty() && selected_count == all.len();
                let mut box_el = Checkbox::new("table-select-all")
                    .is_selected(all_selected)
                    .is_indeterminate(!all_selected && !none_selected);
                if let Some(cb) = self.on_selection_change.clone() {
                    box_el = box_el.on_change(move |_next, window, cx| {
                        // Anything short of everything selects everything.
                        let next: Vec<SharedString> = if all_selected {
                            Vec::new()
                        } else {
                            all.clone()
                        };
                        cb(&next, window, cx);
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
            let effective = resized_now
                .get(column_index)
                .copied()
                .flatten()
                .or(column.width);
            let mut cell = gpui::div()
                .when(effective.is_none(), flex_cell)
                .when_some(effective, |c, w| c.w(w))
                .when_some(column.min_width, |c, w| c.min_w(w))
                .flex()
                .items_center()
                .gap(px(4.))
                .px(px(12.))
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
                    gpui::div()
                        .id(gpui::ElementId::Name(
                            format!("table-sort-{}", column.label).into(),
                        ))
                        .flex_1()
                        .flex()
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover))
                        .on_click(move |_, window, cx| cb(next.clone(), window, cx))
                        .child(cell)
                        .into_any_element()
                }
                _ => cell.into_any_element(),
            };

            // `allowsResizing` puts a handle on the column's trailing edge. The
            // wrapper is what keeps the handle inside the column's box.
            let cell = if column.allows_resizing {
                let held = dragging.clone();
                let start_width = effective.unwrap_or(px(160.));
                gpui::div()
                    .relative()
                    .when(effective.is_none(), flex_cell)
                    .when_some(effective, |c, w| c.w(w))
                    .child(cell)
                    .child(
                        gpui::div()
                            .id(gpui::ElementId::Name(
                                format!("table-resize-{column_index}").into(),
                            ))
                            .absolute()
                            .top(px(0.))
                            .right(px(-2.))
                            .w(px(5.))
                            .h_full()
                            .cursor(gpui::CursorStyle::ResizeLeftRight)
                            .on_mouse_down(gpui::MouseButton::Left, move |ev, _window, cx| {
                                let x = f32::from(ev.position.x);
                                held.update(cx, |v, _| {
                                    *v = Some((column_index, x, f32::from(start_width)));
                                });
                            }),
                    )
                    .into_any_element()
            } else {
                cell
            };
            header = header.child(cell);
        }
        table = table.child(header);

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
                    // 48px is the floor a header label needs to stay readable.
                    let next = (from_w + f32::from(ev.position.x) - from_x).max(48.);
                    widths.update(cx, |v, cx| {
                        if v.len() <= column {
                            v.resize(column + 1, None);
                        }
                        v[column] = Some(px(next));
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
        let _ = drag_now;

        // ---- rows --------------------------------------------------------
        let column_count = self.columns.len();
        // Depth-first, and only through the parents that are open: a nested row
        // is not rendered at all until its parent is expanded.
        let mut flat: Vec<(TableRow, usize, bool, SharedString)> = Vec::new();
        fn flatten(
            rows: Vec<TableRow>,
            depth: usize,
            path: &str,
            expanded: &[SharedString],
            out: &mut Vec<(TableRow, usize, bool, SharedString)>,
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
                out.push((row, depth, has_children, key.clone()));
                if has_children && is_open {
                    flatten(children, depth + 1, key.as_ref(), expanded, out);
                }
            }
        }
        flatten(
            std::mem::take(&mut self.rows),
            0,
            "",
            &self.expanded_keys,
            &mut flat,
        );
        // Whether any row in this table is expandable at all: a flat table
        // keeps its cells flush, rather than reserving a chevron's width.
        let tree_column_has_children = flat.iter().any(|(_, _, has, _)| *has);

        // Everything a row needs, held once. The virtual path builds its rows
        // inside a `'static` closure, so it cannot borrow `self` -- and having
        // one row builder for both paths is what keeps a virtual table drawing
        // the same row as a short one.
        let ctx = std::rc::Rc::new(RowCtx {
            widths: self
                .columns
                .iter()
                .enumerate()
                .map(|(c, col)| {
                    (
                        resized_now.get(c).copied().flatten().or(col.width),
                        col.min_width,
                    )
                })
                .collect(),
            row_header_columns: self.columns.iter().map(|c| c.is_row_header).collect(),
            tree_column: self.tree_column,
            tree_column_has_children,
            selectable,
            selection_mode: self.selection_mode,
            selected_keys: self.selected_keys.clone(),
            row_keys,
            expanded: self.expanded_keys.clone(),
            on_expanded_change: self.on_expanded_change.clone(),
            on_selection_change: self.on_selection_change.clone(),
            on_row_click: self.on_row_click.clone(),
        });

        let row_count = match &self.virtual_rows {
            Some((count, _)) if self.row_height.is_some() => *count,
            _ => flat.len(),
        };

        if let (Some(row_height), Some((count, factory))) =
            (self.row_height, self.virtual_rows.clone())
        {
            // The body scrolls inside `uniform_list`, which asks for the rows the
            // viewport shows and no others.
            let height = self.max_h.unwrap_or(px(400.));
            let body = ctx.clone();
            table = table.child(
                gpui::uniform_list(
                    gpui::ElementId::Name("table-virtual-rows".into()),
                    count,
                    move |range, _window, cx| {
                        range
                            .map(|i| {
                                let row_data = factory(i);
                                let key = row_data.selection_key(i);
                                body.row(i, row_data, 0, false, &key, Some(row_height), cx)
                            })
                            .collect::<Vec<_>>()
                    },
                )
                .h(height)
                .w_full(),
            );
        } else {
            for (i, (row_data, depth, has_children, tree_key)) in flat.into_iter().enumerate() {
                table = table.child(ctx.row(i, row_data, depth, has_children, &tree_key, None, cx));
            }
        }

        // ---- empty state -------------------------------------------------
        if row_count == 0 {
            if let Some(content) = self.empty_state {
                table = table.child(
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

        // ---- load-more sentinel ------------------------------------------
        if self.is_pending || self.on_load_more.is_some() {
            let mut sentinel = gpui::div()
                .id("table-load-more")
                .flex()
                .items_center()
                .justify_center()
                .gap(px(8.))
                .w_full()
                .py(px(12.))
                .text_size(px(13.))
                .text_color(muted);

            if self.is_pending {
                sentinel = sentinel
                    .child(
                        crate::spinner::Spinner::new("table-load-spinner")
                            .size(herogpui_core::Size::Sm),
                    )
                    .child("Loading\u{2026}");
            } else if let Some(cb) = self.on_load_more.clone() {
                let hover = default_soft;
                sentinel = sentinel
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .on_click(move |_, window, cx| cb(window, cx))
                    .child("Load more");
            }
            let _ = column_count;
            table = table.child(sentinel);
        }

        wrapper.child(table)
    }
}

/// Everything one body row needs, so both the plain and the virtualized path can
/// draw it from the same code.
struct RowCtx {
    /// `(defaultWidth or the resize, minWidth)` per column.
    widths: Vec<(Option<Pixels>, Option<Pixels>)>,
    row_header_columns: Vec<bool>,
    tree_column: usize,
    tree_column_has_children: bool,
    selectable: bool,
    selection_mode: SelectionMode,
    selected_keys: Vec<SharedString>,
    row_keys: Vec<SharedString>,
    expanded: Vec<SharedString>,
    on_expanded_change: Option<OnExpandedChange>,
    on_selection_change: Option<OnSelectionChange>,
    on_row_click: Option<OnRowClick>,
}

impl RowCtx {
    /// One body row.
    ///
    /// `fixed_h` is `Some` only on the virtual path, where every row is one
    /// `rowHeight` tall because that is the number the scroll geometry comes
    /// from.
    #[allow(clippy::too_many_arguments)]
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
        let key = self
            .row_keys
            .get(i)
            .cloned()
            .unwrap_or_else(|| tree_key.clone());
        let is_selected = self.selected_keys.contains(&key);
        let row_header_columns = &self.row_header_columns;

        let mut row = gpui::div()
            .id(gpui::ElementId::Name(format!("table-row-{i}").into()))
            .flex()
            .when_some(fixed_h, |e, h| e.h(h).w_full())
            .border_b_1()
            .border_color(colors.separator);

        if self.selectable {
            let mut cell = gpui::div()
                .flex()
                .items_center()
                .justify_center()
                .w(px(44.))
                .py(px(10.));
            let mut box_el =
                Checkbox::new(gpui::ElementId::Name(format!("table-select-{i}").into()))
                    .is_selected(is_selected);
            if let Some(cb) = self.on_selection_change.clone() {
                let current = self.selected_keys.clone();
                let key2 = key;
                let mode = self.selection_mode;
                box_el = box_el.on_change(move |_next, window, cx| {
                    let next = crate::selection::next_selection(&current, &key2, mode, false);
                    cb(&next, window, cx);
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
            let (width, min_width) = widths.get(c).copied().unwrap_or((None, None));
            let mut cell_el = gpui::div()
                .when(width.is_none(), flex_cell)
                .when_some(width, |e, w| e.w(w))
                .when_some(min_width, |e, w| e.min_w(w))
                .flex()
                .items_center()
                .gap(px(6.))
                .px(px(12.))
                .py(px(10.))
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
                            format!("table-expand-{toggle_key}").into(),
                        ))
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(18.))
                        .flex_shrink_0()
                        .cursor_pointer()
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
                    if let Some(cb) = on_expanded.clone() {
                        let key = toggle_key.clone();
                        let before = expanded_before.clone();
                        chevron = chevron.on_click(move |_, window, cx| {
                            let mut next = before.clone();
                            if let Some(at) = next.iter().position(|k| *k == key) {
                                next.remove(at);
                            } else {
                                next.push(key.clone());
                            }
                            cb(&next, window, cx);
                        });
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

        if let Some(on_click) = &self.on_row_click {
            let cb = on_click.clone();
            row = row
                .cursor_pointer()
                .hover(move |s| s.bg(colors.default.soft()))
                .on_click(move |ev, w, cx| cb(i, ev, w, cx));
        }

        row.into_any_element()
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
