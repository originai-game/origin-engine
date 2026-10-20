//! The look cook step: a `type look` .ogt (grammar: [`crate::look`]) is
//! resolved, checked and flattened here, so its blob is the whole look.
//!
//! - `extends "<slug>"` names the game's own `assets/looks/<slug>.ogt`
//!   when it has one (a look of that path never extends itself: there the
//!   slug is the built-in preset), else the built-in preset `<slug>`
//!   ([`PRESETS`], the engine's `templates/looks/<slug>.ogt`); `extends
//!   "looks/base.ogt"` (a path) names another look under `assets/`. Bases
//!   chain up to [`MAX_EXTENDS`] deep; a cycle is refused. The look is laid
//!   over its base field by field ([`LookFile::over`]).
//! - Every field is checked against the IDL's records (`og_api::host::RECORDS`,
//!   the reflection every tool reads): the resource and field exist and are
//!   writable, a number is in its range, a variant is one of the enum's, a
//!   colour is three linear numbers or an sRGB `#rrggbb` (written linear), a
//!   handle field names an asset of its kind that the bundle has.
//! - The stamp covers every look in the chain, so editing a base re-cooks
//!   the looks built on it.

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use crate::look::{LookFile, MAX_EXTENDS, hex_to_linear};
use og_blob::BlobWriter;
use og_core::ogt::{Number, Value};
use std::path::Path;

/// The built-in presets `extends` names by slug: the engine's
/// `templates/looks/<slug>.ogt`, compiled into the cook. Phase B of the
/// stylisation round lands them with their reference frames; until then a
/// slug resolves only to the game's own `assets/looks/<slug>.ogt`.
pub const PRESETS: &[(&str, &str)] = &[];

/// A look in an `extends` chain: its id (an asset id, or a preset's slug)
/// and its text.
struct Base {
    id: String,
    text: String,
}

fn find_base(
    name: &str,
    from: &str,
    root: &Path,
    presets: &[(&str, &str)],
) -> Result<Base, String> {
    let read = |id: &str| std::fs::read_to_string(root.join(id)).ok();
    if name.contains('/') || name.ends_with(".ogt") {
        let id = crate::runtime::asset_id(name).to_string();
        return read(&id)
            .map(|text| Base {
                id: id.clone(),
                text,
            })
            .ok_or_else(|| format!("extends \"{name}\": there is no assets/{id}"));
    }
    let own = format!("looks/{name}.ogt");
    if own != from
        && let Some(text) = read(&own)
    {
        return Ok(Base { id: own, text });
    }
    if let Some((slug, text)) = presets.iter().find(|(s, _)| *s == name) {
        return Ok(Base {
            id: slug.to_string(),
            text: text.to_string(),
        });
    }
    let names: Vec<&str> = presets.iter().map(|(s, _)| *s).collect();
    Err(format!(
        "extends \"{name}\": neither assets/{own} nor a built-in preset ({})",
        if names.is_empty() {
            "the engine ships none yet".to_string()
        } else {
            names.join(", ")
        }
    ))
}

/// The look `text` (of id `id`) laid over its chain of bases, and the
/// bases it read, nearest first.
fn flatten(
    id: &str,
    text: &str,
    root: &Path,
    presets: &[(&str, &str)],
    seen: &mut Vec<String>,
    bases: &mut Vec<Base>,
) -> Result<LookFile, String> {
    let mut look = LookFile::parse(text).map_err(|e| {
        if seen.len() > 1 {
            format!("{id}: {e}")
        } else {
            e
        }
    })?;
    let Some(name) = look.extends.first().cloned() else {
        return Ok(look);
    };
    if look.extends.len() > 1 {
        return Err(format!(
            "{id}: extends names one look; the chain is built by each look naming its own"
        ));
    }
    if seen.len() > MAX_EXTENDS {
        return Err(format!("extends chains at most {MAX_EXTENDS} looks deep"));
    }
    let base = find_base(&name, id, root, presets)?;
    if seen.contains(&base.id) {
        seen.push(base.id.clone());
        return Err(format!(
            "extends goes round in a circle: {}",
            seen.join(" → ")
        ));
    }
    seen.push(base.id.clone());
    let base_look = flatten(&base.id, &base.text, root, presets, seen, bases)?;
    look.extends.clear();
    let merged = look.over(&base_look, &base.id);
    bases.insert(0, base);
    Ok(merged)
}

/// What a handle column takes: the asset kinds by extension.
fn handle_extensions(column: &str) -> Option<&'static [&'static str]> {
    Some(match column {
        "Texture" => &["png", "jpg", "jpeg"],
        "Lut" => &["cube"],
        "Palette" => &["hex", "gpl"],
        "EnvironmentMap" => &["ogt"],
        _ => return None,
    })
}

fn word(v: &[Value]) -> Option<&str> {
    match v {
        [Value::Word(w)] => Some(w),
        _ => None,
    }
}

fn number(n: f64) -> Value {
    Value::Number(Number::from_f64(n).expect("a finite number"))
}

/// Checks the flattened look against the records and writes colours
/// linear; `known` answers whether the bundle has an asset id.
fn check(look: &mut LookFile, known: &dyn Fn(&str) -> bool) -> Result<(), String> {
    let mut missing: Vec<String> = Vec::new();
    if let Some(skin) = &look.skin {
        let id = crate::runtime::asset_id(skin);
        if !id.ends_with(".ogt") {
            return Err(format!(
                "skin \"{skin}\": a UI skin's .ogt (skins/<name>/skin.ogt)"
            ));
        }
        if !known(id) {
            missing.push(id.to_string());
        }
    }
    for block in &mut look.blocks {
        let schema = format!("og.{}", block.resource);
        let record = og_api::host::RECORDS
            .iter()
            .find(|r| r.schema == schema && r.kind == "resource")
            .ok_or_else(|| format!("{}: not a resource of this engine", block.resource))?;
        for (field, values, line) in &mut block.fields {
            let r = &block.resource;
            let at = format!("line {line}: {r}.{field}");
            let decl = record
                .fields
                .iter()
                .find(|f| f.name == field)
                .ok_or_else(|| {
                    let names: Vec<&str> = record
                        .fields
                        .iter()
                        .filter(|f| !crate::look::ANIMATED_FIELDS.contains(&(r.as_str(), f.name)))
                        .map(|f| f.name)
                        .collect();
                    format!(
                        "{at}: {r} has no field '{field}' (its fields: {})",
                        names.join(", ")
                    )
                })?;
            if decl.readonly {
                return Err(format!("{at} is read only"));
            }
            let range = |n: f64| -> Result<(), String> {
                if decl.min.is_some_and(|m| n < m) || decl.max.is_some_and(|m| n > m) {
                    return Err(format!(
                        "{at}: {n} is outside {} to {}",
                        decl.min.map_or("-∞".to_string(), |m| m.to_string()),
                        decl.max.map_or("∞".to_string(), |m| m.to_string())
                    ));
                }
                Ok(())
            };
            let numbers = |n: usize| -> Result<Vec<f64>, String> {
                let out: Vec<f64> = values
                    .iter()
                    .filter_map(|v| v.as_number().map(Number::as_f64))
                    .filter(|x| x.is_finite())
                    .collect();
                if out.len() != n || values.len() != n {
                    return Err(format!(
                        "{at} takes {n} number{}",
                        if n == 1 { "" } else { "s" }
                    ));
                }
                Ok(out)
            };
            if !decl.variants.is_empty() {
                match word(values) {
                    Some(w) if decl.variants.contains(&w) => {}
                    _ => {
                        return Err(format!("{at} is one of {}", decl.variants.join(", ")));
                    }
                }
                continue;
            }
            if let Some(exts) = handle_extensions(decl.column) {
                let path = match values.as_slice() {
                    [Value::String(s)] => crate::runtime::asset_id(s).to_string(),
                    _ => {
                        return Err(format!(
                            "{at} names a {} by its asset id, as a string",
                            decl.column
                        ));
                    }
                };
                let ext = path
                    .rsplit_once('.')
                    .map_or("", |(_, e)| e)
                    .to_ascii_lowercase();
                if !exts.contains(&ext.as_str()) {
                    return Err(format!(
                        "{at}: \"{path}\" is not a {} (.{})",
                        decl.column,
                        exts.join(", .")
                    ));
                }
                if !known(&path) {
                    missing.push(path.clone());
                }
                *values = vec![Value::String(path)];
                continue;
            }
            match decl.column {
                "bool" => {
                    if !matches!(word(values), Some("true" | "false")) {
                        return Err(format!("{at} is true or false"));
                    }
                }
                "f32" => range(numbers(1)?[0])?,
                "u32" | "i32" => {
                    let n = numbers(1)?[0];
                    if n.fract() != 0.0 || (decl.column == "u32" && n < 0.0) {
                        return Err(format!(
                            "{at} is a whole number{}",
                            if decl.column == "u32" { " from 0" } else { "" }
                        ));
                    }
                    range(n)?;
                }
                "Vector2" => {
                    numbers(2)?;
                }
                "Vector3" => {
                    numbers(3)?;
                }
                "Color" => {
                    let rgb = match values.as_slice() {
                        [Value::String(s)] => hex_to_linear(s).map(Vec::from).ok_or_else(|| {
                            format!("{at}: \"{s}\" is not an sRGB #rrggbb colour")
                        })?,
                        _ => numbers(3).map_err(|_| {
                            format!("{at} is three linear numbers or an sRGB \"#rrggbb\"")
                        })?,
                    };
                    *values = rgb.into_iter().map(number).collect();
                }
                other => return Err(format!("{at} ({other}) is not set from a look")),
            }
        }
    }
    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return Err(format!(
            "the look names asset(s) not under assets/: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

/// The flattened, checked look of `s` and the bases it read.
fn resolve(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
    presets: &[(&str, &str)],
) -> Result<(LookFile, Vec<Base>), String> {
    let text = std::str::from_utf8(&s.bytes).map_err(|_| "a look is text (UTF-8)".to_string())?;
    let mut seen = vec![s.rel.clone()];
    let mut bases = Vec::new();
    let mut look = flatten(&s.rel, text, root, presets, &mut seen, &mut bases)?;
    check(&mut look, known)?;
    Ok((look, bases))
}

/// The stamp's dependencies: every base's id and text hash, and whether the
/// assets the look names are in the bundle.
pub(super) fn look_dependencies(s: &Source, root: &Path, known: &dyn Fn(&str) -> bool) -> String {
    let text = String::from_utf8_lossy(&s.bytes);
    let mut seen = vec![s.rel.clone()];
    let mut bases = Vec::new();
    let mut out = Vec::new();
    match flatten(&s.rel, &text, root, PRESETS, &mut seen, &mut bases) {
        Ok(look) => {
            let mut refs: Vec<String> = look.skin.iter().cloned().collect();
            for b in &look.blocks {
                for (_, v, _) in &b.fields {
                    if let [Value::String(p)] = v.as_slice() {
                        refs.push(crate::runtime::asset_id(p).to_string());
                    }
                }
            }
            for r in refs {
                out.push(format!("{r}={}", known(crate::runtime::asset_id(&r))));
            }
        }
        Err(e) => out.push(format!("error {e}")),
    }
    for b in bases {
        out.push(format!(
            "{}={:016x}",
            b.id,
            super::hash_bytes(b.text.as_bytes())
        ));
    }
    out.join("\n")
}

/// Cooks one `type look` file.
pub(super) fn cook_look(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    cook_with(s, root, known, PRESETS, stamp)
}

fn cook_with(
    s: &Source,
    root: &Path,
    known: &dyn Fn(&str) -> bool,
    presets: &[(&str, &str)],
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let (look, _) = resolve(s, root, known, presets).map_err(CookError)?;
    let text = look.to_text();
    let mut w = BlobWriter::new(
        format::Look::type_hash(),
        format::Look::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::LookMeta {
            text_bytes: text.len() as u32,
        }],
        format::row_hash::<format::LookMeta>(),
    )
    .raw("text", text.into_bytes(), 16);
    let fields: usize = look.blocks.iter().map(|b| b.fields.len()).sum();
    Ok((
        w.finish(),
        vec![format!(
            "look '{}': {} ({fields} field(s)){}",
            look.name
                .as_deref()
                .unwrap_or(s.rel.trim_end_matches(".ogt")),
            look.resources().join(", "),
            if look.extends.is_empty() {
                String::new()
            } else {
                format!(", over {}", look.extends.join(" → "))
            }
        )],
    ))
}

/// Cooks one `.hex` or `.gpl` palette.
pub(super) fn cook_palette(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError("a palette is text (UTF-8)".into()))?;
    let colours = crate::look::parse_palette(&s.rel, text).map_err(CookError)?;
    let mut w = BlobWriter::new(
        format::Palette::type_hash(),
        format::Palette::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::PaletteMeta {
            count: colours.len() as u32,
        }],
        format::row_hash::<format::PaletteMeta>(),
    )
    .raw(
        "rgb",
        colours.iter().flatten().copied().collect::<Vec<u8>>(),
        16,
    );
    Ok((
        w.finish(),
        vec![format!("{} colour palette", colours.len())],
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn source(root: &Path, rel: &str) -> Source {
        Source {
            rel: rel.to_string(),
            path: root.join(rel),
            kind: super::super::Kind::Look,
            bytes: std::fs::read(root.join(rel)).unwrap(),
            settings: None,
            hash: 0,
            gltf: None,
            error: None,
        }
    }

    fn project(files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "og-look-{}-{}",
            std::process::id(),
            super::super::hash_bytes(format!("{files:?}").as_bytes())
        ));
        let _ = std::fs::remove_dir_all(&dir);
        for (rel, text) in files {
            let p = dir.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, text).unwrap();
        }
        dir
    }

    const HEAD: &str = "type look\nversion 1\n---\n";
    const PRESET: &str = "type look\nversion 1\n---\nname \"Cel\"\nantialias msaa4\nMaterialStyle {\n  toon true\n  outlineWidth 2\n}\nPostEffects {\n  vignette true\n}\n";

    fn cook(root: &Path, rel: &str, known: &[&str]) -> Result<LookFile, String> {
        let s = source(root, rel);
        let known = |id: &str| known.contains(&id);
        resolve(&s, root, &known, &[("cel", PRESET)]).map(|(l, _)| l)
    }

    #[test]
    fn extends_a_preset_and_the_games_own_look_wins() {
        let root = project(&[(
            "looks/main.ogt",
            &format!(
                "{HEAD}extends \"cel\"\nStylise {{\n  posterize 5\n}}\nMaterialStyle {{\n  outlineWidth 3\n}}\n"
            ),
        )]);
        let l = cook(&root, "looks/main.ogt", &[]).unwrap();
        assert_eq!(l.extends, vec!["cel"]);
        assert_eq!(l.name, None);
        assert_eq!(l.antialias.as_deref(), Some("msaa4"));
        let m = l
            .blocks
            .iter()
            .find(|b| b.resource == "MaterialStyle")
            .unwrap();
        assert_eq!(
            m.get("outlineWidth").unwrap()[0]
                .as_number()
                .unwrap()
                .text(),
            "3"
        );
        assert_eq!(m.get("toon").unwrap()[0].as_word(), Some("true"));
        assert_eq!(
            l.resources(),
            vec!["PostEffects", "Stylise", "MaterialStyle"]
        );

        // The game's own looks/cel.ogt overrides the preset of that slug,
        // and itself extends the preset.
        let root = project(&[
            (
                "looks/cel.ogt",
                &format!("{HEAD}extends \"cel\"\nname \"Our cel\"\n"),
            ),
            ("looks/main.ogt", &format!("{HEAD}extends \"cel\"\n")),
        ]);
        let l = cook(&root, "looks/main.ogt", &[]).unwrap();
        assert_eq!(l.extends, vec!["looks/cel.ogt", "cel"]);
        assert_eq!(l.name.as_deref(), Some("Our cel"));
    }

    #[test]
    fn checks_fields_against_the_records() {
        let one = |body: &str, known: &[&str]| {
            let root = project(&[("looks/a.ogt", &format!("{HEAD}{body}"))]);
            cook(&root, "looks/a.ogt", known)
        };
        let l = one("PostEffects {\n  outlineColor \"#ffffff\"\n  aoMethod gtao\n}\nStylise {\n  palette \"palettes/p.hex\"\n  dither bayer8\n}\n", &["palettes/p.hex"]).unwrap();
        let p = l
            .blocks
            .iter()
            .find(|b| b.resource == "PostEffects")
            .unwrap();
        assert_eq!(p.get("outlineColor").unwrap().len(), 3);
        assert!(
            one("Stylise {\n  posterise 4\n}\n", &[])
                .unwrap_err()
                .contains("Stylise has no field 'posterise'")
        );
        assert!(
            one("Stylise {\n  posterize 100\n}\n", &[])
                .unwrap_err()
                .contains("outside 0 to 64")
        );
        assert!(
            one("Stylise {\n  posterize 2.5\n}\n", &[])
                .unwrap_err()
                .contains("whole number")
        );
        assert!(
            one("Stylise {\n  dither bayer3\n}\n", &[])
                .unwrap_err()
                .contains("one of none, bayer2")
        );
        assert!(
            one("Stylise {\n  palette \"palettes/p.png\"\n}\n", &[])
                .unwrap_err()
                .contains("not a Palette")
        );
        assert!(
            one("Stylise {\n  palette \"palettes/p.hex\"\n}\n", &[])
                .unwrap_err()
                .contains("not under assets/: palettes/p.hex")
        );
        assert!(
            one("PostEffects {\n  outline 1\n}\n", &[])
                .unwrap_err()
                .contains("true or false")
        );
        assert!(
            one("Fog {\n  color \"#12\"\n}\n", &[])
                .unwrap_err()
                .contains("not an sRGB")
        );
        assert!(
            one("skin \"skins/x/skin.ogt\"\n", &[])
                .unwrap_err()
                .contains("skins/x/skin.ogt")
        );
    }

    #[test]
    fn refuses_a_missing_base_and_a_cycle() {
        let root = project(&[
            ("looks/a.ogt", &format!("{HEAD}extends \"looks/b.ogt\"\n")),
            ("looks/b.ogt", &format!("{HEAD}extends \"a\"\n")),
            ("looks/c.ogt", &format!("{HEAD}extends \"nowhere\"\n")),
        ]);
        assert!(
            cook(&root, "looks/a.ogt", &[])
                .unwrap_err()
                .contains("circle")
        );
        let e = cook(&root, "looks/c.ogt", &[]).unwrap_err();
        assert!(
            e.contains("neither assets/looks/nowhere.ogt nor a built-in preset (cel)"),
            "{e}"
        );
    }

    #[test]
    fn the_presets_are_the_template_looks() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/looks");
        let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
            .map(|d| {
                d.filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().to_str().map(str::to_string))
                    .filter_map(|n| n.strip_suffix(".ogt").map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        on_disk.sort();
        let mut built_in: Vec<String> = PRESETS.iter().map(|(s, _)| s.to_string()).collect();
        built_in.sort();
        assert_eq!(built_in, on_disk, "PRESETS lists templates/looks/*.ogt");
        for (slug, text) in PRESETS {
            LookFile::parse(text).unwrap_or_else(|e| panic!("{slug}: {e}"));
        }
    }
}
