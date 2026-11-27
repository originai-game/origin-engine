//! Effekseer effects (`.efkproj`, `.efkefc`) imported as the engine's own:
//! the project's node tree converted into a native composed effect with
//! checked particle emitters, effect materials and meshes it
//! needs, so Effekseer can be the editor. Effekseer's runtime is not
//! embedded; its file formats are read as its MIT source writes them
//! (`third_party/effekseer/`).
//!
//! In `assets/` an Effekseer file is a composed effect under its own path
//! (`og.load.vfx(world, "fx/hit.efkproj")`); what it generates joins the
//! cook as `<path>#<name>`, and the textures and models it names are read
//! relative to it. `og vfx import` writes the same conversion as TypeScript to
//! edit. What the engine cannot express is a finding: a warning of the
//! cook, a line of `og vfx import`.

mod convert;
mod curve;
mod glb;
mod project;
mod value;
mod xml;

pub use convert::{Conversion, Finding, Links, Out, OutKind};

use super::{Source, normalise_rel};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The extensions of Effekseer's project files.
pub const EXTENSIONS: [&str; 2] = ["efkproj", "efkefc"];

/// The folder of `og vfx import`'s output its effects' textures share.
const TEXTURES: &str = "textures/";

/// Whether `rel` is an Effekseer project.
pub fn handles(rel: &str) -> bool {
    EXTENSIONS.contains(&super::ext(rel).as_str())
}

/// Reads and converts an Effekseer project's bytes; a `hit` gets a light
/// and a shake at its impact.
pub fn convert(bytes: &[u8], links: &Links<'_>, hit: bool) -> Result<Conversion, String> {
    let root = xml::read_project(bytes)?;
    let p = project::read(&root)?;
    convert::convert(&p, links, hit)
}

/// Editable declarations use the checked author schema, not another language
/// embedded in TypeScript. Content references are ordinary `$ogRef` values.
fn declaration(kind: &str, input: &serde_json::Value) -> String {
    format!(
        "import {{ file }} from '@origingame/engine/author';\nexport default file.{kind}({});\n",
        serde_json::to_string_pretty(input).expect("checked JSON input")
    )
}

/// The cook's step: every Effekseer source becomes native FX, and what
/// it generates joins the sources; the models it reads are consumed.
pub(crate) fn expand(sources: &mut Vec<Source>, consumed: &mut BTreeSet<String>) {
    let by_lower: BTreeMap<String, String> = sources
        .iter()
        .map(|s| (s.rel.to_ascii_lowercase(), s.rel.clone()))
        .collect();
    let mut extra = Vec::new();
    for s in sources.iter_mut() {
        if s.kind != &crate::cook::vfx::VFX || s.error.is_some() || !handles(&s.rel) {
            continue;
        }
        let root = super::assets_root(s);
        let dir = s
            .rel
            .rsplit_once('/')
            .map_or(String::new(), |(d, _)| format!("{d}/"));
        let resolve = |p: &str| {
            let id = normalise_rel(&format!("{dir}{p}"));
            by_lower
                .get(&id.to_ascii_lowercase())
                .cloned()
                .unwrap_or(id)
        };
        let read = std::cell::RefCell::new(Vec::new());
        let rel = s.rel.clone();
        let own = |leaf: &str| format!("{rel}#{leaf}");
        let asset = |p: &str| resolve(p);
        let read_file = |p: &str| {
            let id = resolve(p);
            let b = std::fs::read(root.join(&id)).ok();
            if b.is_some() {
                read.borrow_mut().push(id);
            }
            b
        };
        let links = Links {
            own: &own,
            asset: &asset,
            read: &read_file,
        };
        let c = match convert(&s.bytes, &links, false) {
            Ok(c) => c,
            Err(e) => {
                s.error = Some(format!("Effekseer import: {e}"));
                continue;
            }
        };
        // A model read is part of this effect; a texture stays its own asset.
        for id in read.into_inner() {
            if !c.textures.iter().any(|t| t.1 == id) {
                consumed.insert(id);
            }
        }
        s.fx = Some(crate::cook::Prepared::Vfx(c.vfx));
        for f in c.files {
            let fx = match f.kind {
                OutKind::Emitter => Some(crate::cook::Prepared::Emitter(
                    og_bundle::fx::cooked::Emitter::from_bytes(&f.bytes)
                        .expect("validated conversion"),
                )),
                OutKind::Effect => Some(crate::cook::Prepared::Effect(
                    og_bundle::fx::cooked::Effect::from_bytes(&f.bytes)
                        .expect("validated conversion"),
                )),
                OutKind::Model => None,
            };
            extra.push(Source {
                rel: f.id,
                path: s.path.clone(),
                kind: match f.kind {
                    OutKind::Emitter => &crate::cook::effect::EMITTER,
                    OutKind::Effect => &crate::cook::effect::EFFECT,
                    OutKind::Model => &crate::cook::model::MODEL,
                },
                settings: crate::cook::imports::Settings {
                    effect_mesh: f.kind == OutKind::Model,
                    ..Default::default()
                },
                bytes: f.bytes,
                hash: 0,
                gltf: None,
                error: None,
                encoded: Default::default(),
                whole: Default::default(),
                model: Default::default(),
                video: None,
                timeline: None,
                motion_action: None,
                strings: None,
                table: None,
                font_pack: None,
                fields: Default::default(),
                terrains: Default::default(),
                navmesh: None,
                lightmap: None,
                tileset: None,
                tilemap: None,
                animgraph: None,
                flipbook: None,
                rig2d: None,
                scene: None,
                authored_model: None,
                fx,
                conformed: None,
                fracture: None,
            });
        }
    }
    sources.extend(extra);
}

/// The findings of the imported effects, as the cook's warnings and their
/// reports' notes (on every cook, cached or not).
pub(crate) fn warnings(
    sources: &[Source],
    reports: &mut [super::AssetReport],
    warnings: &mut Vec<String>,
) {
    for s in sources
        .iter()
        .filter(|s| s.kind == &crate::cook::vfx::VFX && handles(&s.rel))
    {
        // Keep source bytes intact for input ownership and incremental stamps.
        // Raw project imports (not the shipped TS packs) rebuild their findings
        // from those bytes on cached cooks as well as cold ones.
        let root = super::assets_root(s);
        let dir = s.rel.rsplit_once('/').map_or("", |(dir, _)| dir);
        let resolve = |p: &str| {
            let id = normalise_rel(&format!("{dir}/{p}"));
            sources
                .iter()
                .find(|s| s.rel.eq_ignore_ascii_case(&id))
                .map_or(id.clone(), |s| s.rel.clone())
        };
        let own = |leaf: &str| format!("{}#{leaf}", s.rel);
        let read = |p: &str| std::fs::read(root.join(resolve(p))).ok();
        let Ok(c) = convert(
            &s.bytes,
            &Links {
                own: &own,
                asset: &resolve,
                read: &read,
            },
            false,
        ) else {
            continue;
        };
        let report = reports.iter_mut().find(|r| r.source == s.rel);
        let mut notes = Vec::new();
        for f in c.findings {
            let what = format!("warning: Effekseer import: {f}");
            warnings.push(format!("assets/{}: {what}", s.rel));
            notes.push(what);
        }
        if let Some(r) = report {
            r.notes.extend(notes);
        }
    }
}

/// What `og vfx import` wrote.
#[derive(Clone, Debug, Default)]
pub struct Imported {
    /// Every file written, the effect first.
    pub written: Vec<PathBuf>,
    /// The effect's asset id (what `og.load.vfx` takes).
    pub id: String,
    pub findings: Vec<Finding>,
}

/// `og vfx import`: converts `file` and writes into `out` the effect as
/// `<stem>.ts` and beside it `<stem>/`: its emitters and effect materials.
/// Textures and models are copied under the project's matching `assets/`
/// path, with explicit `file.import` declarations under `content/`.
/// Textures in `textures/` are shared by every effect imported
/// into `out`: one copy per file, by name, a different file of the
/// same name under its name with its content's hash (so importing again,
/// or importing more effects, is idempotent). Asset ids are relative to
/// `root` (the game project), by default the nearest folder containing
/// `og-project.json`. `out` must be inside that project's `content/`.
pub fn import(file: &Path, out: &Path, root: Option<&Path>, hit: bool) -> Result<Imported, String> {
    let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let out = out
        .canonicalize()
        .map_err(|e| format!("{}: {e}", out.display()))?;
    let root = match root {
        Some(r) => r
            .canonicalize()
            .map_err(|e| format!("{}: {e}", r.display()))?,
        None => out
            .ancestors()
            .find(|a| a.join("og-project.json").is_file())
            .ok_or("pass --root <project> or import into a game's content/ directory")?
            .to_path_buf(),
    };
    let base = out
        .strip_prefix(root.join("content"))
        .map_err(|_| format!("{} is not inside {}/content", out.display(), root.display()))?;
    let assets = root.join("assets").join(base);
    let prefix: String = base
        .components()
        .map(|c| format!("{}/", c.as_os_str().to_string_lossy()))
        .collect();
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "effect".into());
    let dir = file.parent().unwrap_or(Path::new("."));
    let read = |p: &str| std::fs::read(dir.join(p)).ok();
    // A texture's copy, shared by every effect imported into `out`:
    // `textures/<file name>`, else, when a different file holds that
    // name, the name with its content's hash.
    let shared: RefCell<BTreeMap<String, (String, Vec<u8>)>> = RefCell::default();
    let local = |p: &str| -> String {
        if let Some((rel, _)) = shared.borrow().get(p) {
            return rel.clone();
        }
        let bytes = read(p).unwrap_or_default();
        let n = normalise_rel(p);
        let name = n.rsplit('/').next().unwrap_or(&n);
        // A copy whose name differs only in case is the same file on a
        // case-insensitive file system: it is that copy, under its own
        // name, when it holds these bytes, else this one takes its hash.
        let taken = |rel: &str| -> Option<(String, bool)> {
            if let Some((r, b)) = shared
                .borrow()
                .values()
                .find(|(r, _)| r.eq_ignore_ascii_case(rel))
            {
                return Some((r.clone(), *b == bytes));
            }
            let dir = std::fs::read_dir(assets.join(TEXTURES)).ok()?;
            let r = dir
                .flatten()
                .map(|e| format!("{TEXTURES}{}", e.file_name().to_string_lossy()))
                .find(|r| r.eq_ignore_ascii_case(rel))?;
            let same = std::fs::read(assets.join(&r)).is_ok_and(|b| b == bytes);
            Some((r, same))
        };
        let mut rel = format!("{TEXTURES}{name}");
        let clash = match taken(&rel) {
            Some((on, true)) => {
                rel = on;
                false
            }
            Some((_, false)) => true,
            None => false,
        };
        if clash {
            let mut h = og_core::hash::StableHasher::new();
            h.write(&bytes);
            let hash = format!("{:08x}", h.finish() as u32);
            rel = match name.rsplit_once('.') {
                Some((s, e)) => format!("{TEXTURES}{s}-{hash}.{e}"),
                None => format!("{TEXTURES}{name}-{hash}"),
            };
        }
        shared
            .borrow_mut()
            .insert(p.to_string(), (rel.clone(), bytes));
        rel
    };
    let own = |leaf: &str| {
        let stem = format!("{prefix}{stem}/{leaf}");
        format!(
            "base/{}",
            stem.rsplit_once('.')
                .map_or(stem.as_str(), |(stem, _)| stem)
        )
    };
    let asset = |p: &str| {
        let path = local(p);
        let stem = path
            .rsplit_once('.')
            .map_or(path.as_str(), |(stem, _)| stem);
        format!("base/{prefix}{stem}")
    };
    let c = convert(
        &bytes,
        &Links {
            own: &own,
            asset: &asset,
            read: &read,
        },
        hit,
    )?;
    let write = |path: PathBuf, b: &[u8]| -> Result<PathBuf, String> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
        }
        std::fs::write(&path, b).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    };
    let mut text = declaration("vfx", &c.declaration);
    for f in &c.findings {
        text.push_str(&format!(
            "// effekseer: {}\n",
            f.to_string().replace(['\n', '\r'], " ")
        ));
    }
    let mut written = vec![write(out.join(format!("{stem}.ts")), text.as_bytes())?];
    for f in &c.files {
        let rel =
            f.id.strip_prefix(&format!("base/{prefix}"))
                .expect("generated id prefix");
        if let Some(input) = &f.declaration {
            let kind = if f.kind == OutKind::Emitter {
                "particles"
            } else {
                "effect"
            };
            written.push(write(
                out.join(format!("{rel}.ts")),
                declaration(kind, input).as_bytes(),
            )?);
        } else {
            written.push(write(assets.join(format!("{rel}.glb")), &f.bytes)?);
        }
        if f.kind == OutKind::Model {
            written.push(write(
                out.join(format!("{rel}.ts")),
                declaration(
                    "import",
                    &serde_json::json!({
                        "kind": "model", "source": format!("assets/{prefix}{rel}.glb"),
                        "conform": "effect"
                    }),
                )
                .as_bytes(),
            )?);
        }
    }
    for (p, _) in &c.textures {
        let (rel, b) = shared.borrow().get(p).cloned().unwrap_or_default();
        if rel.is_empty() || b.is_empty() {
            return Err(format!("{}: unreadable", dir.join(p).display()));
        }
        let path = assets.join(&rel);
        // A copy already there is left as it is.
        if std::fs::read(&path).is_ok_and(|on| on == b) {
            written.push(path);
        } else {
            written.push(write(path, &b)?);
        }
        let stem = rel.rsplit_once('.').map_or(rel.as_str(), |(stem, _)| stem);
        written.push(write(
            out.join(format!("{stem}.ts")),
            declaration(
                "import",
                &serde_json::json!({
                    "kind": "texture", "source": format!("assets/{prefix}{rel}")
                }),
            )
            .as_bytes(),
        )?);
    }
    Ok(Imported {
        written,
        id: format!("base/{prefix}{stem}"),
        findings: c.findings,
    })
}
