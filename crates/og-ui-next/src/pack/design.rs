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
            let Some(name) = n.strip_prefix("--color-") else { continue };
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
    let inner = text.trim().strip_prefix("cubic-bezier(")?.strip_suffix(')')?;
    let n: Vec<f64> = inner.split(',').map(|x| x.trim().parse().ok()).collect::<Option<_>>()?;
    (n.len() == 4).then(|| json!(n))
}

/// The theme in one scheme's table.
fn theme(t: &Tokens, names: &[String]) -> Map<String, Value> {
    let mut sections: Map<String, Value> = Map::new();
    let mut put = |section: &str, name: &str, v: Value| {
        let s = sections.entry(section).or_insert_with(|| Value::Object(Map::new()));
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
            let line = t.get(&format!("--font-scale-{step}-line-height")).and_then(px);
            if let (Some(size), Some(line)) = (px(v), line) {
                put("fontSize", step, json!([size, round(line / size)]));
            }
        } else if let Some(step) = after("--font-scale-").and_then(|s| s.strip_suffix("-letter-spacing")) {
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
        } else if let Some(name) = ["sans", "display", "mono", "rounded"].iter().find(|f| n == &format!("--font-{f}")) {
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
fn join(light: &Value, dark: &Value) -> Value {
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

/// `default/pack.json`: the design tokens as a pack.
pub fn default_pack_json() -> String {
    let names = names();
    let [light, dark] = tables("").map(|t| Value::Object(theme(&t, &names)));
    let pack = json!({
        "about": "The engine's default pack, generated from packages/design/tokens/*.css by crates/og-ui-next/scripts/gen-pack.sh; never edit it by hand. Theme names are the tokens' without their prefix: --color-fg-muted is the colour fg-muted, --radius-card the radius card, --font-scale-body-size the font size body.",
        "theme": join(&light, &dark),
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
        let list = css.split("--contrast:").nth(1).unwrap().split(';').next().unwrap();
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
                assert!(low.is_empty(), "seeds {seeds:?}, {:?}: {low:?}", Scheme::ALL[i]);
            }
        }
    }
}
