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

use super::{CookError, Source, execute};
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

/// One font asset's import settings (`font <name> { … }`).
#[derive(Clone, Debug, Default, PartialEq)]
struct FontAssetSettings {
    /// A referenced font's file under `assets/` (an embedded one has none).
    source: Option<String>,
    /// Text set at run time renders from the font's outlines (as any
    /// og-ui font's `runtime true`).
    runtime: bool,
    /// More characters and named sets than the runs' own.
    chars: String,
    charset: Vec<String>,
}

/// A `.riv`'s import settings (`<file>.riv.ogt`, `type import`).
#[derive(Clone, Debug, Default, PartialEq)]
struct Settings {
    /// The artboards to cook (`None`: all).
    artboards: Option<Vec<String>>,
    per: Vec<(String, ArtboardSettings)>,
    fonts: Vec<(String, FontAssetSettings)>,
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
            "font" => {
                let name = word_or_string(e, 0)?;
                let mut f = FontAssetSettings::default();
                for g in e.block.iter().flat_map(|b| &b.entries) {
                    match g.key.as_str() {
                        "source" => f.source = Some(word_or_string(g, 0)?),
                        "chars" => f.chars = word_or_string(g, 0)?,
                        "charset" => {
                            f.charset = (0..g.values.len())
                                .map(|i| word_or_string(g, i))
                                .collect::<Result<_, _>>()?
                        }
                        "runtime" => {
                            f.runtime =
                                g.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
                                    format!("line {}: runtime takes true or false", g.line)
                                })?
                        }
                        k => {
                            return Err(format!(
                                "line {}: '{k}' is not a font setting (source, runtime, chars, charset)",
                                g.line
                            ));
                        }
                    }
                }
                out.fonts.push((name, f));
            }
            k => {
                return Err(format!(
                    "line {}: '{k}' is not a Rive import setting (artboards, artboard, font)",
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
    let fonts = convert::font_assets(&file);
    for (n, f) in &set.fonts {
        let Some(asset) = fonts.iter().find(|a| names_font(n, a.0, &a.1)) else {
            return Err(format!(
                "the settings name font '{n}', which the file does not have ({})",
                fonts
                    .iter()
                    .map(|a| a.1.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        };
        if asset.2.is_some() && f.source.is_some() {
            return Err(format!(
                "font '{n}' is embedded in the file; its settings name no source"
            ));
        }
    }
    Ok((file, names, set))
}

/// Whether settings' `font <n>` names font asset `index` called `name` in
/// Rive: by that name, or as its output does (`font<index>`).
fn names_font(n: &str, index: usize, name: &str) -> bool {
    n == name || n == format!("font{index}")
}

/// A font the `.riv` sets text in, as the cook makes it: its asset index,
/// its name in Rive, the font file's bytes (embedded, or the referenced
/// one's source) and its cook's settings, the charset the characters its
/// runs show.
pub(super) struct RivFont {
    pub index: usize,
    pub name: String,
    pub bytes: Vec<u8>,
    pub settings: super::FontSettings,
}

/// The fonts a `.riv`'s text uses, each with what it cooks from.
fn fonts(
    s: &Source,
    file: &read::RivFile,
    set: &Settings,
    base: &super::FontSettings,
) -> Result<Vec<RivFont>, String> {
    let chars = convert::font_chars(file)?;
    let mut out = Vec::new();
    for (index, name, embedded) in convert::font_assets(file) {
        let Some(used) = chars.get(&index) else {
            continue;
        };
        let own = set
            .fonts
            .iter()
            .find(|f| names_font(&f.0, index, &name))
            .map(|f| &f.1);
        let bytes = match (embedded, own.and_then(|f| f.source.as_ref())) {
            (Some(b), _) => b.to_vec(),
            (None, Some(src)) => execute::read_sibling_bytes(s, src)
                .map_err(|e| format!("font '{name}': source \"{src}\": {e}"))?,
            // The artboard that draws with it names it.
            (None, None) => continue,
        };
        // A face with none of what its runs show would draw nothing of it.
        if let Ok(face) = ttf_parser::Face::parse(&bytes, 0)
            && !used
                .chars()
                .any(|c| c.is_whitespace() || face.glyph_index(c).is_some())
        {
            return Err(format!(
                "font '{name}' has none of the characters its runs show (\"{used}\"); Rive draws them as missing glyphs: set them in a font that has them"
            ));
        }
        let mut settings = base.clone();
        settings.charset = own.map(|f| f.charset.clone()).unwrap_or_default();
        settings.chars = format!("{used}{}", own.map_or("", |f| f.chars.as_str()));
        settings.runtime = own.is_some_and(|f| f.runtime);
        settings.features = Vec::new();
        settings.pixel = 0;
        out.push(RivFont {
            index,
            name,
            bytes,
            settings,
        });
    }
    Ok(out)
}

/// What a `.riv` cooks: its artboards by name (each an output
/// `<file>.riv#<name>`), its embedded images by asset index (each a
/// texture `<file>.riv#image<n>`) and the fonts its text uses (each a font
/// `<file>.riv#font<n>`).
pub(super) type Plan = (Vec<String>, Vec<usize>, Vec<RivFont>);

pub(super) fn plan(s: &Source, base: &super::FontSettings) -> Result<Plan, String> {
    let (file, names, set) = open(s)?;
    let fonts = fonts(s, &file, &set, base)?;
    let images = convert::embedded_images(&file)
        .into_iter()
        .map(|i| i.0)
        .collect();
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
    Ok((chosen, images, fonts))
}

/// Cooks a `.riv`'s font into a font blob (charset: the characters its
/// runs show; `runtime` only when its settings say so).
pub(super) fn cook_font(
    f: &RivFont,
    threads: usize,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let (bytes, notes) = super::font::cook(&f.bytes, &f.settings, threads, stamp)
        .map_err(|e| CookError(format!("font{} '{}': {}", f.index, f.name, e.0)))?;
    Ok((
        bytes,
        vec![format!("font{} '{}': {}", f.index, f.name, notes.note)],
    ))
}

/// Cooks a `.riv`'s embedded image `n` into a colour texture.
pub(super) fn cook_image(
    s: &Source,
    n: usize,
    options: &super::TextureOptions,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let file = read::read(&s.bytes).map_err(CookError)?;
    let bytes = convert::embedded_images(&file)
        .into_iter()
        .find(|i| i.0 == n)
        .map(|i| i.1)
        .ok_or_else(|| CookError(format!("no embedded image {n}")))?;
    let img = super::texture::decode(bytes).map_err(|e| CookError(format!("image {n}: {e}")))?;
    let note = format!("image{n}: {}x{}", img.width, img.height);
    super::texture::cook(&img, format::kind::COLOR, options, stamp).map(|b| (b, vec![note]))
}

/// Gives every image and font of `a` (and the artboards it nests) its
/// asset id under `rel`.
fn place_images(a: &mut og_ui::rive::Artboard, rel: &str) {
    for i in &mut a.images {
        i.id = format!("{rel}{}", i.id);
    }
    for f in &mut a.fonts {
        f.id = format!("{rel}{}", f.id);
    }
    for n in &mut a.nested {
        place_images(n, rel);
    }
}

/// The referenced fonts' bytes the settings supply, by asset index.
fn given_fonts(
    s: &Source,
    file: &read::RivFile,
    set: &Settings,
) -> Result<std::collections::HashMap<usize, Vec<u8>>, String> {
    let mut out = std::collections::HashMap::new();
    for (index, name, embedded) in convert::font_assets(file) {
        if embedded.is_some() {
            continue;
        }
        if let Some(src) = set
            .fonts
            .iter()
            .find(|f| names_font(&f.0, index, &name))
            .and_then(|f| f.1.source.as_ref())
        {
            let bytes = execute::read_sibling_bytes(s, src)
                .map_err(|e| format!("font '{name}': source \"{src}\": {e}"))?;
            out.insert(index, bytes);
        }
    }
    Ok(out)
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
    let given = given_fonts(s, &file, &set).map_err(CookError)?;
    let mut a = convert::artboard(&file, index, &given).map_err(CookError)?;
    place_images(&mut a, &s.rel);
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
    let mut note = format!(
        "{name}: {} objects, {} animations, {} nested artboards",
        a.nodes.len(),
        a.animations.len(),
        a.nested.len()
    );
    if !a.fonts.is_empty() {
        note.push_str(&format!(
            ", {} texts in {} fonts",
            a.nodes
                .iter()
                .filter(|n| matches!(n.kind, og_ui::rive::Kind::Text { .. }))
                .count(),
            a.fonts.len()
        ));
    }
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
    convert::artboard(&file, index, &std::collections::HashMap::new())
}
