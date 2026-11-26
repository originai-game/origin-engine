//! Rive source settings. Source artboard/font/machine existence stays with the
//! importer; roles, fit and alignment are closed author vocabularies.
use super::*;

input!(Rive {
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    source: String,
    /// Omitted: all artboards. Explicit selection retains authored order.
    artboards: Option<Vec<String>>,
    #[serde(default)]
    settings: BTreeMap<String, Artboard>,
    #[serde(default)]
    fonts: BTreeMap<String, FontSource>,
});
input!(FontSource {
    /// Omitted: use the embedded font with this source name.
    #[schemars(length(min = 1), extend("x-og-input" = "bytes"))]
    source: Option<String>,
});
input!(Artboard {
    state_machine: Option<String>, role: Option<Role>, name: Option<String>,
    fit: Option<Fit>, align: Option<Align>,
});
vocabulary!(Fit { Contain => "contain", Cover => "cover", Fill => "fill", None => "none", ScaleDown => "scale-down", FitWidth => "fit-width", FitHeight => "fit-height" });
vocabulary!(Align { Center => "center", Top => "top", Bottom => "bottom", Left => "left", Right => "right", TopLeft => "top-left", TopRight => "top-right", BottomLeft => "bottom-left", BottomRight => "bottom-right" });
vocabulary!(Role {
    Button => "button", Link => "link", Text => "text", Heading => "heading", Image => "image",
    Checkbox => "checkbox", Radio => "radio", Switch => "switch", Slider => "slider",
    Spinbutton => "spinbutton", Textbox => "textbox", Combobox => "combobox", Listbox => "listbox",
    Option => "option", Tab => "tab", Tablist => "tablist", Tabpanel => "tabpanel",
    Radiogroup => "radiogroup", Progressbar => "progressbar", Meter => "meter", Timer => "timer",
    Status => "status", Alert => "alert", Log => "log", Dialog => "dialog", Alertdialog => "alertdialog",
    Tooltip => "tooltip", Separator => "separator", Group => "group", Region => "region",
    Navigation => "navigation", Toolbar => "toolbar", Menu => "menu", Menubar => "menubar",
    Menuitem => "menuitem", Menuitemcheckbox => "menuitemcheckbox", Menuitemradio => "menuitemradio",
    List => "list", Listitem => "listitem", Tree => "tree", Treeitem => "treeitem", Table => "table",
    Grid => "grid", Row => "row", Rowheader => "rowheader", Columnheader => "columnheader", Cell => "cell",
    Gridcell => "gridcell", Figure => "figure", Scrollbar => "scrollbar",
});
