//! The engine's default pack, from the design tokens
//! (`packages/design/tokens/*.css`): each scheme's token table resolved,
//! every colour evaluated (the seeds' relative colours included) and the
//! values written as theme JSON. `default/pack.json` is this module's
//! output, checked in and held equal by a test; regenerate it with
//! `crates/og-ui-next/scripts/gen-pack.sh`.
//!
//! Names follow the tokens: a theme entry is the token's name without its
//! family's prefix (`--color-fg-muted` is the colour `fg-muted`,
//! `--radius-card` the radius `card`, `--font-scale-body-size` the font
//! size `body`), so the theme speaks the design language's words.

use super::Scheme;
use crate::tokens::Tokens;
use serde_json::{Map, Value, json};

const SEED: &str = include_str!("../../../../packages/design/tokens/seed.css");
const COLOR: &str = include_str!("../../../../packages/design/tokens/color.css");
const MOTION: &str = include_str!("../../../../packages/design/tokens/motion.css");

/// The token files the theme reads, in the order the design applies them.
/// Only the generator ([`default_pack_json`]) reads them all; at run time
/// the palette ([`colors`]) reads the seeds and colours and the durations
/// ([`durations`]) the motion tokens, so a game's module carries no more
/// of the design's CSS than those.
const FILES: [&str; 7] = [
    SEED,
    COLOR,
    include_str!("../../../../packages/design/tokens/type.css"),
    include_str!("../../../../packages/design/tokens/space.css"),
    include_str!("../../../../packages/design/tokens/effect.css"),
    MOTION,
    include_str!("../../../../packages/design/tokens/role.css"),
];

/// The palette's files: the seeds and the colours derived from them.
const PALETTE: [&str; 2] = [SEED, COLOR];

/// The generated default pack.
pub const DEFAULT_PACK: &str = include_str!("../../default/pack.json");

/// The token table of `files` in each scheme, with `seeds` (CSS setting
/// `--seed-*`) over the design's own.
fn tables(files: &[&str], seeds: &str) -> [Tokens; 2] {
    let mut t = Tokens::parse(&files.concat());
    if !seeds.is_empty() {
        t.over(seeds);
    }
    Scheme::ALL.map(|s| t.scheme(s.name()))
}

/// The design's motion durations (`--motion-duration-<name>`) by name, in
/// milliseconds: what og.ui.theme answers as `durations`.
pub fn durations() -> std::collections::BTreeMap<String, f32> {
    const PREFIX: &str = "--motion-duration-";
    let [t, _] = tables(&[MOTION], "");
    names(&[MOTION])
        .into_iter()
        .filter_map(|name| {
            let key = name.strip_prefix(PREFIX)?.to_owned();
            let v = t.get(&name)?.trim();
            let ms = match v.strip_suffix("ms") {
                Some(n) => n.trim().parse::<f32>().ok()?,
                None => v.strip_suffix('s')?.trim().parse::<f32>().ok()? * 1000.0,
            };
            Some((key, ms))
        })
        .collect()
}

/// Every token's name in `files`, in their order.
fn names(files: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for file in files {
        for (_, body) in crate::sheet::blocks(file) {
            for (name, _) in crate::sheet::declarations(body) {
                if name.starts_with("--") && !out.iter().any(|n| n == name) {
                    out.push(name.to_owned());
                }
            }
        }
    }
    out
}

/// A resolved colour as `#rrggbb` or `#rrggbbaa`.
fn hex(text: &str) -> Option<String> {
    let c = og_css::color::parse_str(text).ok()?.literal()?;
    let [r, g, b] = c.rgb;
    let a = (c.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    Some(match a {
        255 => format!("#{r:02x}{g:02x}{b:02x}"),
        _ => format!("#{r:02x}{g:02x}{b:02x}{a:02x}"),
    })
}

/// The design palette (`--color-*`) in each scheme, derived from `seeds`.
pub fn colors(seeds: &str) -> [Map<String, Value>; 2] {
    let names = names(&PALETTE);
    tables(&PALETTE, seeds).map(|t| {
        let mut out = Map::new();
        for n in &names {
            let Some(name) = n.strip_prefix("--color-") else {
                continue;
            };
            if let Some(c) = t.get(n).and_then(hex) {
                out.insert(name.to_owned(), Value::String(c));
            }
        }
        out
    })
}

/// `12px` or `12` as points.
fn px(text: &str) -> Option<f64> {
    let t = text.trim();
    t.strip_suffix("px").unwrap_or(t).trim().parse().ok()
}

/// `-0.01em` as ems.
fn em(text: &str) -> Option<f64> {
    text.trim().strip_suffix("em")?.parse().ok()
}

fn round(v: f64) -> Value {
    json!((v * 10000.0).round() / 10000.0)
}

/// `cubic-bezier(a, b, c, d)` as its four numbers.
fn bezier(text: &str) -> Option<Value> {
    let inner = text
        .trim()
        .strip_prefix("cubic-bezier(")?
        .strip_suffix(')')?;
    let n: Vec<f64> = inner
        .split(',')
        .map(|x| x.trim().parse().ok())
        .collect::<Option<_>>()?;
    (n.len() == 4).then(|| json!(n))
}

/// A box shadow with each layer's colour evaluated (a token's relative
/// colour or `var()` as `#rrggbbaa`), which NSH's theme reads.
fn shadow(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0;
    let mut word = String::new();
    let flush = |word: &mut String, out: &mut String| {
        match hex(word).filter(|_| word.contains('(')) {
            Some(c) => out.push_str(&c),
            None => out.push_str(word),
        }
        word.clear();
    };
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ' ' | ',' if depth == 0 => {
                flush(&mut word, &mut out);
                out.push(c);
                continue;
            }
            _ => {}
        }
        word.push(c);
    }
    flush(&mut word, &mut out);
    out
}

/// The theme in one scheme's table.
fn theme(t: &Tokens, names: &[String]) -> Map<String, Value> {
    let mut sections: Map<String, Value> = Map::new();
    let mut put = |section: &str, name: &str, v: Value| {
        let s = sections
            .entry(section)
            .or_insert_with(|| Value::Object(Map::new()));
        s.as_object_mut().unwrap().insert(name.to_owned(), v);
    };
    for n in names {
        let Some(v) = t.get(n) else { continue };
        let v = v.trim();
        let after = |p: &str| n.strip_prefix(p);
        if let Some(name) = after("--color-") {
            if let Some(c) = hex(v) {
                put("colors", name, Value::String(c));
            }
        } else if let Some(name) = after("--radius-") {
            if let Some(r) = px(v) {
                put("radius", name, json!(r));
            }
        } else if let Some(step) = after("--font-scale-").and_then(|s| s.strip_suffix("-size")) {
            let line = t
                .get(&format!("--font-scale-{step}-line-height"))
                .and_then(px);
            if let (Some(size), Some(line)) = (px(v), line) {
                put("fontSize", step, json!([size, round(line / size)]));
            }
        } else if let Some(step) =
            after("--font-scale-").and_then(|s| s.strip_suffix("-letter-spacing"))
        {
            if let Some(e) = em(v) {
                put("tracking", step, round(e));
            }
        } else if let Some(name) = after("--font-tracking-") {
            if let Some(e) = em(v) {
                put("tracking", name, round(e));
            }
        } else if let Some(name) = after("--font-leading-") {
            if let Ok(l) = v.parse::<f64>() {
                put("leading", name, round(l));
            }
        } else if let Some(name) = after("--font-weight-") {
            if let Ok(w) = v.parse::<u16>() {
                put("fontWeight", name, json!(w));
            }
        } else if let Some(name) = ["sans", "display", "mono", "rounded"]
            .iter()
            .find(|f| n == &format!("--font-{f}"))
        {
            put("fontFamily", name, Value::String(v.to_owned()));
        } else if n == "--role-tag-halo" {
            // The halo text over the scene wears: `text-shadow-tag`.
            put("textShadow", "tag", Value::String(shadow(v)));
        } else if let Some(name) = after("--inset-shadow-") {
            put("insetShadow", name, Value::String(shadow(v)));
        } else if let Some(name) = after("--shadow-") {
            put("shadow", name, Value::String(shadow(v)));
        } else if let Some(name) = after("--motion-easing-") {
            // The springs are `linear()` curves: the motion system's, not a
            // Tailwind ease.
            if let Some(b) = bezier(v) {
                put("ease", name, b);
            }
        } else if let Some(name) = after("--measure-") {
            if let Some(w) = px(v) {
                put("container", name, json!(w));
            }
        } else if let Some(name) = after("--vars-material-blur") {
            let name = name.strip_prefix('-').unwrap_or("regular");
            if let Some(b) = px(v) {
                put("blur", name, json!(b));
            }
        }
    }
    // One rem is the body type step (docs/ui.md, Units), not NSH's 16:
    // a bracketed `[1.143rem]` is 16 px of the design, as `p-4` is.
    if let Some(body) = t.get("--font-scale-body-size").and_then(px) {
        sections.insert("rem".to_owned(), json!(body));
    }
    sections
}

/// Two schemes' values as one: the value where they agree, else a pair.
pub(crate) fn join(light: &Value, dark: &Value) -> Value {
    match (light, dark) {
        (Value::Object(l), Value::Object(d)) => {
            let mut out = Map::new();
            for (k, lv) in l {
                match d.get(k) {
                    Some(dv) => out.insert(k.clone(), join(lv, dv)),
                    None => out.insert(k.clone(), lv.clone()),
                };
            }
            for (k, dv) in d {
                if !l.contains_key(k) {
                    out.insert(k.clone(), dv.clone());
                }
            }
            Value::Object(out)
        }
        (l, d) if l == d => l.clone(),
        (l, d) => json!({ "light": l, "dark": d }),
    }
}

/// The slots a pack dresses (`variants` keys), each with what it is and
/// the default pack's own dressing. The default pack declares each, so a
/// component's `packClass` on one resolves under every pack; a slot no
/// layer names stays an unknown class. A HUD plate's default is a nearly
/// opaque ground with its ink, so its text reads over any scene: a
/// component's translucent or cleared fill under it is overruled. A row
/// and a table head are opaque grounds with their inks for the same
/// reason: the lists, tables and feeds that sit over the scene.
pub const SLOTS: &[(&str, &str, &str)] = &[
    (
        "Press.feedback",
        "the squash a control that answers a press shows under it, unless its own classes move it under active (its default is the motion tokens': Timing::press)",
        "",
    ),
    (
        "Screen.frame",
        "a full-screen built-in's root: how it enters and leaves (Timing::screen)",
        "",
    ),
    (
        "Modal.frame",
        "a dialog or popup: how it pops in and leaves (Timing::modal)",
        "",
    ),
    (
        "List.cascade",
        "a list whose rows enter one after another when it shows or gains rows (Timing::cascade)",
        "",
    ),
    (
        "List.item",
        "a row of a list with List.cascade: how it enters in its turn",
        "animate-rise",
    ),
    (
        "Reward.pop",
        "a reward or a reveal arriving: how it pops in (in its turn under List.cascade)",
        "animate-pop",
    ),
    (
        "Tab.indicator",
        "the mark under a selected tab or segment, sliding from the one it replaces (Timing::indicator)",
        "",
    ),
    (
        "Bar.fill",
        "a bar's or meter's fill: how it eases to a new value (Timing::bar)",
        "",
    ),
    (
        "Bar.trail",
        "the trail behind a bar's fill: what was just lost, draining after the fill (Timing::trail)",
        "",
    ),
    (
        "Attention.cue",
        "what a control plays once when it turns from disabled to enabled",
        "animate-nudge",
    ),
    (
        "Count.roll",
        "a number that rolls to a new value: on the Text that holds it, it counts up or down on the count row (Timing::count)",
        "",
    ),
    (
        "Modal.scrim",
        "the scrim behind a dialog: what dims the screen and how it fades in and out, apart from the dialog's pop (Timing::scrim)",
        "",
    ),
    (
        "Focus.ring",
        "the ring a control shows while a keyboard or pad focuses it, unless its own classes style focus-visible",
        "outline-2 outline-focus -outline-offset-2",
    ),
    (
        "Button.plate",
        "a button's body: rune-kit's neutral variant, and the states every variant shares (hover, pressed, a toggle's aria-pressed, selected, disabled, aria-busy)",
        "rounded-button text-fg bg-raised-lo shadow-raise inset-shadow-sunk inset-ring-1 inset-ring-rim transition duration-317 hover:bg-none hover:bg-raised-hover-lo hover:shadow-plate hover:inset-ring-well-rim active:bg-none active:bg-raised-pressed aria-pressed:bg-none aria-pressed:bg-border-strong aria-pressed:inset-ring-2 aria-pressed:inset-ring-accent selected:bg-none selected:bg-border-strong selected:inset-ring-2 selected:inset-ring-accent disabled:text-fg-subtle disabled:bg-none disabled:bg-fill disabled:shadow-none disabled:inset-ring-1 disabled:inset-ring-rim-soft aria-busy:opacity-80",
    ),
    (
        "Button.plate.primary",
        "the main action (rune-kit's submit)",
        "text-accent-fg bg-linear-to-b from-sheen to-transparent to-48% bg-accent inset-shadow-none inset-ring-0 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-accent active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-accent",
    ),
    (
        "Button.plate.danger",
        "a destructive action",
        "text-danger-fg bg-linear-to-b from-sheen to-transparent to-48% bg-danger inset-shadow-none inset-ring-0 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-danger active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-danger",
    ),
    (
        "Button.plate.warning",
        "an action with a cost or a risk",
        "text-warn-fg bg-linear-to-b from-sheen to-transparent to-48% bg-warn inset-shadow-none inset-ring-0 hover:bg-linear-to-b hover:from-sheen hover:to-lit hover:to-60% hover:bg-warn active:bg-linear-to-b active:from-shade active:to-transparent active:to-60% active:bg-warn",
    ),
    (
        "Button.plate.contrast",
        "the inverse of the page: a light button on a dark page",
        "text-fg-inverse bg-none bg-fg shadow-raise inset-shadow-none inset-ring-0 hover:bg-none hover:bg-fg active:bg-none active:bg-fg-muted",
    ),
    (
        "Button.plate.outline",
        "a rim on the page, no fill",
        "text-fg bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0 inset-ring-1 inset-ring-border-strong hover:inset-ring-border-strong",
    ),
    (
        "Button.plate.ghost",
        "no plate until hovered or pressed",
        "text-fg bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0",
    ),
    (
        "Button.plate.ghost-muted",
        "a ghost in the muted ink: a secondary action beside others",
        "text-fg-muted hover:text-fg bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0",
    ),
    (
        "Button.plate.ghost-faint",
        "a ghost in the faint ink: the least of a set (Skip)",
        "text-fg-subtle hover:text-fg-muted bg-none bg-transparent shadow-none inset-shadow-none inset-ring-0 hover:bg-none hover:bg-hover hover:shadow-none active:bg-none active:bg-pressed disabled:bg-none disabled:bg-transparent disabled:inset-ring-0",
    ),
    (
        "Button.spinner",
        "what a busy button shows before its label",
        "size-4 shrink-0 rounded-capsule border-2 border-current border-t-transparent animate-spin",
    ),
    (
        "Checkbox.indicator",
        "a checkbox's box; checked: and indeterminate: follow the checkbox",
        "flex items-center justify-center size-5 shrink-0 rounded-control bg-fill-strong inset-ring-2 inset-ring-border-strong text-accent-fg transition duration-120 checked:bg-accent checked:inset-ring-0 indeterminate:bg-accent indeterminate:inset-ring-0 disabled:opacity-50",
    ),
    (
        "Radio.indicator",
        "a radio's circle; checked: follows the radio",
        "flex items-center justify-center size-5 shrink-0 rounded-capsule bg-fill-strong inset-ring-2 inset-ring-border-strong transition duration-120 checked:inset-ring-accent disabled:opacity-50",
    ),
    (
        "Radio.dot",
        "the dot inside a checked radio",
        "size-2.5 rounded-capsule bg-accent scale-0 transition-transform duration-120 group-checked/radio:scale-100",
    ),
    (
        "Switch.track",
        "a switch's track; checked: follows the switch",
        "relative h-6 w-11 shrink-0 rounded-capsule bg-fill-strong inset-ring-1 inset-ring-border-strong transition duration-200 checked:bg-accent checked:inset-ring-0 disabled:opacity-50",
    ),
    (
        "Switch.thumb",
        "the thumb that moves along a switch's track",
        "absolute top-0.5 left-0.5 size-5 rounded-capsule bg-knob shadow-raise transition-transform duration-200 ease-out checked:translate-x-5 checked:bg-accent-fg",
    ),
    (
        "Tab.item",
        "a tab of a tab list",
        "flex-row items-center justify-center gap-2 min-h-11 px-3 text-sm font-semibold text-fg-muted transition duration-120 hover:text-fg selected:text-fg disabled:text-fg-subtle",
    ),
    (
        "Checkbox.mark",
        "the check inside a checked checkbox",
        "size-3.5 text-accent-fg",
    ),
    (
        "Checkbox.dash",
        "the dash inside an indeterminate checkbox",
        "w-2.5 h-0.5 rounded-capsule bg-accent-fg",
    ),
    (
        "Tab.list",
        "a tab list's row",
        "flex-row items-stretch border-b border-border-muted",
    ),
    (
        "Tab.mark",
        "the selected tab's mark: an underline (Tab.indicator is how it slides)",
        "inset-x-2 bottom-0 h-0.75 rounded-capsule bg-accent",
    ),
    (
        "Toggle.item",
        "a toggle button, alone or in a toggle group; aria-pressed: is on",
        "flex-row items-center justify-center gap-2 min-h-11 min-w-11 px-3 rounded-control text-sm font-semibold text-fg-muted transition duration-120 hover:bg-hover hover:text-fg active:bg-pressed aria-pressed:bg-layer-4 aria-pressed:text-fg aria-pressed:shadow-raise disabled:text-fg-subtle",
    ),
    (
        "ToggleGroup.frame",
        "a toggle group's box: a segmented control's well",
        "flex-row items-stretch gap-0.5 p-0.5 rounded-field bg-layer-2",
    ),
    (
        "Toolbar.frame",
        "a toolbar's bar",
        "flex-row items-center gap-1 p-1 rounded-field bg-layer-2",
    ),
    (
        "Toolbar.separator",
        "the line between runs of a toolbar's items",
        "self-stretch w-px my-1.5 bg-border",
    ),
    (
        "Slider.track",
        "a slider's track",
        "relative grow h-1.5 rounded-capsule bg-fill-strong",
    ),
    (
        "Slider.range",
        "the filled part of a slider's track, from its start to the thumb",
        "absolute left-0 top-0 h-full rounded-capsule bg-accent",
    ),
    (
        "Slider.thumb",
        "a slider's thumb",
        "absolute top-1/2 size-5 -translate-x-1/2 -translate-y-1/2 rounded-capsule bg-knob shadow-raise inset-ring-2 inset-ring-accent transition-transform duration-120 group-active/slider:scale-110 disabled:opacity-50",
    ),
    (
        "NumberInput.frame",
        "a number input's box, around its field and steppers",
        "flex-row items-center rounded-field bg-well inset-ring-1 inset-ring-border",
    ),
    (
        "NumberInput.field",
        "a number input's text",
        "grow min-w-0 px-2 text-center text-base font-semibold tabular-nums text-fg",
    ),
    (
        "NumberInput.step",
        "a number input's decrement and increment buttons",
        "flex items-center justify-center size-11 shrink-0 text-lg font-bold text-fg-muted hover:text-fg active:bg-pressed disabled:text-fg-subtle",
    ),
    (
        "Dialog.frame",
        "a dialog's panel, over Modal.scrim (Modal.frame is how it pops in)",
        "flex-col gap-3 w-115 max-w-full m-auto p-6 compact:p-4 rounded-panel bg-layer-1 text-fg shadow-overlay",
    ),
    (
        "Dialog.title",
        "a dialog's heading",
        "font-display text-2xl font-bold leading-tight text-fg",
    ),
    (
        "Dialog.body",
        "what a dialog says under its heading",
        "text-base leading-normal text-fg-muted",
    ),
    (
        "Dialog.actions",
        "the row of a dialog's buttons",
        "flex-row flex-wrap justify-end gap-2 pt-2",
    ),
    (
        "Sheet.frame",
        "a sheet's panel at the bottom of the screen",
        "flex-col gap-3 w-full max-h-[85%] mt-auto px-4 pt-2 pb-6 rounded-t-sheet bg-layer-1 text-fg shadow-overlay transition ease-out duration-240 starting:translate-y-12 starting:opacity-0 exit:translate-y-12 exit:opacity-0 exit:duration-180 exit:ease-exit",
    ),
    (
        "Sheet.grabber",
        "the handle at the top of a sheet",
        "self-center w-10 h-1.5 my-1 rounded-capsule bg-border-strong",
    ),
    (
        "Popover.frame",
        "a popover's panel by its trigger",
        "flex-col gap-2 min-w-48 max-w-80 p-3 rounded-card bg-layer-2 text-fg shadow-float transition duration-120 ease-out starting:opacity-0 starting:scale-95 exit:opacity-0",
    ),
    (
        "Tooltip.frame",
        "a tooltip's bubble",
        "max-w-64 px-2.5 py-1.5 rounded-control bg-fg text-fg-inverse text-xs font-semibold shadow-float",
    ),
    (
        "Toast.stack",
        "the column toasts stack in",
        "flex-col items-center gap-2 p-4 pt-[calc(env(safe-area-inset-top)+1rem)] pointer-events-none",
    ),
    (
        "Toast.frame",
        "a toast",
        "flex-row items-center gap-3 w-full max-w-96 min-h-12 px-4 py-2 rounded-card bg-layer-2 text-fg shadow-float pointer-events-auto transition ease-out duration-240 starting:opacity-0 starting:-translate-y-4 exit:opacity-0 exit:duration-120",
    ),
    (
        "Toast.frame.success",
        "a toast saying something went well",
        "inset-ring-1 inset-ring-success",
    ),
    (
        "Toast.frame.danger",
        "a toast saying something failed",
        "inset-ring-1 inset-ring-danger",
    ),
    (
        "Select.trigger",
        "a select's button, showing its value",
        "flex-row items-center justify-between gap-2 min-h-11 px-3 rounded-field bg-well inset-ring-1 inset-ring-border text-fg hover:inset-ring-border-strong aria-expanded:inset-ring-accent disabled:text-fg-subtle",
    ),
    (
        "Select.list",
        "a select's open list of options",
        "flex-col p-1 rounded-card bg-layer-2 shadow-float transition duration-120 ease-out starting:opacity-0 starting:-translate-y-1 exit:opacity-0",
    ),
    (
        "Select.option",
        "an option of a select's list",
        "flex-row items-center gap-2 min-h-11 px-3 rounded-control text-fg hover:bg-hover active:bg-pressed selected:bg-layer-4 selected:font-semibold disabled:text-fg-subtle",
    ),
    (
        "Collapsible.trigger",
        "what a collapsible's or accordion item's header presses",
        "flex-row items-center justify-between gap-2 min-h-11 px-3 text-fg font-semibold hover:bg-hover active:bg-pressed",
    ),
    (
        "Collapsible.chevron",
        "the chevron that turns as it opens",
        "size-4 text-fg-muted transition-transform duration-200 group-aria-expanded/collapsible:rotate-180",
    ),
    (
        "Collapsible.panel",
        "what a collapsible shows while open",
        "flex-col gap-2 px-3 pb-3 text-fg-muted transition ease-out duration-200 starting:opacity-0 starting:-translate-y-1 exit:opacity-0 exit:duration-120",
    ),
    (
        "Accordion.item",
        "a section of an accordion",
        "flex-col border-b border-border-muted",
    ),
    (
        "Progress.track",
        "a progress bar's track; its indicator is Bar.fill",
        "w-full h-2 overflow-hidden rounded-capsule bg-fill-strong shadow-sunk",
    ),
    (
        "Progress.indeterminate",
        "a progress bar's indicator while its extent is unknown",
        "w-1/3 animate-pulse",
    ),
    (
        "Panel.frame",
        "a window or panel's frame (the first layer)",
        "bg-layer-1",
    ),
    (
        "Plate.frame",
        "a plate over the scene or inside a panel (the layer above a panel's)",
        "bg-layer-2 text-fg",
    ),
    ("Slot.frame", "an ability or item slot", ""),
    ("Chip.frame", "a tag, badge or chip", ""),
    ("Field.frame", "a text field or input's box", ""),
    (
        "Row.frame",
        "a row of a list, table or feed",
        "bg-layer-3 text-fg",
    ),
    (
        "Row.frame.selected",
        "the chosen row, or the local player's",
        "bg-layer-4 text-fg inset-shadow-bar",
    ),
    (
        "Row.frame.muted",
        "a row out of play (disconnected, locked, spent)",
        "bg-layer-3 text-fg-muted",
    ),
    (
        "Head.frame",
        "a table or list's head row",
        "bg-surface text-fg-muted",
    ),
];

/// The shapes every look has (`templates/styles/shared/shapes/`, white
/// masks drawn 4x, coloured by the element's fill): a round slot's
/// cooldown `disc` and progress `ring` (stretched whole, for a radial
/// wipe), a square slot's `tile` at the slot radius (`--radius-field`),
/// and the HUD's bevels: `plate-cut` (`--hud-plate-cut`, 6) and
/// `panel-cut` (`--hud-panel-cut`, 12) cut the leading top and trailing
/// bottom corners, `chip-cut` the trailing top one; a d-pad's `cross`,
/// its outline `cross-rim` and its top arm `cross-arm` (each stretched
/// whole over the pad's box: one shape, so no scale opens a seam in it);
/// then the art.
fn shapes() -> Value {
    let mut out = Map::new();
    for (name, slice, width) in [
        ("disc", 0, None),
        ("ring", 0, None),
        ("cross", 0, None),
        ("cross-rim", 0, None),
        ("cross-arm", 0, None),
        ("tile", 24, Some(6)),
        ("plate-cut", 24, Some(6)),
        ("panel-cut", 48, Some(12)),
        ("chip-cut", 24, Some(6)),
    ] {
        let mut image = json!({ "src": format!("@styles/shared/shapes/{name}.svg"), "slice": slice, "tint": true });
        if let Some(w) = width {
            image["width"] = json!(w);
        }
        out.insert(name.to_owned(), image);
    }
    out.extend(art());
    Value::Object(out)
}

/// The art every look has (`templates/styles/shared/art/`, generated
/// pictures drawn 4x): an item or ability's octagon `socket` and its
/// `socket-rim` (a white mask the element's fill colours: a rarity rim),
/// a radial `glow` (a white mask the fill colours) and a coin or plate's
/// `sheen` stretched over the box, and a card's ornate `card-frame` (its
/// centre left open for the card's art).
fn art() -> Vec<(String, Value)> {
    [
        ("socket", json!({ "src": "@styles/shared/art/socket.png", "slice": 76, "width": 19 })),
        ("socket-rim", json!({ "src": "@styles/shared/art/socket-rim.png", "slice": 76, "width": 19, "fill": false, "tint": true })),
        ("glow", json!({ "src": "@styles/shared/art/glow.png", "slice": 0, "tint": true })),
        ("sheen", json!({ "src": "@styles/shared/art/sheen.png", "slice": 0 })),
        ("card-frame", json!({ "src": "@styles/shared/art/card-frame.png", "slice": 60, "width": 15, "fill": false })),
        ]
        .into_iter()
        .map(|(name, image)| (name.to_owned(), image))
        .chain(effects())
        .collect()
}

/// The UI effects' particle textures (`templates/styles/shared/fx/`): the
/// engine's emitters name them by id, and as images the fill colours them
/// (`nine-fx-sparkle bg-gold`), stretched whole.
fn effects() -> Vec<(String, Value)> {
    let mut out = Vec::new();
    for name in ["sparkle", "glint", "shard", "coin", "ray", "ring"] {
        out.push((
            format!("fx-{name}"),
            json!({ "src": format!("@styles/shared/fx/{name}.png"), "slice": 0, "tint": true }),
        ));
    }
    out
}

/// The input prompt glyphs every look has (`templates/styles/shared/glyphs/`,
/// single-colour masks the text colour tints), by og-input's glyph names
/// (`Glyph::icon`): every key og-input binds (`key.<code lowercased>`; a
/// left and right modifier, a numpad digit or operator and its main key
/// share one), the Mac's `key.command`, the mouse's, touch's, and each pad
/// family's buttons, d-pad and sticks.
fn icons() -> Value {
    let mut out = Map::new();
    let mut put = |name: String, file: &str| {
        out.insert(name, json!(format!("@styles/shared/glyphs/{file}.svg")));
    };
    for code in og_input::KEY_CODES {
        let lower = code.to_ascii_lowercase();
        let file = match *code {
            c if c.starts_with("Key") => format!("key-{}", &lower[3..]),
            c if c.starts_with("Digit") => format!("key-{}", &c[5..]),
            c if c.starts_with("Numpad") && c[6..].parse::<u8>().is_ok() => {
                format!("key-{}", &c[6..])
            }
            "ShiftLeft" | "ShiftRight" => "key-shift".into(),
            "ControlLeft" | "ControlRight" => "key-ctrl".into(),
            "AltLeft" | "AltRight" => "key-alt".into(),
            "MetaLeft" | "MetaRight" => "key-meta-win".into(),
            "NumpadSubtract" => "key-minus".into(),
            "NumpadDivide" => "key-slash".into(),
            "NumpadDecimal" => "key-period".into(),
            "NumpadEnter" => "key-enter".into(),
            "NumpadEqual" => "key-equal".into(),
            _ => format!("key-{lower}"),
        };
        put(format!("key.{lower}"), &file);
    }
    for (name, file) in [
        ("key.command", "key-meta-mac"),
        ("key.wasd", "key-wasd"),
        ("key.arrows", "key-arrows"),
        ("mouse.left", "mouse-left"),
        ("mouse.middle", "mouse-middle"),
        ("mouse.right", "mouse-right"),
        ("mouse.move", "mouse-move"),
        ("mouse.wheel", "mouse-wheel"),
        ("touch.tap", "touch-tap"),
        ("xbox.a", "face-a"),
        ("xbox.b", "face-b"),
        ("xbox.x", "face-x"),
        ("xbox.y", "face-y"),
        ("xbox.lb", "bumper-lb"),
        ("xbox.rb", "bumper-rb"),
        ("xbox.lt", "trigger-lt"),
        ("xbox.rt", "trigger-rt"),
        ("xbox.view", "view"),
        ("xbox.menu", "menu"),
        ("ps.cross", "ps-cross"),
        ("ps.circle", "ps-circle"),
        ("ps.square", "ps-square"),
        ("ps.triangle", "ps-triangle"),
        ("ps.l1", "bumper-l1"),
        ("ps.r1", "bumper-r1"),
        ("ps.l2", "trigger-l2"),
        ("ps.r2", "trigger-r2"),
        ("ps.create", "ps-create"),
        ("ps.options", "menu"),
        ("switch.a", "face-a"),
        ("switch.b", "face-b"),
        ("switch.x", "face-x"),
        ("switch.y", "face-y"),
        ("switch.l", "bumper-l"),
        ("switch.r", "bumper-r"),
        ("switch.zl", "trigger-zl"),
        ("switch.zr", "trigger-zr"),
        ("switch.minus", "minus"),
        ("switch.plus", "plus"),
        ("pad.south", "pad-south"),
        ("pad.east", "pad-east"),
        ("pad.west", "pad-west"),
        ("pad.north", "pad-north"),
        ("pad.l1", "bumper-l1"),
        ("pad.r1", "bumper-r1"),
        ("pad.l2", "trigger-l2"),
        ("pad.r2", "trigger-r2"),
        ("pad.select", "view"),
        ("pad.start", "menu"),
    ] {
        put(name.into(), file);
    }
    // Every family's d-pad, sticks and home button.
    for family in ["xbox", "ps", "switch", "pad"] {
        for (name, file) in [
            ("dpad", "dpad"),
            ("dpadUp", "dpad-up"),
            ("dpadDown", "dpad-down"),
            ("dpadLeft", "dpad-left"),
            ("dpadRight", "dpad-right"),
            ("stickL", "stick-l"),
            ("stickR", "stick-r"),
            ("ls", "stick-l-press"),
            ("rs", "stick-r-press"),
            ("home", "home"),
        ] {
            put(format!("{family}.{name}"), file);
        }
    }
    Value::Object(out)
}

/// What each scheme's theme takes in high contrast: the entries of the
/// `:root[data-contrast=more]` tokens' theme that differ from the plain one.
fn contrast(names: &[String]) -> [Value; 2] {
    let t = Tokens::parse(&FILES.concat());
    Scheme::ALL.map(|s| {
        let (plain, more) = (
            theme(&t.table(s.name(), false), names),
            theme(&t.table(s.name(), true), names),
        );
        let mut out = Map::new();
        for (section, v) in more {
            let (Value::Object(m), Some(Value::Object(p))) = (v, plain.get(&section)) else {
                continue;
            };
            let changed: Map<String, Value> =
                m.into_iter().filter(|(k, v)| p.get(k) != Some(v)).collect();
            if !changed.is_empty() {
                out.insert(section, Value::Object(changed));
            }
        }
        Value::Object(out)
    })
}

/// The motion slots' defaults, from motion.css's timing table, so a pack
/// restyles them by its tokens or its own `variants`.
struct Timing<'a>(&'a Tokens);

impl Timing<'_> {
    fn num(&self, name: &str) -> f64 {
        let v = self.0.get(name).unwrap_or_else(|| panic!("{name}"));
        v.trim().parse().unwrap_or_else(|_| panic!("{name}: {v}"))
    }

    /// A duration in ms: its own (`50ms`) or the `--motion-duration-*` it names.
    fn ms(&self, name: &str) -> f64 {
        let ms = |v: &str| {
            v.trim()
                .strip_suffix("ms")
                .and_then(|n| n.trim().parse::<f64>().ok())
        };
        let v = self.0.get(name).unwrap_or_else(|| panic!("{name}"));
        ms(v)
            .or_else(|| {
                self.0
                    .get(&format!("--motion-duration-{}", v.trim()))
                    .and_then(ms)
            })
            .unwrap_or_else(|| panic!("{name}: {v} names no duration"))
    }

    /// A scale as a `scale-*` utility.
    fn scale(&self, name: &str) -> String {
        let s = self.num(name);
        let pct = s * 100.0;
        if (pct - pct.round()).abs() < 1e-6 {
            format!("scale-{}", pct.round())
        } else {
            format!("scale-[{s}]")
        }
    }

    /// NSH's `ease-spring-[stiffness,damping]` of a unit mass for a damping
    /// ratio ζ and a response T in seconds: k = (2π/T)², c = 4πζ/T.
    fn spring(zeta: f64, response: f64) -> String {
        let tau = std::f64::consts::TAU;
        let (k, c) = ((tau / response).powi(2), 2.0 * tau * zeta / response);
        format!("ease-spring-[{},{}]", k.round(), (c * 10.0).round() / 10.0)
    }

    /// `Press.feedback`: the press scale (`--motion-scale-press`) under
    /// `active:`, in over the press row's `in` on ease-out, and back on its
    /// spring (its damping, its duration as the response).
    fn press(&self) -> String {
        let spring = Self::spring(
            self.num("--motion-timing-press-damping"),
            self.ms("--motion-timing-press-duration") / 1000.0,
        );
        format!(
            "transition-transform {spring} active:{} active:duration-{} active:ease-out",
            self.scale("--motion-scale-press"),
            self.ms("--motion-timing-press-in")
        )
    }

    /// `Screen.frame`: in from transparent at the enter scale on the enter
    /// row, out to transparent at the exit scale on the exit row.
    fn screen(&self) -> String {
        let ease = |n: &str| self.ease(n);
        format!(
            "transition duration-{} ease-{} starting:opacity-0 starting:{} exit:opacity-0 exit:{} exit:duration-{} exit:ease-{}",
            self.ms("--motion-timing-enter-duration"),
            ease("--motion-timing-enter-easing"),
            self.scale("--motion-timing-enter-scale"),
            self.scale("--motion-timing-exit-scale"),
            self.ms("--motion-timing-exit-duration"),
            ease("--motion-timing-exit-easing"),
        )
    }

    /// `Modal.frame`: a pop in from transparent at the pop scale, back on
    /// the pop row's spring (its damping, its duration as the response),
    /// out as a screen leaves.
    fn modal(&self) -> String {
        let spring = Self::spring(
            self.num("--motion-timing-pop-damping"),
            self.ms("--motion-timing-pop-duration") / 1000.0,
        );
        format!(
            "transition {spring} starting:opacity-0 starting:{} exit:opacity-0 exit:{} exit:duration-{} exit:ease-{}",
            self.scale("--motion-timing-pop-scale"),
            self.scale("--motion-timing-exit-scale"),
            self.ms("--motion-timing-exit-duration"),
            self.ease("--motion-timing-exit-easing"),
        )
    }

    /// `Modal.scrim`: the scrim a dialog dims the screen with, fading in on
    /// the enter row and out on the exit row, on its own while the dialog
    /// pops.
    fn scrim(&self) -> String {
        format!(
            "absolute inset-0 bg-scrim transition-opacity duration-{} ease-{} starting:opacity-0 exit:opacity-0 exit:duration-{} exit:ease-{}",
            self.ms("--motion-timing-enter-duration"),
            self.ease("--motion-timing-enter-easing"),
            self.ms("--motion-timing-exit-duration"),
            self.ease("--motion-timing-exit-easing"),
        )
    }

    /// `List.cascade`: the rows a list puts on screen together enter one
    /// after another, the cascade row's step apart, spread over its cap.
    fn cascade(&self) -> String {
        format!(
            "stagger-{}/{}",
            self.ms("--motion-timing-cascade-step"),
            self.ms("--motion-timing-cascade-cap")
        )
    }

    /// `Count.roll`: a number rolling to a new value on the count row.
    fn count(&self) -> String {
        format!(
            "transition-count duration-{} ease-{}",
            self.ms("--motion-timing-count-duration"),
            self.ease("--motion-timing-count-easing")
        )
    }

    /// `Tab.indicator`: a selection's indicator, sliding on the glide row's
    /// spring from the one it replaces (`glide-tab`).
    fn indicator(&self) -> String {
        let spring = Self::spring(
            self.num("--motion-timing-glide-damping"),
            self.ms("--motion-timing-glide-duration") / 1000.0,
        );
        format!("transition-layout {spring} glide-tab")
    }

    /// `Bar.fill`: a fill easing to a new value on the bar row.
    fn bar(&self) -> String {
        format!(
            "transition-layout duration-{} ease-{}",
            self.ms("--motion-timing-bar-duration"),
            self.ease("--motion-timing-bar-easing")
        )
    }

    /// `Bar.trail`: what was lost draining after the fill, on the trail row.
    fn trail(&self) -> String {
        format!(
            "transition-layout duration-{} delay-{} ease-{}",
            self.ms("--motion-timing-trail-duration"),
            self.ms("--motion-timing-trail-delay"),
            self.ease("--motion-timing-bar-easing")
        )
    }

    /// An easing row's name, as its `ease-*` utility takes it.
    fn ease(&self, name: &str) -> String {
        self.0
            .get(name)
            .unwrap_or_else(|| panic!("{name}"))
            .trim()
            .to_owned()
    }
}

/// The default pack's sounds: each event's clip of the shared set
/// (`@styles/shared/sfx/default/`, mastered by
/// templates/tools/master-ui-sounds.py so the events are balanced), with a
/// small pitch spread on the repeated ones.
fn sounds() -> Value {
    const SFX: &str = "@styles/shared/sfx/default";
    let sound = |clip: &str, volume: Value, pitch: Option<[f32; 2]>| {
        let mut s = json!({ "clips": [format!("{SFX}/{clip}.ogg")], "volume": volume });
        if let Some(p) = pitch {
            s["pitch"] = json!(p);
        }
        s
    };
    let jitter = Some([0.98, 1.02]);
    let hover = sound("tap", json!(0.12), Some([1.18, 1.26]));
    let mut out = Map::new();
    for (event, clip, volume, pitch) in [
        ("press", "tap", json!([0.45, 0.5]), Some([0.96, 1.04])),
        ("select", "tab", json!(0.5), jitter),
        ("confirm", "confirm", json!(0.5), None),
        ("cancel", "back", json!(0.5), None),
        ("back", "back", json!(0.5), None),
        ("open", "open", json!(0.45), None),
        ("close", "close", json!(0.45), None),
        ("error", "error", json!(0.5), None),
        ("reward", "reward", json!(0.5), None),
        ("toggle", "toggle", json!(0.5), jitter),
        ("tab", "tab", json!(0.5), jitter),
        ("levelup", "levelup", json!(0.5), None),
        ("purchase", "purchase", json!(0.5), None),
        ("countdown", "countdown", json!(0.5), None),
    ] {
        out.insert(event.into(), sound(clip, volume, pitch));
    }
    out.insert("hover".into(), hover.clone());
    out.insert("Button.plate".into(), json!({ "hover": hover }));
    out.insert(
        "Button.plate.primary".into(),
        json!({ "hover": hover, "press": sound("confirm", json!(0.45), jitter) }),
    );
    Value::Object(out)
}

/// `default/pack.json`: the design tokens as a pack.
pub fn default_pack_json() -> String {
    let names = names(&FILES);
    let [light, dark] = tables(&FILES, "").map(|t| Value::Object(theme(&t, &names)));
    let [more_light, more_dark] = contrast(&names);
    let pack = json!({
        "about": "The engine's default pack, generated from packages/design/tokens/*.css by crates/og-ui-next/scripts/gen-pack.sh; never edit it by hand. Theme names are the tokens' without their prefix: --color-fg-muted is the colour fg-muted, --radius-card the radius card, --font-scale-body-size the font size body.",
        "theme": join(&light, &dark),
        "contrast": join(&more_light, &more_dark),
        "variants": SLOTS.iter().map(|&(slot, _, dress)| {
            let motion = &tables(&[MOTION], "")[0];
            let dress = match slot {
                "Press.feedback" => Timing(motion).press(),
                "Screen.frame" => Timing(motion).screen(),
                "Modal.frame" => Timing(motion).modal(),
                "Modal.scrim" => Timing(motion).scrim(),
                "List.cascade" => Timing(motion).cascade(),
                "Count.roll" => Timing(motion).count(),
                "Tab.indicator" => Timing(motion).indicator(),
                "Bar.fill" => Timing(motion).bar(),
                "Bar.trail" => Timing(motion).trail(),
                _ => dress.to_owned(),
            };
            (slot.to_string(), json!(dress))
        }).collect::<Map<_, _>>(),
        "images": shapes(),
        "icons": icons(),
        "iconSet": "@styles/shared/icons/game.ogt",
        "sounds": sounds(),
    });
    let mut out = serde_json::to_string_pretty(&pack).unwrap();
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pack_json_is_the_generators_output() {
        assert!(
            default_pack_json() == DEFAULT_PACK,
            "crates/og-ui-next/default/pack.json is stale: run crates/og-ui-next/scripts/gen-pack.sh"
        );
    }

    /// One rem is the design's body step, 14 px, in the default pack's
    /// theme as the class resolver reads it: a bracketed rem and the step
    /// the design names for the same length agree (NSH's own rem is 16).
    #[test]
    fn a_rem_is_the_body_type_step() {
        use crate::classes::{Classes, Env};
        let pack = crate::pack::Pack::new(&[crate::pack::Layer::default_pack()]);
        let (mut c, _) = Classes::new(Some(&pack.theme_json(Scheme::Dark))).unwrap();
        let env = Env::default();
        let mut width = |class: &str| {
            taffy::CoreStyle::size(&c.record(class, &env).layout)
                .width
                .value()
        };
        // --size-6xl is 240 px: 17.143 rem of 14, and 60 steps of 4.
        assert!((width("w-[17.143rem]") - 240.0).abs() < 0.01);
        assert_eq!(width("w-60"), 240.0);
        // A style prop's rem is the same one.
        let styled = c.record_styled("", "width: 17.143rem", &env);
        let w = taffy::CoreStyle::size(&styled.layout).width.value();
        assert!((w - 240.0).abs() < 0.01, "{w}");
        // --font-scale-xs-size is 12 px: 0.857 rem of 14.
        let (mut c, _) = Classes::new(Some(&pack.theme_json(Scheme::Dark))).unwrap();
        let bracket = c.record("text-[0.857rem]", &env).font_size;
        assert!((bracket - 12.0).abs() < 0.01, "{bracket}");
        assert_eq!(c.record("text-xs", &env).font_size, 12.0);
    }

    /// Every glyph og-input names has the default pack's art: each key it
    /// binds, the Mac's Command, every pad family's buttons, sticks and
    /// d-pad, the mouse's and touch's; and each icon is a pinned file
    /// (templates/og-assets.json), so none draws as a missing texture.
    #[test]
    fn every_binding_has_glyph_art() {
        use og_input::{Control, Device, GAMEPAD_BUTTONS, KEY_CODES, key_glyph, pad_glyph};
        let pack: Value = serde_json::from_str(DEFAULT_PACK).unwrap();
        let icons = pack["icons"].as_object().unwrap();
        let mut names: Vec<String> = KEY_CODES.iter().map(|c| key_glyph(c).icon).collect();
        names.push("key.command".into());
        for device in [
            Device::Xbox,
            Device::PlayStation,
            Device::Switch,
            Device::Gamepad,
        ] {
            names.extend(GAMEPAD_BUTTONS.iter().map(|b| pad_glyph(device, b).icon));
            for control in ["pad.leftXY", "pad.rightXY", "pad.dpad", "pad.leftTrigger"] {
                names.push(Control::parse(control).unwrap().glyph(device).icon);
            }
        }
        for control in [
            "mouse.left",
            "mouse.middle",
            "mouse.right",
            "mouse.xy",
            "wheel.y",
            "touch",
        ] {
            names.push(
                Control::parse(control)
                    .unwrap()
                    .glyph(Device::KeyboardMouse)
                    .icon,
            );
        }
        let missing: Vec<&String> = names.iter().filter(|n| !icons.contains_key(*n)).collect();
        assert!(missing.is_empty(), "glyphs without art: {missing:?}");
        let pins = include_str!("../../../../templates/og-assets.json");
        let unpinned: Vec<&str> = icons
            .values()
            .filter_map(Value::as_str)
            .map(|src| src.trim_start_matches('@'))
            .filter(|src| !pins.contains(&format!("\"path\": \"{src}\"")))
            .collect();
        assert!(unpinned.is_empty(), "icons not pinned: {unpinned:?}");
        // Every event the default pack sounds is a pinned clip too.
        let layer = crate::pack::Layer::default_pack();
        let clips: Vec<&str> = layer
            .assets()
            .into_iter()
            .filter(|src| src.ends_with(".ogg"))
            .collect();
        assert!(clips.len() >= 12, "{clips:?}");
        let unpinned: Vec<&&str> = clips
            .iter()
            .filter(|src| !pins.contains(&format!("\"path\": \"{}\"", src.trim_start_matches('@'))))
            .collect();
        assert!(unpinned.is_empty(), "sounds not pinned: {unpinned:?}");
    }

    /// WCAG contrast of `ink` on `ground`, a translucent ground over `bg`.
    fn contrast(c: &Map<String, Value>, ink: &str, ground: &str) -> f32 {
        let get = |n: &str| og_css::parse_color(c[n].as_str().unwrap()).unwrap();
        let ground = get(ground).over(get("bg"));
        get(ink).over(ground).contrast(ground)
    }

    /// The design's contrast pairs (contrast.css `--contrast`, the two-name
    /// ones) hold in both schemes, for the default seeds and for a pack's.
    /// The run time reads only the palette's and the motion's files: a
    /// colour or duration that came to depend on another file's token
    /// would resolve differently there than in the generated pack.
    #[test]
    fn the_run_times_files_resolve_as_the_whole_design_does() {
        let seeds = ":root { --seed-primary: #2f8f5b; } :root[data-scheme=dark] { --seed-primary: #6fd39a; }";
        for seeds in ["", seeds] {
            let all = tables(&FILES, seeds).map(|t| {
                let names = names(&FILES);
                names
                    .iter()
                    .filter(|n| n.starts_with("--color-"))
                    .map(|n| (n.clone(), t.get(n).and_then(hex)))
                    .collect::<Vec<_>>()
            });
            let palette = colors(seeds);
            for (scheme, all) in all.iter().enumerate() {
                assert!(all.len() > 50, "the design names its colours");
                for (n, c) in all {
                    let got = palette[scheme].get(&n["--color-".len()..]);
                    assert_eq!(got.and_then(|v| v.as_str()), c.as_deref(), "{n}");
                }
            }
        }
        let [t, _] = tables(&FILES, "");
        let d = durations();
        assert!(d.len() >= 10, "{d:?}");
        for (name, ms) in &d {
            let v = t.get(&format!("--motion-duration-{name}")).unwrap();
            assert_eq!(v.trim(), format!("{ms}ms"), "{name}");
        }
    }

    #[test]
    fn derived_palettes_keep_the_designs_contrast_pairs() {
        let css = include_str!("../../../../packages/design/tokens/contrast.css");
        let list = css
            .split("--contrast:")
            .nth(1)
            .unwrap()
            .split(';')
            .next()
            .unwrap();
        let pairs: Vec<(&str, &str)> = list
            .split(',')
            .filter_map(|p| match p.split_whitespace().collect::<Vec<_>>()[..] {
                [a, b] => Some((a, b)),
                _ => None,
            })
            .collect();
        assert!(pairs.len() > 40, "{}", pairs.len());
        let warm = ":root { --seed-primary: #ffc71f; --seed-neutral: #232a73; --seed-danger: #ff7272; --seed-team: #45b3ff; }";
        for seeds in ["", warm] {
            for (i, c) in colors(seeds).iter().enumerate() {
                let low: Vec<String> = pairs
                    .iter()
                    .map(|&(a, b)| (a, b, contrast(c, a, b)))
                    .filter(|&(.., r)| r < 4.5)
                    .map(|(a, b, r)| format!("{a} on {b} {r:.2}"))
                    .collect();
                assert!(
                    low.is_empty(),
                    "seeds {seeds:?}, {:?}: {low:?}",
                    Scheme::ALL[i]
                );
            }
        }
    }
}
