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
        } else if let Some(name) = after("--motion-duration-") {
            // `duration-base`, `delay-fast`: milliseconds.
            if let Some(ms) = v
                .strip_suffix("ms")
                .and_then(|n| n.trim().parse::<f64>().ok())
            {
                put("duration", name, json!(ms));
            }
        } else if let Some(name) = after("--motion-scale-") {
            if let Ok(f) = v.parse::<f64>() {
                put("scale", name, round(f));
            }
        } else if let Some(name) = after("--opacity-") {
            if let Ok(f) = v.parse::<f64>() {
                put("opacity", name, round(f));
            }
        } else if let Some(name) = after("--stroke-") {
            // `ring-regular`, `border-thick`: a line width of the scale.
            if let Some(w) = px(v) {
                put("stroke", name, json!(w));
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

mod assets;
mod slots;
use assets::{Timing, icons, shapes, sounds};
pub use slots::SLOTS;

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
}
