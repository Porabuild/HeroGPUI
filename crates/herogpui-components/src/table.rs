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
            width: None,
            min_width: None,
        }
    }

    /// `defaultWidth` — a fixed width for this column.
    ///
    /// v3 pairs this with a resizer, which does not exist here; the width
    /// itself is just layout and does apply.
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
}

impl TableRow {
    pub fn new(cells: Vec<AnyElement>) -> Self {
        Self { key: None, cells }
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

/// HeroUI Table.
#[derive(IntoElement)]
pub struct Table {
    /// `indicator` — v3's render prop for the sort chevron, handed the
    /// `sortDirection` the column is sorted in.
    indicator: Option<Indicator>,
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
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

    /// Adds one row of cells (`Table.Row` cells).
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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = cx.colors();
        let accent = colors.accent;
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

        let mut table = gpui::div().flex().flex_col().w_full().text_size(px(14.));

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

        for column in &self.columns {
            let sorted = self
                .sort_descriptor
                .as_ref()
                .filter(|d| d.column == column.label);
            // A column with a `defaultWidth` takes it; the rest share the row.
            let mut cell = gpui::div()
                .when(column.width.is_none(), |c| c.flex_1())
                .when_some(column.width, |c, w| c.w(w))
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
            header = header.child(cell);
        }
        table = table.child(header);

        // ---- rows --------------------------------------------------------
        let column_count = self.columns.len();
        let row_count = self.rows.len();
        for (i, row_data) in self.rows.into_iter().enumerate() {
            let key = row_keys[i].clone();
            let is_selected = self.selected_keys.contains(&key);
            let row_header_columns: Vec<bool> =
                self.columns.iter().map(|c| c.is_row_header).collect();

            let mut row = gpui::div()
                .id(gpui::ElementId::Name(format!("table-row-{i}").into()))
                .flex()
                .border_b_1()
                .border_color(colors.separator);

            if selectable {
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
                    let key2 = key.clone();
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
            let widths: Vec<(Option<Pixels>, Option<Pixels>)> = self
                .columns
                .iter()
                .map(|c| (c.width, c.min_width))
                .collect();
            row = row.children(row_data.cells.into_iter().enumerate().map(|(c, cell)| {
                let (width, min_width) = widths.get(c).copied().unwrap_or((None, None));
                gpui::div()
                    .when(width.is_none(), |e| e.flex_1())
                    .when_some(width, |e, w| e.w(w))
                    .when_some(min_width, |e, w| e.min_w(w))
                    .flex()
                    .items_center()
                    .px(px(12.))
                    .py(px(10.))
                    .when(row_header_columns.get(c).copied().unwrap_or(false), |e| {
                        e.font_weight(gpui::FontWeight::MEDIUM)
                    })
                    .child(cell)
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

            table = table.child(row);
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
                        .text_color(colors.muted)
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
                .text_color(colors.muted);

            if self.is_pending {
                sentinel = sentinel
                    .child(
                        crate::spinner::Spinner::new("table-load-spinner")
                            .size(herogpui_core::Size::Sm),
                    )
                    .child("Loading\u{2026}");
            } else if let Some(cb) = self.on_load_more.clone() {
                let hover = colors.default.soft();
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
