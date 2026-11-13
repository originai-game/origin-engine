//! The inspector's UI view (docs/ui.md §6.4): the visual debugger of the
//! game's UI in a development build. Pick picks the element under the
//! pointer (`Ui::debug_pick`); the panel shows its box model, the layout
//! boundaries it sets, its computed styles with the line each was written
//! on, the style rules that dress it and its bindings
//! (`Ui::debug_element`), and the frame draws its margin, border, padding
//! and content boxes over the game. Layout bounds outlines every element
//! drawn. It reads the game's UI only while this view is open, and only
//! the element picked (or under the pointer while picking).

use super::Inspector;
use og_ui::{Insets, LabelDesc, Rect, ScrollDesc, Size, Srgba, Ui, UiDebugElement};

/// What the view read of the game's UI this frame, in CSS px.
#[derive(Clone, Debug, Default)]
pub(super) struct View {
    pub element: Option<UiDebugElement>,
    pub hover: Option<UiDebugElement>,
    /// Every element drawn, while Layout bounds is on.
    pub bounds: Vec<Rect>,
}

/// The tab the view is.
pub(super) const TAB: usize = 2;

/// Reads the game's UI for this frame: the element a pick asked for, the
/// one under the pointer while picking, and the bounds when asked.
pub(super) fn read(ins: &mut Inspector, game: &Ui) -> View {
    let u = game.frame().unit.max(1.0);
    let at = |x: f32, y: f32| game.debug_pick(x / u, y / u);
    if let Some((x, y)) = ins.ui_pick.take() {
        ins.ui_selected = at(x, y);
        ins.status = match &ins.ui_selected {
            Some(p) => format!("Picked {p}"),
            None => "No UI element under the pointer".into(),
        };
    }
    let scaled = |e: UiDebugElement| scale(e, u);
    let element = ins
        .ui_selected
        .as_deref()
        .and_then(|p| game.debug_element(p))
        .map(scaled);
    let hover = if ins.picking {
        let p = ins.input.pointer;
        at(p.x, p.y)
            .and_then(|p| game.debug_element(&p))
            .map(scaled)
    } else {
        None
    };
    let bounds = if ins.ui_bounds {
        game.elements()
            .iter()
            .filter(|e| !e.clipped)
            .map(|e| Rect {
                x: e.x * u,
                y: e.y * u,
                width: e.width * u,
                height: e.height * u,
            })
            .collect()
    } else {
        Vec::new()
    };
    View {
        element,
        hover,
        bounds,
    }
}

fn scale(mut e: UiDebugElement, u: f32) -> UiDebugElement {
    let r = |r: Rect| Rect {
        x: r.x * u,
        y: r.y * u,
        width: r.width * u,
        height: r.height * u,
    };
    let i = |i: Insets| Insets {
        left: i.left * u,
        top: i.top * u,
        right: i.right * u,
        bottom: i.bottom * u,
    };
    e.border_box = r(e.border_box);
    e.content = r(e.content);
    e.margin = i(e.margin);
    e.border = i(e.border);
    e.padding = i(e.padding);
    e
}

/// The panel's detail: the picked element, or how to pick one.
/// `filter` (lower case) keeps the styles, rules and bindings whose name
/// holds it.
pub(super) fn detail(ui: &mut Ui, view: &View, filter: &str, bounds: &mut bool) {
    let kept = |name: &str| filter.is_empty() || name.to_lowercase().contains(filter);
    let t = *ui.theme();
    let p = t.palette;
    let r = ui.check_field(
        "LayoutBounds",
        *bounds,
        &og_ui::CheckFieldDesc {
            label: "Layout bounds".into(),
            ..Default::default()
        },
    );
    *bounds = r.on;
    ui.scroll(
        "Detail",
        ScrollDesc {
            width: Size::grow(),
            height: Size::grow(),
            gap: t.space.xs,
            ..ScrollDesc::default()
        },
        |ui| {
            let Some(e) = &view.element else {
                text(ui, "Empty", "Pick an element of the game's UI", p.ink_muted);
                return;
            };
            text(ui, "Path", &e.path, p.ink);
            text(ui, "Kind", &format!("<{}>", e.kind), p.ink_muted);
            heading(ui, "BoxHead", "Box model");
            let b = &e.border_box;
            for (id, name, i) in [
                ("Margin", "margin", e.margin),
                ("Border", "border", e.border),
                ("Padding", "padding", e.padding),
            ] {
                text(ui, id, &format!("{name}  {}", insets(i)), p.ink);
            }
            text(
                ui,
                "Content",
                &format!(
                    "content  {} × {}  (box {} × {} at {}, {})",
                    px(e.content.width),
                    px(e.content.height),
                    px(b.width),
                    px(b.height),
                    px(b.x),
                    px(b.y)
                ),
                p.ink,
            );
            if !e.boundaries.is_empty() {
                text(
                    ui,
                    "Boundaries",
                    &format!("layout boundary: {}", e.boundaries.join(", ")),
                    p.ink,
                );
            }
            heading(ui, "StylesHead", "Computed styles");
            if e.styles.is_empty() {
                text(ui, "NoStyles", "No document declarations", p.ink_muted);
            }
            for (k, s) in e.styles.iter().enumerate().filter(|(_, s)| kept(&s.name)) {
                text(
                    ui,
                    &format!("Style{k}"),
                    &format!("{}: {}   {}", s.name, s.value, s.source),
                    p.ink,
                );
            }
            heading(ui, "RulesHead", "Style rules");
            if e.rules.is_empty() {
                text(ui, "NoRules", "No style rule dresses it", p.ink_muted);
            }
            for (k, s) in e.rules.iter().enumerate().filter(|(_, s)| kept(&s.name)) {
                text(
                    ui,
                    &format!("Rule{k}"),
                    &format!("{}   {}", s.name, s.source),
                    p.ink,
                );
            }
            heading(ui, "BindingsHead", "Bindings");
            if e.bindings.is_empty() {
                text(ui, "NoBindings", "No bindings", p.ink_muted);
            }
            for (k, (prop, expr, value)) in
                e.bindings.iter().enumerate().filter(|(_, b)| kept(&b.0))
            {
                text(
                    ui,
                    &format!("Binding{k}"),
                    &format!("{prop} = {expr} → {value}"),
                    p.ink,
                );
            }
        },
    );
}

fn px(v: f32) -> String {
    let r = (v * 10.0).round() / 10.0;
    if r.fract() == 0.0 {
        format!("{r:.0}")
    } else {
        format!("{r:.1}")
    }
}

/// CSS order: top right bottom left.
fn insets(i: Insets) -> String {
    format!(
        "{} {} {} {}",
        px(i.top),
        px(i.right),
        px(i.bottom),
        px(i.left)
    )
}

fn heading(ui: &mut Ui, id: &str, text_: &str) {
    let t = *ui.theme();
    ui.label(
        id,
        text_,
        LabelDesc {
            step: Some(t.type_ramp.sm),
            color: Some(t.palette.ink),
            ..LabelDesc::default()
        },
    );
}

fn text(ui: &mut Ui, id: &str, s: &str, color: Srgba) {
    let t = *ui.theme();
    ui.label(
        id,
        s,
        LabelDesc {
            step: Some(t.type_ramp.xs),
            color: Some(color),
            width: Size::grow(),
            ..LabelDesc::default()
        },
    );
}
