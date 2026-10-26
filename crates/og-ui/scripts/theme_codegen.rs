//! The theme generator: the platform's design tokens (`packages/tokens/tokens.json`)
//! to `crates/og-ui/src/theme.rs`. Shared by the `gen-theme` example (which
//! writes the file) and `tests/theme.rs` (which fails when the committed file
//! is not exactly what this produces). Every value comes from the token file;
//! nothing here is a design value.

use serde_json::{Map, Value};
use std::fmt::Write;

/// FNV-1a 64 of the token file's bytes (`og_core::hash::hash_bytes`).
fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// A token key as a Rust field: `bg-subtle` → `bg_subtle`, `2xl` → `xl2`,
/// `display-2xl` → `display_xl2`.
fn ident(key: &str) -> String {
    key.split('-')
        .map(|part| {
            let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
            if digits.is_empty() {
                part.to_owned()
            } else {
                format!("{}{digits}", &part[digits.len()..])
            }
        })
        .collect::<Vec<_>>()
        .join("_")
}

fn object<'a>(v: &'a Value, path: &str) -> Result<&'a Map<String, Value>, String> {
    let mut at = v;
    for seg in path.split('.') {
        at = at
            .get(seg)
            .ok_or_else(|| format!("tokens.json has no `{path}`"))?;
    }
    at.as_object()
        .ok_or_else(|| format!("tokens.json `{path}` is not an object"))
}

fn entries(map: &Map<String, Value>) -> impl Iterator<Item = (&String, &Value)> {
    map.iter().filter(|(k, _)| !k.starts_with('_'))
}

/// `#rrggbb` or `rgb(r g b / a)` → (rgb, alpha).
fn color(text: &str) -> Result<([u8; 3], f64), String> {
    let t = text.trim();
    if let Some(hex) = t.strip_prefix('#') {
        if hex.len() != 6 {
            return Err(format!("colour `{t}`: write #rrggbb"));
        }
        let byte = |i: usize| {
            u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| format!("colour `{t}`: {e}"))
        };
        return Ok(([byte(0)?, byte(2)?, byte(4)?], 1.0));
    }
    if let Some(body) = t.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
        let (channels, alpha) = match body.split_once('/') {
            Some((c, a)) => (
                c,
                a.trim()
                    .parse::<f64>()
                    .map_err(|e| format!("colour `{t}`: {e}"))?,
            ),
            None => (body, 1.0),
        };
        let c: Vec<u8> = channels
            .split_whitespace()
            .map(|v| v.parse::<u8>().map_err(|e| format!("colour `{t}`: {e}")))
            .collect::<Result<_, _>>()?;
        if c.len() != 3 {
            return Err(format!("colour `{t}`: three channels expected"));
        }
        return Ok(([c[0], c[1], c[2]], alpha));
    }
    Err(format!(
        "colour `{t}`: only #rrggbb and rgb(r g b / a) are understood"
    ))
}

/// `12px`, `-0.01em`, `120ms`, `0.96`, `1` → the number.
fn number(v: &Value, what: &str) -> Result<f64, String> {
    if let Some(n) = v.as_f64() {
        return Ok(n);
    }
    let s = v
        .as_str()
        .ok_or_else(|| format!("{what}: a number or a string expected"))?;
    let trimmed = ["px", "em", "ms", "%"]
        .iter()
        .find_map(|u| s.strip_suffix(u))
        .unwrap_or(s);
    trimmed
        .trim()
        .parse::<f64>()
        .map_err(|e| format!("{what} `{s}`: {e}"))
}

fn lit(n: f64) -> String {
    let s = format!("{n}");
    if s.contains('.') || s.contains('e') {
        s
    } else {
        format!("{s}.0")
    }
}

fn srgba(c: ([u8; 3], f64)) -> String {
    let ([r, g, b], a) = c;
    format!(
        "Srgba {{ rgb: [0x{r:02x}, 0x{g:02x}, 0x{b:02x}], alpha: {} }}",
        lit(a)
    )
}

fn doc_of(map: &Map<String, Value>) -> Option<String> {
    map.get("_doc").and_then(Value::as_str).map(str::to_owned)
}

fn doc_lines(out: &mut String, indent: &str, text: &str) {
    for line in wrap(text, 76) {
        let _ = writeln!(out, "{indent}/// {line}");
    }
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// A const of an `f32` struct: its name, its doc and its fields.
type F32Const<'a> = (&'a str, &'a str, Vec<(String, f64)>);

/// A struct of `f32` fields and one const of it.
fn f32_struct(out: &mut String, name: &str, doc: &str, consts: &[F32Const<'_>]) {
    doc_lines(out, "", doc);
    let _ = writeln!(out, "#[derive(Clone, Copy, Debug, PartialEq)]");
    let _ = writeln!(out, "pub struct {name} {{");
    for (field, _) in &consts[0].2 {
        let _ = writeln!(out, "    pub {field}: f32,");
    }
    let _ = writeln!(out, "}}\n");
    for (cname, cdoc, fields) in consts {
        doc_lines(out, "", cdoc);
        let _ = writeln!(out, "pub const {cname}: {name} = {name} {{");
        for (field, v) in fields {
            let _ = writeln!(out, "    {field}: {},", lit(*v));
        }
        let _ = writeln!(out, "}};\n");
    }
}

/// The whole `theme.rs` for a token file.
pub fn generate(source: &str) -> Result<String, String> {
    let root: Value =
        serde_json::from_str(source).map_err(|e| format!("tokens.json does not parse: {e}"))?;
    let mut out = String::new();
    let _ = writeln!(
        out,
        "//! GENERATED from the platform design tokens (`packages/tokens/tokens.json`,\n\
         //! vendored verbatim as `crates/og-ui/tokens/tokens.json`) by\n\
         //! `crates/og-ui/scripts/gen-theme.sh`. Do not edit: change the tokens in the\n\
         //! platform, run the script, commit both files. `tests/theme.rs` fails when\n\
         //! this file differs from what the generator writes.\n\
         //!\n\
         //! Lengths are CSS pixels, durations milliseconds, colours sRGB as authored\n\
         //! (straight alpha); `crate::style` converts them for the renderer.\n"
    );
    let _ = writeln!(
        out,
        "/// FNV-1a 64 of the token file this was generated from.\npub const SOURCE_FNV1A64: u64 = 0x{:016x};\n",
        fnv1a64(source.as_bytes())
    );
    let _ = writeln!(
        out,
        "/// An sRGB colour as the tokens author it: 8-bit channels, straight alpha.\n\
         #[derive(Clone, Copy, Debug, PartialEq)]\n\
         pub struct Srgba {{\n    pub rgb: [u8; 3],\n    pub alpha: f32,\n}}\n"
    );

    // Colours: light, then dark (light overlaid with dark.color).
    let light = object(&root, "color")?;
    let dark = object(&root, "dark.color")?;
    let _ = writeln!(
        out,
        "/// The semantic colours (`color` in tokens.json; `docs/design-language.md` §Colour)."
    );
    let _ = writeln!(
        out,
        "#[derive(Clone, Copy, Debug, PartialEq)]\npub struct Palette {{"
    );
    for (k, _) in entries(light) {
        let _ = writeln!(out, "    /// `{k}`");
        let _ = writeln!(out, "    pub {}: Srgba,", ident(k));
    }
    let _ = writeln!(out, "}}\n");
    for (name, doc, over) in [
        ("LIGHT", "The light palette (`color`).", None),
        (
            "DARK",
            "The dark palette (`color` overlaid with `dark.color`).",
            Some(dark),
        ),
    ] {
        let _ = writeln!(out, "/// {doc}\npub const {name}: Palette = Palette {{");
        for (k, v) in entries(light) {
            let v = over.and_then(|d| d.get(k)).unwrap_or(v);
            let text = v
                .as_str()
                .ok_or_else(|| format!("color `{k}` is not a string"))?;
            let _ = writeln!(out, "    {}: {},", ident(k), srgba(color(text)?));
        }
        let _ = writeln!(out, "}};\n");
    }

    // Hues.
    let hue = object(&root, "hue")?;
    let dark_hue = object(&root, "dark.hue")?;
    let _ = writeln!(
        out,
        "/// The twelve category hues (`hue`): categories, levels, badges, series. Never body text."
    );
    let _ = writeln!(
        out,
        "#[derive(Clone, Copy, Debug, PartialEq)]\npub struct Hues {{"
    );
    for (k, _) in entries(hue) {
        let _ = writeln!(out, "    pub {}: Srgba,", ident(k));
    }
    let _ = writeln!(out, "}}\n");
    for (name, over) in [("HUES_LIGHT", None), ("HUES_DARK", Some(dark_hue))] {
        let _ = writeln!(out, "pub const {name}: Hues = Hues {{");
        for (k, v) in entries(hue) {
            let v = over.and_then(|d| d.get(k)).unwrap_or(v);
            let text = v
                .as_str()
                .ok_or_else(|| format!("hue `{k}` is not a string"))?;
            let _ = writeln!(out, "    {}: {},", ident(k), srgba(color(text)?));
        }
        let _ = writeln!(out, "}};\n");
    }

    // Spacing and radii.
    let fields = |map: &Map<String, Value>, what: &str| -> Result<Vec<(String, f64)>, String> {
        entries(map)
            .map(|(k, v)| Ok((ident(k), number(v, &format!("{what}.{k}"))?)))
            .collect()
    };
    f32_struct(
        &mut out,
        "Space",
        "The 4px spacing scale (`space`), CSS pixels.",
        &[("SPACE", "", fields(object(&root, "space")?, "space")?)],
    );
    f32_struct(
        &mut out,
        "Radius",
        "The concentric radius scale (`radius`), CSS pixels; `button` is the capsule.",
        &[("RADIUS", "", fields(object(&root, "radius")?, "radius")?)],
    );

    // The type ramp.
    let scale = object(&root, "font.scale")?;
    let cjk = object(&root, "font.cjk.lineHeight")?;
    let _ = writeln!(
        out,
        "/// One step of the type ramp (`font.scale`): size and leading in CSS pixels,\n\
         /// Latin tracking in em, and the leading Chinese text takes (`font.cjk`)."
    );
    let _ = writeln!(
        out,
        "#[derive(Clone, Copy, Debug, PartialEq)]\npub struct TypeStep {{\n    pub size: f32,\n    pub line_height: f32,\n    pub tracking_em: f32,\n    pub cjk_line_height: f32,\n    /// Tracking of text with CJK characters, in em (CSS `letter-spacing`, which spaces Han too); 0 in the tokens.\n    pub cjk_tracking_em: f32,\n}}\n"
    );
    let _ = writeln!(out, "/// The type ramp (`font.scale`).");
    let _ = writeln!(
        out,
        "#[derive(Clone, Copy, Debug, PartialEq)]\npub struct TypeRamp {{"
    );
    for (k, _) in entries(scale) {
        let _ = writeln!(out, "    /// `text-{k}`\n    pub {}: TypeStep,", ident(k));
    }
    let _ = writeln!(out, "}}\n\npub const TYPE_RAMP: TypeRamp = TypeRamp {{");
    for (k, v) in entries(scale) {
        let size = number(
            v.get("size")
                .ok_or_else(|| format!("font.scale.{k}: no size"))?,
            &format!("font.scale.{k}.size"),
        )?;
        let lh = number(
            v.get("lineHeight")
                .ok_or_else(|| format!("font.scale.{k}: no lineHeight"))?,
            &format!("font.scale.{k}.lineHeight"),
        )?;
        let tracking = match v.get("letterSpacing") {
            Some(t) => number(t, &format!("font.scale.{k}.letterSpacing"))?,
            None => 0.0,
        };
        let cjk_lh = match cjk.get(k) {
            Some(t) => number(t, &format!("font.cjk.lineHeight.{k}"))?,
            None => lh,
        };
        let _ = writeln!(
            out,
            "    {}: TypeStep {{ size: {}, line_height: {}, tracking_em: {}, cjk_line_height: {}, cjk_tracking_em: 0.0 }},",
            ident(k),
            lit(size),
            lit(lh),
            lit(tracking),
            lit(cjk_lh)
        );
    }
    let _ = writeln!(out, "}};\n");

    // Density.
    let density = object(&root, "density")?;
    let compact = density_value(density, "compact")?;
    let mut consts = Vec::new();
    for name in ["compact", "comfortable", "touch"] {
        let map = density_value(density, name)?;
        let mut f = Vec::new();
        for (k, base) in entries(compact) {
            let v = map.get(k).unwrap_or(base);
            f.push((ident(k), number(v, &format!("density.{name}.{k}"))?));
        }
        consts.push((name.to_uppercase(), f));
    }
    let docs = [
        "Compact density (`density.compact`), the default for tools.",
        "Comfortable density (`density.comfortable`), for browsing surfaces.",
        "Touch density (`density.touch`): coarse pointers and widths under 768px. Keys the tokens do not raise (`gutter`, `section`) keep the compact value.",
    ];
    let consts_ref: Vec<F32Const<'_>> = consts
        .iter()
        .zip(docs)
        .map(|((n, f), d)| (n.as_str(), d, f.clone()))
        .collect();
    f32_struct(
        &mut out,
        "Density",
        "Control, row and header heights and the gutter (`density`), CSS pixels.",
        &consts_ref,
    );

    // Motion.
    f32_struct(
        &mut out,
        "Durations",
        "Motion durations (`motion.duration`), milliseconds.",
        &[(
            "DURATIONS_MS",
            "",
            fields(object(&root, "motion.duration")?, "motion.duration")?,
        )],
    );
    let springs = object(&root, "motion.spring")?;
    let _ = writeln!(
        out,
        "/// A spring (`motion.spring`): damping ratio and response in milliseconds."
    );
    if let Some(d) = doc_of(springs) {
        let _ = writeln!(out, "///");
        doc_lines(&mut out, "", &d);
    }
    let _ = writeln!(
        out,
        "#[derive(Clone, Copy, Debug, PartialEq)]\npub struct Spring {{\n    pub damping: f32,\n    pub response_ms: f32,\n}}\n"
    );
    for (k, v) in entries(springs) {
        let damping = number(
            v.get("damping")
                .ok_or_else(|| format!("motion.spring.{k}: no damping"))?,
            "damping",
        )?;
        let response = number(
            v.get("response")
                .ok_or_else(|| format!("motion.spring.{k}: no response"))?,
            "response",
        )?;
        let _ = writeln!(
            out,
            "pub const SPRING_{}: Spring = Spring {{ damping: {}, response_ms: {} }};",
            ident(k).to_uppercase(),
            lit(damping),
            lit(response)
        );
    }
    let _ = writeln!(out);
    let easing = object(&root, "motion.easing")?;
    for (k, v) in entries(easing) {
        let s = v.as_str().unwrap_or_default();
        if let Some(body) = s
            .strip_prefix("cubic-bezier(")
            .and_then(|b| b.strip_suffix(')'))
        {
            let p: Vec<f64> = body
                .split(',')
                .map(|x| {
                    x.trim()
                        .parse::<f64>()
                        .map_err(|e| format!("motion.easing.{k}: {e}"))
                })
                .collect::<Result<_, _>>()?;
            if p.len() != 4 {
                return Err(format!("motion.easing.{k}: four control values expected"));
            }
            let _ = writeln!(
                out,
                "/// `motion.easing.{k}`: cubic Bézier control points (x1, y1, x2, y2).\npub const EASE_{}: [f32; 4] = [{}, {}, {}, {}];",
                ident(k).to_uppercase(),
                lit(p[0]),
                lit(p[1]),
                lit(p[2]),
                lit(p[3])
            );
        }
    }
    // The card edge (`shadow.edge`; in dark the edge light): its inset
    // layers, each `inset <dx> <dy> <blur> [<spread>] <colour>`.
    let _ = writeln!(
        out,
        "\n/// One inset layer of a `shadow-*` token, drawn inside a surface's edge:\n\
         /// the surface less itself moved by (`dx`, `dy`) and shrunk by `spread`,\n\
         /// in `color` (CSS px; the tokens' edge layers have no blur).\n\
         #[derive(Clone, Copy, Debug, PartialEq)]\n\
         pub struct InsetShadow {{\n    pub dx: f32,\n    pub dy: f32,\n    pub spread: f32,\n    pub color: Srgba,\n}}"
    );
    for (scheme, path) in [("LIGHT", "shadow.edge"), ("DARK", "dark.shadow.edge")] {
        let (parent, key) = path.rsplit_once('.').expect("a dotted path");
        let text = object(&root, parent)?
            .get(key)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("tokens.json has no string `{path}`"))?;
        let mut layers = Vec::new();
        for layer in text.split("),").map(|l| l.trim()) {
            let layer = if layer.ends_with(')') {
                layer.to_owned()
            } else {
                format!("{layer})")
            };
            let rest = layer
                .strip_prefix("inset ")
                .ok_or_else(|| format!("`{path}`: `{layer}` is not an inset layer"))?;
            let at = rest
                .find("rgb(")
                .ok_or_else(|| format!("`{path}`: `{layer}` has no rgb() colour"))?;
            let lengths: Vec<f64> = rest[..at]
                .split_whitespace()
                .map(|v| {
                    v.trim_end_matches("px")
                        .parse::<f64>()
                        .map_err(|e| format!("`{path}`: `{v}`: {e}"))
                })
                .collect::<Result<_, _>>()?;
            if !(3..=4).contains(&lengths.len()) || lengths[2] != 0.0 {
                return Err(format!(
                    "`{path}`: `{layer}`: offsets, no blur and a spread expected"
                ));
            }
            layers.push(format!(
                "    InsetShadow {{ dx: {}, dy: {}, spread: {}, color: {} }},",
                lit(lengths[0]),
                lit(lengths[1]),
                lit(lengths.get(3).copied().unwrap_or(0.0)),
                srgba(color(&rest[at..])?)
            ));
        }
        let _ = writeln!(
            out,
            "\n/// `{path}`: the card edge's inset layers.\npub const SHADOW_EDGE_{scheme}: &[InsetShadow] = &[\n{}\n];",
            layers.join("\n")
        );
    }
    let vars = object(&root, "vars")?;
    let press = vars
        .get("press-scale")
        .ok_or("tokens.json has no `vars.press-scale`")?;
    let _ = writeln!(
        out,
        "\n/// `vars.press-scale`: how far a pressed control shrinks.\npub const PRESS_SCALE: f32 = {};",
        lit(number(press, "vars.press-scale")?)
    );
    Ok(out)
}

fn density_value<'a>(
    density: &'a Map<String, Value>,
    name: &str,
) -> Result<&'a Map<String, Value>, String> {
    density
        .get(name)
        .and_then(Value::as_object)
        .ok_or_else(|| format!("tokens.json has no object `density.{name}`"))
}
