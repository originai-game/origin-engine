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

/// The token files the theme reads, in the order the design applies them.
const FILES: [&str; 6] = [
    include_str!("../../../../packages/design/tokens/seed.css"),
    include_str!("../../../../packages/design/tokens/color.css"),
    include_str!("../../../../packages/design/tokens/type.css"),
    include_str!("../../../../packages/design/tokens/space.css"),
    include_str!("../../../../packages/design/tokens/effect.css"),
    include_str!("../../../../packages/design/tokens/motion.css"),
];

/// The generated default pack.
pub const DEFAULT_PACK: &str = include_str!("../../default/pack.json");

/// The token table in each scheme, with `seeds` (CSS setting `--seed-*`)
/// over the design's own.
fn tables(seeds: &str) -> [Tokens; 2] {
    let mut t = Tokens::parse(&FILES.concat());
    if !seeds.is_empty() {
        t.over(seeds);
    }
    Scheme::ALL.map(|s| t.scheme(s.name()))
}

/// Every token's name, in the files' order.
fn names() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for file in FILES {
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
    let names = names();
    tables(seeds).map(|t| {
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
        } else if let Some(name) = after("--shadow-") {
            put("shadow", name, Value::String(v.to_owned()));
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
pub const SLOTS: [(&str, &str, &str); 10] = [
    ("Button.plate", "a button's body", ""),
    ("Panel.frame", "a window or panel's frame", ""),
    (
        "Plate.frame",
        "a HUD plate over the scene",
        "bg-glass text-fg",
    ),
    ("Slot.frame", "an ability or item slot", ""),
    ("Chip.frame", "a tag, badge or chip", ""),
    ("Field.frame", "a text field or input's box", ""),
    (
        "Row.frame",
        "a row of a list, table or feed",
        "bg-raised-lo text-fg",
    ),
    (
        "Row.frame.selected",
        "the chosen row, or the local player's",
        "bg-surface-hover text-fg border-l-3 border-hud-accent",
    ),
    (
        "Row.frame.muted",
        "a row out of play (disconnected, locked, spent)",
        "bg-raised-lo text-fg-muted",
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
/// bottom corners, `chip-cut` the trailing top one; then the art.
fn shapes() -> Value {
    let mut out = Map::new();
    for (name, slice, width) in [
        ("disc", 0, None),
        ("ring", 0, None),
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
    .collect()
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

/// `default/pack.json`: the design tokens as a pack.
pub fn default_pack_json() -> String {
    let names = names();
    let [light, dark] = tables("").map(|t| Value::Object(theme(&t, &names)));
    let [more_light, more_dark] = contrast(&names);
    let pack = json!({
        "about": "The engine's default pack, generated from packages/design/tokens/*.css by crates/og-ui-next/scripts/gen-pack.sh; never edit it by hand. Theme names are the tokens' without their prefix: --color-fg-muted is the colour fg-muted, --radius-card the radius card, --font-scale-body-size the font size body.",
        "theme": join(&light, &dark),
        "contrast": join(&more_light, &more_dark),
        "variants": SLOTS.iter().map(|(slot, _, dress)| (slot.to_string(), json!(dress))).collect::<Map<_, _>>(),
        "images": shapes(),
        "icons": icons(),
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
    }

    /// WCAG contrast of `ink` on `ground`, a translucent ground over `bg`.
    fn contrast(c: &Map<String, Value>, ink: &str, ground: &str) -> f32 {
        let get = |n: &str| og_css::parse_color(c[n].as_str().unwrap()).unwrap();
        let ground = get(ground).over(get("bg"));
        get(ink).over(ground).contrast(ground)
    }

    /// The design's contrast pairs (contrast.css `--contrast`, the two-name
    /// ones) hold in both schemes, for the default seeds and for a pack's.
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
        let cartoon = ":root { --seed-primary: #ffc71f; --seed-neutral: #232a73; --seed-danger: #ff7272; --seed-team: #45b3ff; }";
        for seeds in ["", cartoon] {
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
