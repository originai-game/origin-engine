//! The effect files' grammar as data: each parser's line table
//! ([`crate::fx::EMITTER_LINES`], [`crate::effect::EFFECT_LINES`] and
//! [`crate::effect::UV_LINES`], [`crate::vfx::VFX_LINES`] and
//! [`crate::vfx::ELEMENT_LINES`]). A parser refuses a key its table does
//! not list, `og api gen` writes the tables into the `effect-files` guide
//! ([`PATH`], held equal by `og api check`), and the tests parse every
//! row's example and check that writing a row's default changes nothing,
//! so the guide states exactly the lines the parsers take.

use std::fmt::Write;

pub use og_core::ogt::line::Line;

/// Validate a file block against its published field vocabulary.
pub fn check(block: &og_core::ogt::Block, fields: &[Line], at: &str) -> Result<(), String> {
    for e in &block.entries {
        if !fields.iter().any(|f| f.key == e.key) {
            return Err(format!("line {}: {at} has no '{}'", e.line, e.key));
        }
    }
    Ok(())
}

/// The row of `lines` for `key` taken by an element labelled `labels`
/// (any row for a body line, whose labels are empty).
pub fn find<'a>(lines: &'a [Line], key: &str, labels: &[&str]) -> Option<&'a Line> {
    lines.iter().find(|l| {
        l.key == key && (l.within.is_empty() || l.within.iter().any(|w| labels.contains(w)))
    })
}

/// The keys of `lines` an element labelled `labels` takes, comma
/// separated and each once, for a refusal.
pub fn keys(lines: &[Line], labels: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::new();
    for l in lines {
        if (l.within.is_empty() || l.within.iter().any(|w| labels.contains(w)))
            && !out.contains(&l.key)
        {
            out.push(l.key);
        }
    }
    out.join(", ")
}

/// Where the guide lives, from the repository root.
pub const PATH: &str = "crates/og-api/guides/effect-files.md";

fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}

fn table(out: &mut String, lines: &[Line], within: bool) {
    if within {
        out.push_str("| Line | In | Default | Does |\n|---|---|---|---|\n");
    } else {
        out.push_str("| Line | Default | Does |\n|---|---|---|\n");
    }
    for l in lines {
        let syntax = if l.args.is_empty() {
            format!("`{}`", l.key)
        } else {
            format!("`{} {}`", l.key, cell(l.args))
        };
        let default = if l.default.is_empty() {
            String::new()
        } else {
            format!("`{}`", cell(l.default))
        };
        if within {
            let what = if l.within.is_empty() {
                "every".to_string()
            } else {
                l.within.join(", ")
            };
            let _ = writeln!(out, "| {syntax} | {what} | {default} | {} |", cell(l.doc));
        } else {
            let _ = writeln!(out, "| {syntax} | {default} | {} |", cell(l.doc));
        }
    }
}

/// The `effect-files` guide's text.
pub fn markdown() -> String {
    let mut out = String::from(
        "<!-- Generated from the effect parsers' line tables (crates/og-bundle/src/fx.rs, effect.rs and vfx.rs) by `og api gen`. Do not edit: `og api check` fails on any difference. -->\n\
# The effect files\n\n\
Every line the three effect formats take, from the tables their parsers read: an emitter (`type particles`), an effect material (`type effect`) and a composed effect (`type vfx`). A parser refuses any other line and names the ones it takes. Each file starts `type <name>`, `version <n>` and `---`; a line left out takes its default. What the formats are for and how a game loads them is `loading §5` and `loading §11`; ready-made effects are `effect-packs`, pixel effects (`type vfx2d`) `pixel-vfx`.\n",
    );
    let _ = writeln!(
        out,
        "\n## 1. Emitters (type particles)\n\n\
`type particles`, `version 1`: one particle emitter, loaded with og.load.emitter or named by a vfx `emitter` element. `<min> [<max>]` ranges are uniform random per particle; one number is both ends. A curve is 1 to {} `<t> <value>` pairs over the particle's life (t 0 to 1). Every line runs the same in compute and on the CPU path.\n",
        og_render_data::MAX_CURVE_KEYS
    );
    table(&mut out, crate::fx::EMITTER_LINES, false);
    out.push_str(
        "\n## 2. Effect materials (type effect)\n\n\
`type effect`, `version 1`: how a billboard, a mesh, a trail or a beam is coloured (`effect \"<id>\"` on an emitter or a vfx mesh, trail or beam). A value is a number, `random <min> <max>` (fixed per particle, strip or mesh) or `curve <t> <v> ...` (over the life: a particle's age, a strip's length from its start, a mesh's life; at most 8 keys).\n\n",
    );
    table(&mut out, crate::effect::EFFECT_LINES, false);
    out.push_str(
        "\nA texture line (`main`, `mask`, `noise`, `flowMap`) takes a block of how it is read:\n\n",
    );
    table(&mut out, crate::effect::UV_LINES, false);
    let _ = writeln!(
        out,
        "\n## 3. Composed effects (type vfx)\n\n\
`type vfx`, `version {}` (version 1 still reads, its `flash` as `glow`): one effect as a timeline of elements, spawned with og.vfx.spawn. Times are seconds from the effect's start; the tick they land on is the start plus the time times the tick rate, rounded. A value is as an effect material's, over the element's life. The header lines mean the same in a `type vfx2d` pixel effect.\n",
        crate::vfx::VFX_VERSION
    );
    table(&mut out, crate::vfx::VFX_LINES, false);
    out.push_str(
        "\nAn element's block takes these lines; In names the elements that take one (every: all of them; ring: a `mesh ring`):\n\n",
    );
    table(&mut out, crate::vfx::ELEMENT_LINES, true);
    out.push_str(
        "\n## 4. Cook-generated sources\n\nEach of `particles`, `effect` and `vfx` can contain one exclusive `generate` block instead of body declarations. The existing restricted TS executor returns `EffectSource {text}`: only the body, without type/version/header. The target format's ordinary parser validates the result at the file's declared version; imports, input bytes and parameters are tracked as in `generating`. Recursive generation is refused. See `file-types` for the generate block's fields.\n\n`type effect-texture` cooks density/heat volumes or shared procgen noise/Worley/curl into a scattering sheet at its id, a thermal emission sheet at `#emission`, and chronological arrays at `#array` and `#emission#array`. All parameters, units and limits are in `og api show file:effect-texture`. `effects §5` explains sampling and has a checked recipe composing all three generated effect types with a generated volume.\n",
    );
    out
}

/// A `blend` line's word: additive, alpha, subtractive or multiply.
pub(crate) fn blend_word(w: Option<&str>) -> Option<og_render_data::ParticleBlend> {
    use og_render_data::ParticleBlend::*;
    Some(match w? {
        "additive" => Additive,
        "alpha" => Alpha,
        "subtractive" => Subtractive,
        "multiply" => Multiply,
        _ => return None,
    })
}

/// [`blend_word`]'s words, for a line's syntax and its errors.
pub(crate) const BLENDS: &str = "additive | alpha | subtractive | multiply";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::{EFFECT_LINES, EffectLinks, UV_LINES, parse_effect};
    use crate::fx::{EMITTER_LINES, EmitterLinks, parse_emitter_linked};
    use crate::vfx::{ELEMENT_LINES, VFX_LINES, parse_vfx};

    fn emitter(body: &str) -> Result<og_render_data::ParticleEmitter, String> {
        parse_emitter_linked(
            &format!("type particles\nversion 1\n---\n{body}\n"),
            &mut EmitterLinks {
                texture: &mut |_| Ok(None),
                array: &mut |_| Ok(None),
                mesh: &mut |_| Ok(None),
                emitter: &mut |_| Ok(None),
                effect: &mut |_| Ok(None),
            },
        )
    }

    fn effect(body: &str) -> Result<og_render_data::EffectMaterial, String> {
        parse_effect(
            &format!("type effect\nversion 1\n---\n{body}\n"),
            &mut EffectLinks {
                texture: &mut |_| Ok(None),
                shader: &mut |_| Ok(None),
            },
        )
    }

    fn vfx(body: &str) -> Result<crate::vfx::Vfx, String> {
        parse_vfx(&format!("type vfx\nversion 2\n---\n{body}\n"))
    }

    /// A vfx body holding `line` inside an element labelled `label`, with
    /// a param `power`.
    fn element(label: &str, line: &str) -> String {
        let key = line.split_whitespace().next().unwrap_or("");
        let life = if matches!(key, "life" | "until") {
            ""
        } else {
            "  life 1\n"
        };
        let head = match label {
            "emitter" => "emitter \"fx/a.ogt\" {\n".to_string(),
            "mesh" => format!("mesh sphere {{\n  effect \"fx/e.ogt\"\n{life}"),
            "ring" => format!("mesh ring {{\n  effect \"fx/e.ogt\"\n{life}"),
            "shake" => "shake {\n".to_string(),
            "sound" => "sound \"sfx/a.wav\" {\n".to_string(),
            "flipbook" => "flipbook \"fx/f.ogt\" {\n".to_string(),
            kind => format!("{kind} {{\n{life}"),
        };
        // A param for `when` and `amount` to name.
        format!("duration 2\nparam power 1\n{head}  {line}\n}}")
    }

    fn labels(l: &Line) -> Vec<&'static str> {
        if l.within.is_empty() {
            vec!["light"]
        } else {
            l.within.to_vec()
        }
    }

    /// Every row's example parses, so the guide shows no line the parser
    /// refuses: a row whose key the parser does not read fails here.
    #[test]
    fn every_rows_example_parses() {
        for l in EMITTER_LINES {
            emitter(l.example).unwrap_or_else(|e| panic!("emitter {}: {e}", l.key));
        }
        for l in EFFECT_LINES.iter().chain(UV_LINES) {
            effect(l.example).unwrap_or_else(|e| panic!("effect {}: {e}", l.key));
        }
        for l in VFX_LINES {
            let body = if l.key == "duration" {
                l.example.to_string()
            } else {
                format!("duration 2\n{}", l.example)
            };
            vfx(&body).unwrap_or_else(|e| panic!("vfx {}: {e}", l.key));
        }
        for l in ELEMENT_LINES {
            for label in labels(l) {
                vfx(&element(label, l.example))
                    .unwrap_or_else(|e| panic!("vfx {label} {}: {e}", l.key));
            }
        }
    }
}
