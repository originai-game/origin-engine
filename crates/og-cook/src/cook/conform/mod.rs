//! Conformance (docs/art.md §2.3): the cook makes every asset meet the
//! project's art contract (project.ogt's `look`, flattened, its art
//! blocks: `look::contract`) as it cooks it. A source is never written:
//! each pass rewrites the in-memory `Source` the rest of the cook reads
//! (the conformed image in `Source::conformed`, a sheet's text, an extra
//! output as a source of its own), so the planner, the stamps and the
//! cache treat a conformed asset as any other, keyed by the look's hash.
//!
//! One module per kind of asset, each called once from `cook_project`:
//! `texture2d` (textures used by sprites, flipbooks, tile sets and UI
//! images: stream A2) and `mesh` (models, the textures they use, clips and
//! flipbooks' timing: stream A3, held through `MeshOptions.conform`, its
//! changes notes starting `conform:` on the asset's report). What they share is here: the report's lines and
//! the opt-outs (`conform off` in an asset's `.ogt`, project.ogt's
//! `conform { off "<path>" }`).

pub mod mesh;
pub(super) mod texture2d;

/// Oklab palette extraction and pixel usage, shared with footage study.
pub use texture2d::palette;

use og_core::ogt::{Block, Document, Entry};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// What conformance did to one asset, for the cook report (`og cook
/// --report`, `Report::conform`): every change and every opt-out.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConformEntry {
    /// The asset id (under `assets/`).
    pub source: String,
    pub action: ConformAction,
    /// What changed, or why it opted out, in a sentence.
    pub what: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ConformAction {
    /// The cooked asset differs from its source to meet the contract.
    Changed,
    /// The asset meets the contract as it is.
    Unchanged,
    /// The asset opted out: `conform off`, or project.ogt's `conform` block.
    Off,
}

impl ConformAction {
    pub fn name(self) -> &'static str {
        match self {
            ConformAction::Changed => "changed",
            ConformAction::Unchanged => "unchanged",
            ConformAction::Off => "off",
        }
    }
}

/// What a texture is to conformance (docs/art.md §2.3): which pass holds
/// it to the art contract, and how `og style check-project` judges it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureRole {
    /// Drawn whole as a sprite (the 2D pass).
    Sprite,
    /// A flipbook's frames or a tile set's tiles (the 2D pass).
    Sheet,
    /// A layout the cook does not move: a UI style's atlas, a 2D rig's
    /// parts, a map's own tile set, an image with `conform ui` (the 2D
    /// pass: palette and alpha).
    Fixed,
    /// On a mesh: in a 3D project, a texture nothing else claims (the mesh
    /// pass, sized by ArtTexels.maxSize). A model's own textures are its
    /// outputs (`<model>#tex<n>`), not sources of their own.
    Mesh,
    /// A normal or linear map: data, not art.
    Data,
}

/// The role of every texture of the project at `root` that cooks on its
/// own (`.ktx2` files, which the cook passes through, and images whose
/// import settings do not read are left out): the decision the cook's
/// passes make, read from the project's sources as the cook reads them,
/// for `og style check-project`.
pub fn texture_roles(root: &Path) -> Result<BTreeMap<String, TextureRole>, String> {
    let assets = root.join("assets");
    let mut sources = super::scan(&assets).map_err(|e| e.0)?.sources;
    let project = super::source_project::Project::open(root).map_err(|e| e.to_string())?;
    let two_d = matches!(
        project.input.create.as_ref().and_then(|c| c.form.as_ref()),
        Some(super::source_schemas::project::Form::TwoD)
    );
    let project_off = ProjectConform {
        off: project.input.conform.map_or_else(Vec::new, |c| c.off),
    };
    let known: BTreeSet<String> = sources.iter().map(|s| s.rel.clone()).collect();
    let sheets = texture2d::strip_sheets(&mut sources);
    let uses = texture2d::uses(&sources, &known, &sheets, &project_off);
    let mut out = BTreeMap::new();
    for s in &sources {
        if s.kind != &crate::cook::texture::TEXTURE || s.rel.to_ascii_lowercase().ends_with(".ktx2")
        {
            continue;
        }
        let used = uses.get(&s.rel).map_or(&[][..], Vec::as_slice);
        if let Ok((role, _)) = texture2d::texture_role(s, used, two_d) {
            out.insert(s.rel.clone(), role);
        }
    }
    Ok(out)
}

/// What a model is to the art contract's face budgets (docs/art.md
/// §2.3), as it says (`conform character`, `conform effect`): a prop
/// unless it says otherwise (or has a skin, which makes it a character).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ModelRole {
    /// Held to `ArtShape.propFaces`.
    #[default]
    Prop,
    /// Held to `ArtShape.characterFaces`, as a skinned model is: a hero
    /// built of generators, or a rigid figure, has no skin to tell the cook.
    Character,
    /// An effect's mesh (a `type vfx` mesh element's, an emitter's `render
    /// mesh`): drawn through its effect material, not a prop, so no face
    /// budget holds or simplifies it. `og vfx import` says it of every mesh
    /// it writes.
    Effect,
}

/// The models of the project at `root` that say what they are
/// ([`model_role`]), by asset id: what the cook holds to
/// `ArtShape.characterFaces` besides the skinned ones, and the effect
/// meshes no face budget holds, read from the project's sources as the
/// cook reads them, for `og style check-project`.
pub fn model_roles(root: &Path) -> Result<BTreeMap<String, ModelRole>, String> {
    let sources = super::scan(&root.join("assets")).map_err(|e| e.0)?.sources;
    let mut out = BTreeMap::new();
    for s in sources
        .iter()
        .filter(|s| s.kind == &crate::cook::model::MODEL)
    {
        let role = model_role(s).map_err(|e| format!("assets/{}: {e}", s.rel))?;
        if role != ModelRole::Prop {
            out.insert(s.rel.clone(), role);
        }
    }
    Ok(out)
}

/// What a model says it is ([`ModelRole`]): `conform character` or
/// `conform effect` at the top of its `type model` text or of its import
/// settings (a glTF's, a `.vox`'s). `conform off` is also read here;
/// another word is an error.
pub(super) fn model_role(s: &super::Source) -> Result<ModelRole, String> {
    let text;
    let body = if s.rel.ends_with(".ogt") {
        text = Document::parse(&String::from_utf8_lossy(&s.bytes)).map_err(|e| e.to_string())?;
        &text.body
    } else {
        match &s.settings {
            Some((_, d)) => &d.body,
            None => return Ok(ModelRole::Prop),
        }
    };
    Ok(
        match AssetConform::of(body, &["character", "effect"])?.and_then(|c| c.role) {
            Some(r) if r == "character" => ModelRole::Character,
            Some(r) if r == "effect" => ModelRole::Effect,
            _ => ModelRole::Prop,
        },
    )
}

/// project.ogt's `conform { off "sprites/raw/*" "ui/logo.png" }`: the
/// assets no pass conforms, by asset id or pattern (`*` within one path
/// segment, `**` across segments, a trailing `/` a whole folder).
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct ProjectConform {
    pub off: Vec<String>,
}

impl ProjectConform {
    /// The pattern that turns `id` off, if one does.
    pub fn off_by(&self, id: &str) -> Option<&str> {
        self.off.iter().find(|p| matches(p, id)).map(String::as_str)
    }
}

/// `*` matches within one path segment, `**` across them, a trailing `/`
/// everything under that folder; anything else is the id itself.
pub(super) fn matches(pattern: &str, id: &str) -> bool {
    if let Some(dir) = pattern.strip_suffix('/') {
        return id.starts_with(dir) && id[dir.len()..].starts_with('/');
    }
    fn go(p: &[u8], s: &[u8]) -> bool {
        match p {
            [] => s.is_empty(),
            [b'*', b'*', rest @ ..] => (0..=s.len()).any(|i| go(rest, &s[i..])),
            [b'*', rest @ ..] => {
                let seg = s.iter().position(|&c| c == b'/').unwrap_or(s.len());
                (0..=seg).any(|i| go(rest, &s[i..]))
            }
            [c, rest @ ..] => s.first() == Some(c) && go(rest, &s[1..]),
        }
    }
    go(pattern.as_bytes(), id.as_bytes())
}

/// An asset's own `conform` line (its `.ogt`, or an image's import
/// settings): `conform off`, `conform <role>` (`sprite`, `ui`), and a
/// block of settings (`conform { size 32 }`, `conform sprite { size 32 }`).
#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct AssetConform {
    pub off: bool,
    pub role: Option<String>,
    pub size: Option<u32>,
    pub line: usize,
}

impl AssetConform {
    /// Reads `conform` from a document's body, if it has one.
    pub fn of(body: &Block, roles: &[&str]) -> Result<Option<AssetConform>, String> {
        let Some(e) = body.get("conform") else {
            return Ok(None);
        };
        Self::entry(e, roles).map(Some)
    }

    fn entry(e: &Entry, roles: &[&str]) -> Result<AssetConform, String> {
        let mut c = AssetConform {
            line: e.line,
            ..AssetConform::default()
        };
        let words = || {
            let mut all = vec!["off"];
            all.extend_from_slice(roles);
            all.join(", ")
        };
        match e.values.as_slice() {
            [] if e.block.is_some() => {}
            [v] => match v.as_word() {
                Some("off") => c.off = true,
                Some(w) if roles.contains(&w) => c.role = Some(w.to_string()),
                _ => {
                    return Err(format!(
                        "line {}: conform takes one of {} (and a block of settings)",
                        e.line,
                        words()
                    ));
                }
            },
            _ => {
                return Err(format!(
                    "line {}: conform takes one of {} (and a block of settings)",
                    e.line,
                    words()
                ));
            }
        }
        if let Some(b) = &e.block {
            if c.off {
                return Err(format!("line {}: conform off takes no settings", e.line));
            }
            for s in &b.entries {
                match s.key.as_str() {
                    "size" => {
                        let n = s
                            .number(0)
                            .ok()
                            .and_then(|n| n.as_u64().ok())
                            .filter(|n| (1..=4096).contains(n))
                            .ok_or_else(|| {
                                format!(
                                    "line {}: conform.size is the cell's longer side in logical pixels, 1 to 4096",
                                    s.line
                                )
                            })?;
                        c.size = Some(n as u32);
                    }
                    k => {
                        return Err(format!(
                            "line {}: conform has no setting `{k}`; it takes size",
                            s.line
                        ));
                    }
                }
            }
        }
        Ok(c)
    }
}

/// `text` without its top-level `conform` line, and that line, when it
/// has one: an asset's text as the importer that does not know the line
/// reads it. `None` when the text has none (or is not an `.ogt`).
pub(super) fn strip(text: &str, roles: &[&str]) -> Option<(String, Result<AssetConform, String>)> {
    let mut doc = Document::parse(text).ok()?;
    let at = doc.body.entries.iter().position(|e| e.key == "conform")?;
    let e = doc.body.entries.remove(at);
    let c = AssetConform::entry(&e, roles);
    Some((doc.to_text().ok()?, c))
}
