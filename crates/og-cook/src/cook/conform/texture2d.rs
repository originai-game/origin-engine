//! Conformance of 2D textures (docs/art.md §2.3, stream A2): every texture
//! a sprite, a flipbook, a tile set or a UI image draws is cooked to the
//! project's art contract. The source image is never written; the pass
//! puts the conformed image in the texture's `Source::conformed`, rewrites
//! the in-memory text of the sheets whose cells it re-lays out, and adds
//! the index texture (and the rotations and the asset's own palette) as
//! sources of their own. og-cook's README (§Art conformance) is the
//! contract of what it writes.
//!
//! Which textures: those a flipbook or a tile set names (cells: its frames
//! or tiles), those a 2D rig, a UI style or a map's own tileset names (a
//! layout the cook cannot move: palette and alpha only), and the textures
//! no such asset names that are 2D by themselves: `kind pixel` in their
//! import settings, or any image in a `profile "2d"` project. An image's
//! import settings may say `conform sprite` or `conform ui` for one the
//! cook would not find (a HUD image of a 3D game), `conform off` to keep
//! it as it is, and `conform { size 32 }` for the cell size it takes.
//!
//! What, in order, per cell: the `Stylise` paint fields in texel space
//! (painted styles); the pixel-art downscale to the contract's cell size
//! (`ArtPixels.spriteSizes`, a tile set's tiles to `ArtPixels.unit`) and
//! 1-bit alpha; the palette (`ArtPalette`); the line (`ArtLine`, inside
//! the cell); the rotations (`ArtPixels.rotation prerotated`).

mod art_layout;
mod cleanedge;
pub(in crate::cook) mod colour;
mod outline;
pub(in crate::cook) mod paint;
pub(in crate::cook) mod palette;
mod resample;

mod process;

pub(in crate::cook) use art_layout::paint_of;
use art_layout::{Art, Grid, Layout, grid_of, relayout};
use process::run;

use super::super::{Kind, Source, flipbook, look, settings_block, tilemap};
use super::{AssetConform, ConformAction, ConformEntry, ProjectConform, TextureRole};
use crate::look::LookFile;
use og_core::hash::StableHasher;
use og_core::ogt::{Document, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use outline::Brush;
use paint::{Paint, Wash};
use palette::{Dither, Palette};

/// Moves every conformed output's stamp when the pass's algorithms change.
const VERSION: &str = "conform-2d 2";

/// A cell of a texture: x, y, width, height in texels.
type Rect = [u32; 4];

/// An RGBA8 image.
#[derive(Clone, Debug, PartialEq)]
struct Rgba {
    w: u32,
    h: u32,
    px: Vec<[u8; 4]>,
}

impl Rgba {
    fn new(w: u32, h: u32) -> Rgba {
        Rgba {
            w,
            h,
            px: vec![[0; 4]; (w * h) as usize],
        }
    }

    fn decode(bytes: &[u8]) -> Result<Rgba, String> {
        let img = super::super::texture::decode(bytes)?;
        Ok(Rgba {
            w: img.width,
            h: img.height,
            px: img.rgba.as_chunks::<4>().0.to_vec(),
        })
    }

    fn at(&self, x: u32, y: u32) -> [u8; 4] {
        self.px[(y * self.w + x) as usize]
    }

    fn crop(&self, r: Rect) -> Rgba {
        let mut out = Rgba::new(r[2], r[3]);
        for y in 0..r[3] {
            for x in 0..r[2] {
                out.px[(y * r[2] + x) as usize] = self.at(r[0] + x, r[1] + y);
            }
        }
        out
    }

    fn blit(&mut self, x0: u32, y0: u32, cell: &Rgba) {
        for y in 0..cell.h {
            for x in 0..cell.w {
                self.px[((y0 + y) * self.w + x0 + x) as usize] = cell.at(x, y);
            }
        }
    }

    /// PNG bytes (the cook decodes them as any source image).
    fn png(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let flat: Vec<u8> = self.px.iter().flatten().copied().collect();
        image::codecs::png::PngEncoder::new(&mut out)
            .write_image(&flat, self.w, self.h, image::ExtendedColorType::Rgba8)
            .expect("an in-memory PNG");
        out
    }
}

use image::ImageEncoder;

/// What a texture is to the 2D conformance.
#[derive(Clone, Debug)]
enum Role {
    /// A sprite drawn whole: one cell, the image.
    Sprite,
    /// A sheet's cells (a flipbook's frames, a tile set's tiles).
    Sheet {
        cells: Vec<Rect>,
        grid: Option<Grid>,
        tiles: bool,
    },
    /// A layout the cook does not move (a UI style's atlas, a 2D rig's
    /// parts, a map's own tileset): palette and alpha only.
    Fixed(String),
}

/// One asset that names a texture.
#[derive(Clone, Debug)]
pub(super) struct Use {
    by: usize,
    kind: UseKind,
    off: bool,
    size: Option<u32>,
}

#[derive(Clone, Debug)]
enum UseKind {
    Flipbook(Box<og_anim::FlipbookSheet>),
    Tileset(Box<og_tilemap::Tileset>),
    Fixed(&'static str),
}

/// The result of conforming one texture.
struct Done {
    image: Rgba,
    layout: Layout,
    /// The colours mapped onto (for the index), with their source: the
    /// contract's palette id, or the asset's own.
    palette: Option<(Palette, Option<String>)>,
    index: Option<Rgba>,
    rotations: Option<(Rgba, Option<Rgba>, (u32, u32))>,
    pixel: bool,
    what: Vec<String>,
}

/// The texture ids a sheet-like asset's `texture` line names, resolved as
/// its own importer resolves it.
fn style_images(s: &Source, known: &BTreeSet<String>) -> Vec<String> {
    let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes)) else {
        return Vec::new();
    };
    let dir = s.rel.rsplit_once('/').map_or("", |(d, _)| d);
    ["atlas", "icons"]
        .iter()
        .filter_map(|k| {
            doc.body
                .get(k)?
                .values
                .first()?
                .as_str()
                .map(str::to_string)
        })
        .map(|p| {
            let rel = if dir.is_empty() {
                p.clone()
            } else {
                format!("{dir}/{p}")
            };
            if known.contains(&rel) {
                rel
            } else {
                crate::runtime::asset_id(&p).to_string()
            }
        })
        .collect()
}

const IMAGE_ROLES: [&str; 2] = ["sprite", "ui"];

/// Whether project.ogt's `profile` is `2d`.
pub(super) fn profile_2d(project: Option<&Document>) -> bool {
    project
        .and_then(|d| {
            d.body
                .get("profile")?
                .values
                .first()?
                .as_str()
                .map(|p| p == "2d")
        })
        .unwrap_or(false)
}

/// Reads each sheet's own `conform` line and takes it out of its text
/// (its importer does not know it), with or without a look.
pub(super) fn strip_sheets(sources: &mut [Source]) -> BTreeMap<usize, AssetConform> {
    let mut sheet_conform = BTreeMap::new();
    for (i, s) in sources.iter_mut().enumerate() {
        if !matches!(s.kind, Kind::Flipbook | Kind::Tileset | Kind::Rig2d) {
            continue;
        }
        if let Some((text, c)) = super::strip(&String::from_utf8_lossy(&s.bytes), &[]) {
            match c {
                Ok(c) => {
                    sheet_conform.insert(i, c);
                }
                Err(e) => s.error = Some(e),
            }
            s.bytes = text.into_bytes();
            s.hash = rehash(s, &[]);
        }
    }
    sheet_conform
}

/// Who names which texture: flipbooks, tile sets, 2D rigs, UI styles and
/// maps' own tile sets, by texture id.
pub(super) fn uses(
    sources: &[Source],
    known: &BTreeSet<String>,
    sheet_conform: &BTreeMap<usize, AssetConform>,
    project_off: &ProjectConform,
) -> BTreeMap<String, Vec<Use>> {
    let mut uses: BTreeMap<String, Vec<Use>> = BTreeMap::new();
    for (i, s) in sources.iter().enumerate() {
        if s.error.is_some() {
            continue;
        }
        let c = sheet_conform.get(&i);
        let mut add = |id: String, kind: UseKind| {
            uses.entry(id).or_default().push(Use {
                by: i,
                kind,
                off: c.is_some_and(|c| c.off) || project_off.off_by(&s.rel).is_some(),
                size: c.and_then(|c| c.size),
            });
        };
        match s.kind {
            Kind::Flipbook => {
                if let Ok(sheet) = flipbook::sheet(s) {
                    add(sheet.texture.clone(), UseKind::Flipbook(Box::new(sheet)));
                }
            }
            Kind::Tileset => {
                if let Ok(t) = og_tilemap::Tileset::parse(&String::from_utf8_lossy(&s.bytes))
                    && !t.texture.is_empty()
                {
                    add(
                        crate::runtime::asset_id(&t.texture).to_string(),
                        UseKind::Tileset(Box::new(t)),
                    );
                }
            }
            Kind::Rig2d => {
                if let Ok(doc) = Document::parse(&String::from_utf8_lossy(&s.bytes))
                    && let Some(p) = doc
                        .body
                        .get("texture")
                        .and_then(|e| e.values.first()?.as_str())
                {
                    add(flipbook::resolve(s, p), UseKind::Fixed("a 2D rig's parts"));
                }
            }
            Kind::Style => {
                for id in style_images(s, known) {
                    add(id, UseKind::Fixed("a UI style's atlas"));
                }
            }
            Kind::Tilemap => {
                if let Ok(m) = tilemap::MapSettings::parse(settings_block(s, "tilemap"))
                    && let Ok(map) = tilemap::import(&s.rel, &s.bytes, &m, &|id| known.contains(id))
                {
                    for t in &map.tilesets {
                        if let og_tilemap::TilesetRef::Embedded(t) = t
                            && !t.texture.is_empty()
                        {
                            add(t.texture.clone(), UseKind::Fixed("a map's own tileset"));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    uses
}

/// What the texture `s` is to conformance (docs/art.md §2.3), and its own
/// `conform` line: the one decision the cook and `og style check` share
/// (`super::texture_roles`). Its own role wins (`conform ui`, `conform
/// sprite`); then who names it (a UI style, a 2D rig or a map's own tile
/// set: a fixed layout; a flipbook or a tile set: its sheet); then a
/// normal or linear map is data; then in a 2D project, or as pixel art, a
/// sprite; else, in a 3D project with nothing claiming it, a mesh's.
pub(super) fn texture_role(
    s: &Source,
    used: &[Use],
    two_d: bool,
) -> Result<(TextureRole, Option<AssetConform>), String> {
    let own = match s
        .settings
        .as_ref()
        .map(|(_, d)| AssetConform::of(&d.body, &IMAGE_ROLES))
    {
        Some(r) => r?,
        None => None,
    };
    let kind = super::super::texture_kind(s).unwrap_or(crate::format::kind::COLOR);
    let role = match own.as_ref().and_then(|c| c.role.as_deref()) {
        Some("ui") => TextureRole::Fixed,
        Some(_) => TextureRole::Sprite,
        None if used.iter().any(|u| matches!(u.kind, UseKind::Fixed(_))) => TextureRole::Fixed,
        None if !used.is_empty() => TextureRole::Sheet,
        None if kind == crate::format::kind::NORMAL || kind == crate::format::kind::LINEAR => {
            TextureRole::Data
        }
        None if two_d || kind == crate::format::kind::PIXEL => TextureRole::Sprite,
        None => TextureRole::Mesh,
    };
    Ok((role, own))
}

type SheetText = (usize, Vec<u8>, Arc<Vec<u8>>, u64);

/// Conforms the 2D textures of `sources` to the project's look
/// (`contract`: its id and art blocks), in place. Called once by
/// `cook_project` before planning; returns the report's lines and the
/// textures whose role is the mesh pass's ([`TextureRole::Mesh`]).
pub(in crate::cook) fn conform(
    root: &Path,
    assets: &Path,
    sources: &mut Vec<Source>,
    known: &BTreeSet<String>,
    contract: Option<&(String, LookFile)>,
) -> Result<(Vec<ConformEntry>, BTreeSet<String>), String> {
    let project = std::fs::read_to_string(root.join("project.ogt"))
        .ok()
        .and_then(|t| Document::parse(&t).ok());
    let project_off = match &project {
        Some(d) => ProjectConform::parse(&d.body).map_err(|e| format!("project.ogt:{e}"))?,
        None => ProjectConform::default(),
    };
    let two_d = profile_2d(project.as_ref());
    let sheet_conform = strip_sheets(sources);
    let mut entries = Vec::new();

    let Some((look_id, contract)) = contract else {
        return Ok((entries, BTreeSet::new()));
    };
    let uses = uses(sources, known, &sheet_conform, &project_off);
    // The mesh pass's textures, whatever the 2D blocks say.
    let mesh: BTreeSet<String> = sources
        .iter()
        .filter(|s| {
            s.kind == Kind::Texture
                && s.error.is_none()
                && !s.rel.to_ascii_lowercase().ends_with(".ktx2")
        })
        .filter(|s| {
            let used = uses.get(&s.rel).map_or(&[][..], Vec::as_slice);
            matches!(texture_role(s, used, two_d), Ok((TextureRole::Mesh, _)))
        })
        .map(|s| s.rel.clone())
        .collect();
    let Some(look_source) = sources.iter().find(|s| &s.rel == look_id) else {
        return Ok((entries, mesh));
    };
    let full = look::flattened(look_source, assets, &|id| known.contains(id))?;
    let Some(art) = Art::of(&full, contract, assets)? else {
        return Ok((entries, mesh));
    };

    // Each texture: its role, conformed, and what it writes.
    let mut extra: Vec<Source> = Vec::new();
    // A sheet's new text, the conformed image it reads, the look's hash.
    let mut sheet_texts: Vec<SheetText> = Vec::new();
    for i in 0..sources.len() {
        let s = &sources[i];
        if s.kind != Kind::Texture
            || s.error.is_some()
            || s.rel.to_ascii_lowercase().ends_with(".ktx2")
        {
            continue;
        }
        let used: Vec<Use> = uses.get(&s.rel).cloned().unwrap_or_default();
        let (role, own) = match texture_role(s, &used, two_d) {
            Ok(r) => r,
            Err(e) => {
                sources[i].error = Some(e);
                continue;
            }
        };
        let kind = super::super::texture_kind(s).unwrap_or(crate::format::kind::COLOR);
        let data = kind == crate::format::kind::NORMAL || kind == crate::format::kind::LINEAR;
        let role = match role {
            TextureRole::Fixed if own.as_ref().is_some_and(|c| c.role.is_some()) => {
                Some(Role::Fixed("its import settings (conform ui)".into()))
            }
            TextureRole::Fixed => used.iter().find_map(|u| match u.kind {
                UseKind::Fixed(what) => {
                    Some(Role::Fixed(format!("{what} ({})", sources[u.by].rel)))
                }
                _ => None,
            }),
            TextureRole::Sheet => sheet_role(s, &used),
            TextureRole::Sprite => Some(Role::Sprite),
            // The mesh pass's (`conform::mesh`), or data.
            TextureRole::Mesh | TextureRole::Data => None,
        };
        let Some(role) = role else { continue };
        let off_why = if own.as_ref().is_some_and(|c| c.off) {
            Some(format!("conform off (assets/{}.ogt)", s.rel))
        } else if let Some(p) = project_off.off_by(&s.rel) {
            Some(format!("project.ogt's conform block: off \"{p}\""))
        } else if let Some(u) = used.iter().find(|u| u.off) {
            Some(match project_off.off_by(&sources[u.by].rel) {
                Some(p) => format!(
                    "{} is off in project.ogt's conform block (\"{p}\")",
                    sources[u.by].rel
                ),
                None => format!("conform off in {}", sources[u.by].rel),
            })
        } else if data {
            Some("a normal or linear texture is data, not art: not conformed".into())
        } else {
            None
        };
        if let Some(why) = off_why {
            entries.push(ConformEntry {
                source: s.rel.clone(),
                action: ConformAction::Off,
                what: why,
            });
            continue;
        }
        let bytes = s.conformed.as_deref().map_or(&s.bytes[..], |b| &b[..]);
        let img = match Rgba::decode(bytes) {
            Ok(img) => img,
            Err(_) => continue, // its own cook reports the image
        };
        let size = own
            .as_ref()
            .and_then(|c| c.size)
            .or_else(|| used.iter().find_map(|u| u.size));
        let done = run(&art, &img, &role, size);
        let changed = done.image != img || done.index.is_some() || done.rotations.is_some();
        let base = s.rel.clone();
        let mut what = done.what.clone();
        let png = Arc::new(done.image.png());
        // Sheets whose cells moved get their text again; every sheet of a
        // conformed texture reads the conformed image.
        if changed {
            for u in &used {
                let text = match &u.kind {
                    UseKind::Flipbook(sheet) => {
                        let mut sheet = (**sheet).clone();
                        for f in &mut sheet.frames {
                            if let Some(r) = done.layout.moved(f.rect) {
                                f.rect = r;
                            }
                        }
                        sheet.size = [done.layout.size.0, done.layout.size.1];
                        sheet.to_text().into_bytes()
                    }
                    UseKind::Tileset(t) => match done.layout.grid {
                        Some(g) if done.layout.size != (img.w, img.h) => {
                            let mut t = (**t).clone();
                            let f = |v: f32, old: u32, new: u32| v * new as f32 / old.max(1) as f32;
                            for d in t.tiles.values_mut() {
                                if let Some(og_tilemap::Collision::Rect { x, y, w, h }) =
                                    &mut d.collision
                                {
                                    *x = f(*x, t.tile_width, g.w);
                                    *w = f(*w, t.tile_width, g.w);
                                    *y = f(*y, t.tile_height, g.h);
                                    *h = f(*h, t.tile_height, g.h);
                                }
                            }
                            t.tile_width = g.w;
                            t.tile_height = g.h;
                            t.margin = g.mx;
                            t.spacing = g.sx;
                            t.to_text().into_bytes()
                        }
                        _ => sources[u.by].bytes.clone(),
                    },
                    UseKind::Fixed(_) => continue,
                };
                if text != sources[u.by].bytes {
                    entries.push(ConformEntry {
                        source: sources[u.by].rel.clone(),
                        action: ConformAction::Changed,
                        what: format!(
                            "its cells follow {base}'s conformed layout ({}x{} texels)",
                            done.layout.size.0, done.layout.size.1
                        ),
                    });
                }
                sheet_texts.push((u.by, text, png.clone(), art.hash));
            }
        }
        // The extra outputs, as sources of their own.
        let pixel_import = "type import\nversion 1\n---\ntexture {\n  kind pixel\n}\n";
        let path = s.path.clone();
        let mut push = |rel: String, kind: Kind, bytes: Vec<u8>, settings: Option<&str>| {
            let settings =
                settings.map(|t| (t.to_string(), Document::parse(t).expect("the import text")));
            let mut src = Source {
                rel,
                path: path.clone(),
                kind,
                bytes,
                settings,
                settings_files: Vec::new(),
                hash: 0,
                gltf: None,
                error: None,
                encoded: Default::default(),
                whole: Default::default(),
                conformed: None,
                fracture: None,
            };
            src.hash = rehash(&src, &[]);
            extra.push(src);
        };
        if let Some(index) = &done.index {
            push(
                format!("{base}#index"),
                Kind::Texture,
                index.png(),
                Some(pixel_import),
            );
            what.push(format!("index texture {base}#index"));
        }
        if let Some((pal, None)) = &done.palette
            && done.index.is_some()
        {
            let hex: String = pal
                .rgb
                .iter()
                .map(|c| format!("{:02x}{:02x}{:02x}\n", c[0], c[1], c[2]))
                .collect();
            push(
                format!("{base}#palette"),
                Kind::Palette,
                hex.into_bytes(),
                None,
            );
            what.push(format!("its own palette {base}#palette"));
        }
        if let Some((rot, rot_index, (sx, sy))) = &done.rotations {
            push(
                format!("{base}#rotations"),
                Kind::Texture,
                rot.png(),
                Some(pixel_import),
            );
            let mut w = format!(
                "{} rotations by cleanEdge in {sx}x{sy} cells: {base}#rotations",
                art.rotation_steps
            );
            if let Some(ri) = rot_index {
                push(
                    format!("{base}#rotations-index"),
                    Kind::Texture,
                    ri.png(),
                    Some(pixel_import),
                );
                w.push_str(&format!(" (index {base}#rotations-index)"));
            }
            what.push(w);
        }
        let s = &mut sources[i];
        if changed {
            if done.pixel && kind != crate::format::kind::PIXEL {
                what.push("cooks as pixel art (lossless RGBA8, no mips)".into());
                set_pixel_kind(s);
            }
            s.conformed = Some(png);
        }
        s.hash = rehash(
            s,
            &[
                &art.hash.to_le_bytes(),
                format!("{role:?} {size:?}").as_bytes(),
            ],
        );
        entries.push(ConformEntry {
            source: s.rel.clone(),
            action: if changed {
                ConformAction::Changed
            } else {
                ConformAction::Unchanged
            },
            what: if changed {
                what.join("; ")
            } else {
                "meets the contract as it is".into()
            },
        });
    }
    for (i, text, png, look_hash) in sheet_texts {
        let s = &mut sources[i];
        s.bytes = text;
        s.conformed = Some(png);
        s.hash = rehash(s, &[&look_hash.to_le_bytes()]);
    }
    sources.extend(extra);
    entries.sort_by(|a, b| a.source.cmp(&b.source));
    Ok((entries, mesh))
}

/// A sheet texture's role: its cells when every sheet that names it agrees
/// on them; a tile set's tiles; else the whole image as one fixed layout.
fn sheet_role(s: &Source, used: &[Use]) -> Option<Role> {
    let size = Rgba::decode(&s.bytes).ok().map(|i| (i.w, i.h))?;
    let mut cells: Option<(Vec<Rect>, Option<Grid>, bool)> = None;
    for u in used {
        let these = match &u.kind {
            UseKind::Flipbook(sheet) => {
                let mut v: Vec<Rect> = Vec::new();
                for f in &sheet.frames {
                    if !v.contains(&f.rect) {
                        v.push(f.rect);
                    }
                }
                let g = grid_of(&v, size);
                (v, g, false)
            }
            UseKind::Tileset(t) => {
                let g = Grid {
                    cols: t.columns,
                    rows: t.rows,
                    w: t.tile_width,
                    h: t.tile_height,
                    mx: t.margin,
                    my: t.margin,
                    sx: t.spacing,
                    sy: t.spacing,
                    right: size.0.saturating_sub(
                        t.margin
                            + t.columns * t.tile_width
                            + t.columns.saturating_sub(1) * t.spacing,
                    ),
                    bottom: size.1.saturating_sub(
                        t.margin + t.rows * t.tile_height + t.rows.saturating_sub(1) * t.spacing,
                    ),
                };
                let v = (0..t.rows)
                    .flat_map(|r| {
                        (0..t.columns).map(move |c| {
                            [
                                t.margin + c * (t.tile_width + t.spacing),
                                t.margin + r * (t.tile_height + t.spacing),
                                t.tile_width,
                                t.tile_height,
                            ]
                        })
                    })
                    .collect();
                (v, Some(g), true)
            }
            UseKind::Fixed(_) => continue,
        };
        match &cells {
            None => cells = Some(these),
            Some(c) if c.0 == these.0 => {}
            Some(_) => {
                return Some(Role::Fixed(
                    "sheets that cut it into different cells".into(),
                ));
            }
        }
    }
    let (cells, grid, tiles) = cells?;
    if cells.is_empty()
        || cells
            .iter()
            .any(|c| c[2] == 0 || c[3] == 0 || c[0] + c[2] > size.0 || c[1] + c[3] > size.1)
    {
        return Some(Role::Fixed("a sheet whose cells leave the image".into()));
    }
    Some(Role::Sheet { cells, grid, tiles })
}

/// The import settings of `s` with `texture { kind pixel }`.
fn set_pixel_kind(s: &mut Source) {
    let (text, mut doc) = s
        .settings
        .take()
        .unwrap_or_else(|| (String::new(), Document::new("import", 1)));
    let kind = og_core::ogt::Entry::new("kind").with(Value::Word("pixel".into()));
    match doc.body.entries.iter_mut().find(|e| e.key == "texture") {
        Some(e) => {
            let b = e.block.get_or_insert_with(og_core::ogt::Block::new);
            b.entries.retain(|e| e.key != "kind");
            b.entries.push(kind);
        }
        None => doc.body.push(
            og_core::ogt::Entry::new("texture").with_block(og_core::ogt::Block::new().with(kind)),
        ),
    }
    s.settings = Some((text, doc));
}

/// A source's hash as the cook takes it (its id, bytes, settings), with
/// `more` after it.
fn rehash(s: &Source, more: &[&[u8]]) -> u64 {
    let mut h = StableHasher::new();
    h.write_str(&s.rel);
    h.write(&s.bytes);
    if let Some((text, _)) = &s.settings {
        h.write_str(text);
    }
    if let Some(c) = &s.conformed {
        h.write(c);
    }
    for m in more {
        h.write(m);
    }
    h.finish()
}
