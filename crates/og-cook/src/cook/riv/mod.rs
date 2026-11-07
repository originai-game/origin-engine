//! Rive import (`docs/ui.md` §11.2): a `.riv` converted at cook time into
//! og-ui data, one component per artboard. The Rive runtime is not
//! embedded; the format is read from its documentation and rive-runtime's
//! generated tables (MIT, `third_party/rive/`, `scripts/gen-rive-defs.py`).

pub(crate) mod convert;
mod defs;
pub(crate) mod read;

/// The `.riv` major format the import reads.
pub const MAJOR: u64 = 7;

/// A property's field type in the binary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Field {
    Uint,
    Int,
    Float,
    Bool,
    Color,
    String,
    Bytes,
}

#[cfg(test)]
mod tests;

use super::{CookError, Source};
use crate::format::{self, *};
use og_blob::BlobWriter;
use og_core::glam::Vec2;
use og_core::ogt::{Block, Entry, Value};
use og_ui::rive::Fit;

/// One artboard's import settings (`artboard <name> { … }`).
#[derive(Clone, Debug, Default, PartialEq)]
struct ArtboardSettings {
    machine: Option<String>,
    role: Option<String>,
    name: Option<String>,
    fit: Option<Fit>,
    align: Option<Vec2>,
}

/// A `.riv`'s import settings (`<file>.riv.ogt`, `type import`).
#[derive(Clone, Debug, Default, PartialEq)]
struct Settings {
    /// The artboards to cook (`None`: all).
    artboards: Option<Vec<String>>,
    per: Vec<(String, ArtboardSettings)>,
}

fn word_or_string(e: &Entry, i: usize) -> Result<String, String> {
    match e.values.get(i) {
        Some(Value::Word(w)) => Ok(w.clone()),
        Some(Value::String(s)) => Ok(s.clone()),
        _ => Err(format!(
            "line {}: '{}' takes a name at position {}",
            e.line,
            e.key,
            i + 1
        )),
    }
}

fn settings(body: &Block) -> Result<Settings, String> {
    let mut out = Settings::default();
    for e in &body.entries {
        match e.key.as_str() {
            "artboards" => {
                out.artboards = Some(
                    (0..e.values.len())
                        .map(|i| word_or_string(e, i))
                        .collect::<Result<_, _>>()?,
                )
            }
            "artboard" => {
                let name = word_or_string(e, 0)?;
                let mut a = ArtboardSettings::default();
                for f in e.block.iter().flat_map(|b| &b.entries) {
                    match f.key.as_str() {
                        "state-machine" => a.machine = Some(word_or_string(f, 0)?),
                        "role" => {
                            let r = word_or_string(f, 0)?;
                            if !og_ui::ROLES.contains(&r.as_str()) {
                                return Err(format!(
                                    "line {}: role '{r}' is not an og-ui role ({})",
                                    f.line,
                                    og_ui::ROLES.join(", ")
                                ));
                            }
                            a.role = Some(r);
                        }
                        "name" => a.name = Some(word_or_string(f, 0)?),
                        "fit" => {
                            let w = word_or_string(f, 0)?;
                            a.fit = Some(
                                Fit::ALL
                                    .iter()
                                    .find(|x| x.1 == w)
                                    .ok_or_else(|| {
                                        format!(
                                            "line {}: fit '{w}' (contain, cover, fill, none, scale-down, fit-width or fit-height)",
                                            f.line
                                        )
                                    })?
                                    .0,
                            );
                        }
                        "align" => {
                            let w = word_or_string(f, 0)?;
                            let (x, y) = match w.as_str() {
                                "center" => (0.5, 0.5),
                                "top-left" => (0.0, 0.0),
                                "top" => (0.5, 0.0),
                                "top-right" => (1.0, 0.0),
                                "left" => (0.0, 0.5),
                                "right" => (1.0, 0.5),
                                "bottom-left" => (0.0, 1.0),
                                "bottom" => (0.5, 1.0),
                                "bottom-right" => (1.0, 1.0),
                                _ => {
                                    return Err(format!(
                                        "line {}: align '{w}' (center, top, bottom, left, right, top-left, top-right, bottom-left or bottom-right)",
                                        f.line
                                    ));
                                }
                            };
                            a.align = Some(Vec2::new(x, y));
                        }
                        k => {
                            return Err(format!(
                                "line {}: '{k}' is not an artboard setting (state-machine, role, name, fit, align)",
                                f.line
                            ));
                        }
                    }
                }
                out.per.push((name, a));
            }
            k => {
                return Err(format!(
                    "line {}: '{k}' is not a Rive import setting (artboards, artboard)",
                    e.line
                ));
            }
        }
    }
    Ok(out)
}

/// The file read and its settings, each artboard the settings name checked
/// against the file.
fn open(s: &Source) -> Result<(read::RivFile, Vec<String>, Settings), String> {
    let file = read::read(&s.bytes)?;
    let names = convert::names(&file);
    let set = match &s.settings {
        Some((_, doc)) => settings(&doc.body)?,
        None => Settings::default(),
    };
    for n in set
        .artboards
        .iter()
        .flatten()
        .chain(set.per.iter().map(|p| &p.0))
    {
        if !names.contains(n) {
            return Err(format!(
                "the settings name artboard '{n}', which the file does not have ({})",
                names.join(", ")
            ));
        }
    }
    Ok((file, names, set))
}

/// The artboards a `.riv` cooks, by name (each an output
/// `<file>.riv#<name>`).
pub(super) fn plan(s: &Source) -> Result<Vec<String>, String> {
    let (_, names, set) = open(s)?;
    let chosen = set.artboards.unwrap_or_else(|| names.clone());
    for (i, n) in chosen.iter().enumerate() {
        if n.is_empty() || n.contains('#') {
            return Err(format!(
                "artboard {}: a name for its id needs no '#' and is not empty ('{n}')",
                i + 1
            ));
        }
        if names.iter().filter(|m| *m == n).count() > 1 {
            return Err(format!(
                "two artboards are named '{n}'; their ids would clash, so rename one in Rive"
            ));
        }
    }
    Ok(chosen)
}

/// Cooks artboard `name` of a `.riv`.
pub(super) fn cook(
    s: &Source,
    name: &str,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let (file, names, set) = open(s).map_err(CookError)?;
    let index = names
        .iter()
        .position(|n| n == name)
        .expect("planned from the file");
    let mut a = convert::artboard(&file, index).map_err(CookError)?;
    if let Some((_, o)) = set.per.iter().find(|p| p.0 == name) {
        if let Some(m) = &o.machine {
            a.play = a
                .machines
                .iter()
                .find(|x| x.0 == *m)
                .ok_or_else(|| {
                    CookError(format!(
                        "artboard '{name}': no state machine '{m}' ({})",
                        a.machines
                            .iter()
                            .map(|x| x.0.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                })?
                .1;
        }
        if let Some(r) = &o.role {
            a.role = r.clone();
        }
        if let Some(n) = &o.name {
            a.label = n.clone();
        }
        if let Some(f) = o.fit {
            a.fit = f;
        }
        if let Some(al) = o.align {
            a.align = al;
        }
    }
    let bytes = og_ui::rive::encode(&a);
    let mut w = BlobWriter::new(
        format::RiveArtboard::type_hash(),
        format::RiveArtboard::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[RiveArtboardMeta {
            bytes: bytes.len() as u32,
        }],
        format::row_hash::<RiveArtboardMeta>(),
    )
    .raw("artboard", bytes, 16);
    let note = format!(
        "{name}: {} objects, {} animations, {} nested artboards",
        a.nodes.len(),
        a.animations.len(),
        a.nested.len()
    );
    Ok((w.finish(), vec![note]))
}

/// Artboard `name` of a `.riv`'s bytes converted, without import settings
/// (what a test scene draws; a project cooks through its settings).
pub fn artboard(bytes: &[u8], name: &str) -> Result<og_ui::rive::Artboard, String> {
    let file = read::read(bytes)?;
    let names = convert::names(&file);
    let index = names
        .iter()
        .position(|n| n == name)
        .ok_or_else(|| format!("no artboard '{name}' ({})", names.join(", ")))?;
    convert::artboard(&file, index)
}
