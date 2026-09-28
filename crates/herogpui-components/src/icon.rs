//! A named, embedded Lucide icon set (HeroGPUI extension, not a HeroUI v3
//! API).
//!
//! [`IconName`] names a curated subset of [Lucide](https://lucide.dev)
//! icons, and [`Icon`] draws one at a size and colour. The SVGs are copied
//! verbatim from the official `lucide-static` [`LUCIDE_VERSION`] release
//! into this crate's `assets/lucide/` and compiled in with `include_bytes!`,
//! so [`HeroGpuiAssets`](crate::HeroGpuiAssets) serves every variant with no
//! file or feature to set up:
//!
//! ```
//! use herogpui_components::{Icon, IconName};
//! use gpui::{px, rgb};
//!
//! let _search = Icon::new(IconName::Search);
//! let _big_red = Icon::new(IconName::Heart).size(px(24.)).color(rgb(0xe11d48));
//!
//! // Any builder that takes an icon path takes a name too.
//! let path: gpui::SharedString = IconName::Folder.into();
//! assert_eq!(path.as_ref(), "herogpui/icons/lucide/folder.svg");
//! ```
//!
//! The chrome paths in [`icons`](crate::icons) are separate and unchanged:
//! those are HeroUI's own glyphs, drawn by the components, and keep their
//! `herogpui/icons/<name>.svg` paths. Lucide icons live one level down, under
//! [`LUCIDE_ICON_PREFIX`], so the two sets never collide.
//!
//! Lucide is ISC-licensed, and a few of its icons derive from Feather (MIT);
//! the full text ships beside the SVGs as `assets/lucide/LICENSE` and the
//! attribution is in this crate's `NOTICE`.
//!
//! To add an icon, copy `icons/<name>.svg` byte for byte out of the same
//! `lucide-static` tarball into `assets/lucide/` and add one
//! `Variant => "name"` line to the list below; `tests/icon.rs` fails for a
//! variant without a file and for a file without a variant.

use gpui::{prelude::*, px, svg, App, Hsla, IntoElement, Pixels, RenderOnce, SharedString, Window};
use herogpui_theme::ActiveTheme;

/// The `lucide-static` release the embedded SVGs were copied from.
pub const LUCIDE_VERSION: &str = "1.31.0";

/// The asset-path directory every [`IconName::path`] lives in.
pub const LUCIDE_ICON_PREFIX: &str = "herogpui/icons/lucide";

/// The size [`Icon`] draws at unless told otherwise — the 16px the
/// components' own chrome icons use.
pub const DEFAULT_ICON_SIZE: Pixels = px(16.);

macro_rules! lucide_icons {
    ($($variant:ident => $name:literal),* $(,)?) => {
        /// A Lucide icon embedded in this crate (HeroGPUI extension).
        ///
        /// Each variant is the Lucide icon of the same name in PascalCase
        /// (`circle-check` is [`IconName::CircleCheck`]). The set is
        /// curated, not all of Lucide, and grows in minor releases, which is
        /// why the enum is `#[non_exhaustive]`.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[non_exhaustive]
        pub enum IconName {
            $(
                #[doc = concat!("Lucide `", $name, "`.")]
                $variant,
            )*
        }

        impl IconName {
            /// Every icon, in name order.
            pub const ALL: &'static [IconName] = &[$(IconName::$variant),*];

            /// Lucide's own kebab-case name, e.g. `"circle-check"`.
            pub const fn name(self) -> &'static str {
                match self {
                    $(IconName::$variant => $name,)*
                }
            }

            /// The asset path [`HeroGpuiAssets`](crate::HeroGpuiAssets)
            /// serves this icon under, e.g.
            /// `"herogpui/icons/lucide/circle-check.svg"`.
            pub const fn path(self) -> &'static str {
                match self {
                    $(IconName::$variant => concat!("herogpui/icons/lucide/", $name, ".svg"),)*
                }
            }

            /// The embedded SVG document, byte-identical to `lucide-static`.
            pub const fn svg(self) -> &'static [u8] {
                match self {
                    $(IconName::$variant => include_bytes!(concat!("../assets/lucide/", $name, ".svg")),)*
                }
            }
        }
    };
}

lucide_icons! {
    Activity => "activity",
    AlarmClock => "alarm-clock",
    Archive => "archive",
    ArrowDown => "arrow-down",
    ArrowDownLeft => "arrow-down-left",
    ArrowDownRight => "arrow-down-right",
    ArrowDownToLine => "arrow-down-to-line",
    ArrowLeft => "arrow-left",
    ArrowLeftRight => "arrow-left-right",
    ArrowRight => "arrow-right",
    ArrowUp => "arrow-up",
    ArrowUpDown => "arrow-up-down",
    ArrowUpLeft => "arrow-up-left",
    ArrowUpRight => "arrow-up-right",
    ArrowUpToLine => "arrow-up-to-line",
    AtSign => "at-sign",
    Ban => "ban",
    Battery => "battery",
    Bell => "bell",
    BellOff => "bell-off",
    Bold => "bold",
    Book => "book",
    BookOpen => "book-open",
    Bookmark => "bookmark",
    Bot => "bot",
    Box => "box",
    Briefcase => "briefcase",
    Bug => "bug",
    Building => "building",
    Calendar => "calendar",
    CalendarDays => "calendar-days",
    Camera => "camera",
    ChartBar => "chart-bar",
    ChartColumn => "chart-column",
    ChartLine => "chart-line",
    ChartPie => "chart-pie",
    Check => "check",
    CheckCheck => "check-check",
    ChevronDown => "chevron-down",
    ChevronLeft => "chevron-left",
    ChevronRight => "chevron-right",
    ChevronUp => "chevron-up",
    ChevronsDown => "chevrons-down",
    ChevronsLeft => "chevrons-left",
    ChevronsRight => "chevrons-right",
    ChevronsUp => "chevrons-up",
    ChevronsUpDown => "chevrons-up-down",
    Circle => "circle",
    CircleAlert => "circle-alert",
    CircleCheck => "circle-check",
    CircleDot => "circle-dot",
    CircleHelp => "circle-help",
    CircleMinus => "circle-minus",
    CirclePlus => "circle-plus",
    CircleUser => "circle-user",
    CircleX => "circle-x",
    Clipboard => "clipboard",
    ClipboardCheck => "clipboard-check",
    ClipboardCopy => "clipboard-copy",
    Clock => "clock",
    Cloud => "cloud",
    CloudDownload => "cloud-download",
    CloudUpload => "cloud-upload",
    Code => "code",
    CodeXml => "code-xml",
    Columns2 => "columns-2",
    Command => "command",
    Compass => "compass",
    Contrast => "contrast",
    Copy => "copy",
    CornerDownLeft => "corner-down-left",
    Cpu => "cpu",
    CreditCard => "credit-card",
    Crop => "crop",
    Database => "database",
    DollarSign => "dollar-sign",
    Download => "download",
    Droplet => "droplet",
    Ellipsis => "ellipsis",
    EllipsisVertical => "ellipsis-vertical",
    Expand => "expand",
    ExternalLink => "external-link",
    Eye => "eye",
    EyeOff => "eye-off",
    File => "file",
    FileCode => "file-code",
    FilePlus => "file-plus",
    FileText => "file-text",
    Files => "files",
    Film => "film",
    Flag => "flag",
    Folder => "folder",
    FolderOpen => "folder-open",
    FolderPlus => "folder-plus",
    Funnel => "funnel",
    Gift => "gift",
    GitBranch => "git-branch",
    GitCommitHorizontal => "git-commit-horizontal",
    GitMerge => "git-merge",
    GitPullRequest => "git-pull-request",
    Globe => "globe",
    GraduationCap => "graduation-cap",
    Grid2x2 => "grid-2x2",
    GripHorizontal => "grip-horizontal",
    GripVertical => "grip-vertical",
    HardDrive => "hard-drive",
    Hash => "hash",
    Headphones => "headphones",
    Heart => "heart",
    History => "history",
    House => "house",
    Image => "image",
    Inbox => "inbox",
    Info => "info",
    Italic => "italic",
    Key => "key",
    Keyboard => "keyboard",
    Languages => "languages",
    Laptop => "laptop",
    Layers => "layers",
    LayoutDashboard => "layout-dashboard",
    LayoutGrid => "layout-grid",
    LayoutList => "layout-list",
    Lightbulb => "lightbulb",
    Link => "link",
    Link2 => "link-2",
    List => "list",
    ListChecks => "list-checks",
    ListFilter => "list-filter",
    ListOrdered => "list-ordered",
    Loader => "loader",
    LoaderCircle => "loader-circle",
    Lock => "lock",
    LockOpen => "lock-open",
    LogIn => "log-in",
    LogOut => "log-out",
    Mail => "mail",
    Map => "map",
    MapPin => "map-pin",
    Maximize => "maximize",
    Maximize2 => "maximize-2",
    Menu => "menu",
    MessageCircle => "message-circle",
    MessageSquare => "message-square",
    Mic => "mic",
    MicOff => "mic-off",
    Minimize => "minimize",
    Minimize2 => "minimize-2",
    Minus => "minus",
    Monitor => "monitor",
    Moon => "moon",
    MousePointer => "mouse-pointer",
    Move => "move",
    OctagonAlert => "octagon-alert",
    Package => "package",
    Palette => "palette",
    PanelLeft => "panel-left",
    PanelRight => "panel-right",
    Paperclip => "paperclip",
    Pause => "pause",
    Pencil => "pencil",
    Phone => "phone",
    Pin => "pin",
    Pipette => "pipette",
    Play => "play",
    Plus => "plus",
    Power => "power",
    Printer => "printer",
    Puzzle => "puzzle",
    Quote => "quote",
    Redo2 => "redo-2",
    RefreshCw => "refresh-cw",
    Repeat => "repeat",
    Rocket => "rocket",
    RotateCcw => "rotate-ccw",
    RotateCw => "rotate-cw",
    Rows2 => "rows-2",
    Rss => "rss",
    Save => "save",
    Scissors => "scissors",
    Search => "search",
    Send => "send",
    Server => "server",
    Settings => "settings",
    Settings2 => "settings-2",
    Share => "share",
    Share2 => "share-2",
    Shield => "shield",
    ShieldCheck => "shield-check",
    ShoppingBag => "shopping-bag",
    ShoppingCart => "shopping-cart",
    Shrink => "shrink",
    Shuffle => "shuffle",
    SkipBack => "skip-back",
    SkipForward => "skip-forward",
    SlidersHorizontal => "sliders-horizontal",
    Smartphone => "smartphone",
    Smile => "smile",
    Sparkles => "sparkles",
    Square => "square",
    SquareCheck => "square-check",
    SquarePen => "square-pen",
    Star => "star",
    Strikethrough => "strikethrough",
    Sun => "sun",
    SunMoon => "sun-moon",
    Table => "table",
    Tablet => "tablet",
    Tag => "tag",
    Tags => "tags",
    Terminal => "terminal",
    TextAlignCenter => "text-align-center",
    TextAlignEnd => "text-align-end",
    TextAlignStart => "text-align-start",
    ThumbsDown => "thumbs-down",
    ThumbsUp => "thumbs-up",
    Timer => "timer",
    Trash => "trash",
    Trash2 => "trash-2",
    TrendingDown => "trending-down",
    TrendingUp => "trending-up",
    TriangleAlert => "triangle-alert",
    Truck => "truck",
    Type => "type",
    Underline => "underline",
    Undo2 => "undo-2",
    Unlink => "unlink",
    Upload => "upload",
    User => "user",
    UserCheck => "user-check",
    UserMinus => "user-minus",
    UserPlus => "user-plus",
    Users => "users",
    Video => "video",
    Volume2 => "volume-2",
    VolumeX => "volume-x",
    Wallet => "wallet",
    WandSparkles => "wand-sparkles",
    Wifi => "wifi",
    WifiOff => "wifi-off",
    Wrench => "wrench",
    X => "x",
    Zap => "zap",
    ZoomIn => "zoom-in",
    ZoomOut => "zoom-out",
}

impl IconName {
    /// The icon Lucide calls `name` (kebab-case, as in [`IconName::name`]).
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .binary_search_by(|icon| icon.name().cmp(name))
            .ok()
            .map(|index| Self::ALL[index])
    }

    /// The icon whose [`IconName::path`] is `path`.
    pub fn from_path(path: &str) -> Option<Self> {
        path.strip_prefix(LUCIDE_ICON_PREFIX)?
            .strip_prefix('/')?
            .strip_suffix(".svg")
            .and_then(Self::from_name)
    }
}

impl From<IconName> for SharedString {
    fn from(icon: IconName) -> Self {
        SharedString::new_static(icon.path())
    }
}

impl From<IconName> for Icon {
    fn from(icon: IconName) -> Self {
        Icon::new(icon)
    }
}

/// One icon, drawn as a GPUI `svg()` (HeroGPUI extension).
///
/// Sized [`DEFAULT_ICON_SIZE`] and painted in the theme's `foreground`
/// unless told otherwise — gpui's `svg()` does not inherit the surrounding
/// text colour, so an icon on an accent or danger fill needs `.color(..)`.
/// The icon is decorative and reports no accessibility node, as
/// `lucide-react` marks its `<svg>` `aria-hidden` when it is given no label:
/// name the control that holds it instead.
#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    size: Pixels,
    color: Option<Hsla>,
}

impl Icon {
    /// An embedded Lucide icon.
    pub fn new(icon: IconName) -> Self {
        Self::from_path(icon)
    }

    /// Any SVG the registered asset source serves: one of the
    /// [`icons`](crate::icons) chrome paths, or an application's own.
    pub fn from_path(path: impl Into<SharedString>) -> Self {
        Self {
            path: path.into(),
            size: DEFAULT_ICON_SIZE,
            color: None,
        }
    }

    /// The width and height of the square the icon draws in.
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into();
        self
    }

    /// The paint colour (default: the theme's `foreground`).
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or(cx.colors().foreground);
        svg()
            .path(self.path)
            .size(self.size)
            .flex_none()
            .text_color(color)
    }
}
