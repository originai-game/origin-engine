//! `shape process <mesh> (--budget N | --ratio R) [-o out.glb]`: the mesh tools'
//! chain on one file (`guides/meshtools.md`): import → join → repair →
//! decimate to the triangle budget (or `--ratio R` of what the repair
//! leaves) → place → unwrap → textures → measure →
//! export,
//! as one JSON report of every step. The place step (turn, size, pivot:
//! `shape place`) runs only when asked, and only then has a `place` key
//! in the report; it comes after the steps that drop or move vertices,
//! so the output stands on the base it reports. The repair's and the
//! decimation's lengths are the file's. The output is a plain GLB with the UVs
//! (one node, one mesh), or OBJ or STL without them. Repair and unwrap
//! take their zero values unless set, as in `shape repair` and
//! `shape unwrap`.
//!
//! A mesh that carries UVs from its file keeps them, and its material with
//! every map (when its parts share one material), through repair and
//! decimation: the unwrap step is then skipped (`"unwrap": null`) unless
//! `--unwrap` asks for fresh UVs, which drops the old UVs before
//! decimation (their seams no longer hold it back) and every map (each
//! is listed in the material's `dropped`). Vertex colours and the
//! material's factors are kept either way. `--max-texture PX` shrinks
//! each map kept whose longest side is over `PX` texels
//! (`og_shape_formats::texture::downsize`) and reports each map's size before and
//! after (`textures`).
//!
//! `--bake PX` keeps the look through a fresh unwrap: the repaired mesh
//! (with its UVs, maps and vertex colours) is the source; a copy without
//! UVs or colours is decimated to the budget, its charts packed on a
//! PX × PX grid (`og_shape::tools::unwrap_atlas`; `--padding` texels apart,
//! 4 if not given), and the source's base colour (times its vertex
//! colours), metallic-roughness, emissive and occlusion maps and its
//! shading normals are baked into new maps in that layout
//! (`og_shape::tools::bake`; `--bake-distance` in the file's units, mm for a
//! glTF, 2 % of the diagonal if not given). The GLB carries the baked
//! maps, the material's factors and `TANGENT`. The report gains `bake`
//! (what was baked, texels without a source, the distances, the normal
//! map's angles), and `unwrap` is the atlas's report (its measures:
//! coverage, overlap, texel density). Without `--bake` nothing changes.
//!
//! `--out <folder>` (a folder, or a path without an extension) writes a
//! prop named after the folder instead of one file: `<name>.glb`, standing
//! on its base unless
//! `--pivot` says otherwise, its import settings `<name>.glb.ogt` (the
//! cook's levels of detail and baked triangle-mesh collider) and its
//! convex hulls `<name>.collider.glb`. `--height 1.2` scales it to that
//! height, `--size x,y,z` (GLB axes, metres, 0 for a free axis) to the
//! one scale nearest every axis given. The report then adds the facts of
//! `og model measure` (`size` in metres, `triangles`, the look's facts)
//! and `scale`, `leftover`, `textureBytes` (and `textureBytesBefore`, the
//! file's maps as read), `edges` (open and non-manifold, `before` the
//! repair and `after`), `lods` and `collider`. The repair does what its
//! options ask (`--weld MM`, `--non-manifold`, `--holes N`, …) and nothing
//! by itself; `--max-texture PX` shrinks the maps.

use std::path::PathBuf;

use og_shape::tools::{
    Decimate, ImageInfo, Import, Material, Measure, Pivot, Place, Repair, Size, Unwrap,
};

use og_shape_formats::texture::Downsize;

use og_shape_formats::texture::downsize;

use super::args::{Args, IMPORT_OPTIONS};
use super::report::{self, object};
use super::source::{Input, Local, load_mesh};
use super::{Failure, Files, failed};

/// The chain. The zero value has no input and no budget (no
/// decimation).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProcessDesc {
    /// The mesh file.
    pub input: Input,
    /// How to read the mesh file: its unit (OBJ) and axes.
    pub import: Import,
    /// The placement after decimation; the zero value skips it. Its `up`
    /// and `front` are kernel axes; the file's go in `import`.
    pub place: Place,
    /// The repair.
    pub repair: Repair,
    /// Decimate to at most this many triangles; 0 skips decimation, and
    /// so does a mesh already within it.
    pub budget: usize,
    /// Decimate to this share of the triangles the repair leaves, over 0
    /// and at most 1; 0 is off. Not with `budget`.
    pub ratio: f64,
    /// `Decimate::merge_charts`: UV islands that agree across a seam up
    /// to a rigid move within this distance are one chart for the seam
    /// rule; 0 is off.
    pub merge_charts: f64,
    /// `Decimate::cross_color_borders`: collapse across hard vertex-colour
    /// borders as anywhere.
    pub cross_color_borders: bool,
    /// The unwrap.
    pub unwrap: Unwrap,
    /// Unwrap even a mesh that carries UVs (its texture is then not
    /// written: it was painted for the old UVs).
    pub fresh_uvs: bool,
    /// Count self-intersecting triangle pairs in the final measures.
    pub intersections: bool,
    /// Shrink every map kept whose longest side is over this many texels
    /// (box filter, written as PNG); 0 keeps them as they are.
    pub texture_max: u32,
    /// Write every map kept as KTX2 (UASTC blocks with mip levels,
    /// Zstandard: `KHR_texture_basisu`), after any shrinking.
    pub ktx2: bool,
    /// With `ktx2` (which it implies): let a UASTC block be a copy of a
    /// neighbour where that costs at most this much squared error a
    /// channel of a texel (`og_shape_formats::texture::to_ktx2_rdo`); 0 does not.
    pub ktx2_rdo: f64,
    /// Write every map kept as KTX2 of ETC1S blocks under BasisLZ: far
    /// smaller files than `ktx2`'s UASTC, at a lower quality.
    pub etc1s: bool,
    /// Write every map that is not KTX2 as lossless WebP
    /// (`EXT_texture_webp`, `og_shape_formats::texture::to_webp`): every texel as it
    /// is. Not with `ktx2` or `etc1s`.
    pub webp: bool,
    /// Bake the source's look into maps of this many texels across, on
    /// fresh UVs (implies `fresh_uvs`); 0 does not bake.
    pub bake: u32,
    /// How far from the low surface a source point may be when baking, in
    /// the mesh's units (mm); 0 is 2 % of the diagonal.
    pub bake_distance: f64,
    /// Where to write the result, if anywhere.
    pub out: Option<PathBuf>,
    /// Write the result as a prop into this folder instead of `out`:
    /// `<name>.glb`, its import settings `<name>.glb.ogt` (the cook's
    /// levels of detail and baked collider) and its convex hulls
    /// `<name>.collider.glb`. The pivot is then the base unless placed
    /// otherwise.
    pub prop: Option<PathBuf>,
    /// The project whose look a prop's facts are measured against.
    pub project: Option<PathBuf>,
}

/// The levels of detail a prop's import settings ask the cook for: each
/// level's share of the full mesh's triangles.
pub const PROP_LODS: [f64; 3] = [1.0, 0.5, 0.25];

/// Takes every map off a material whose UVs were replaced, and says so.
pub fn drop_maps(m: &mut Material) -> Vec<String> {
    m.maps_mut()
        .into_iter()
        .filter_map(|(kind, slot)| {
            slot.take().map(|t| {
                format!(
                    "{kind} {:?}: painted for the file's UVs, which the unwrap replaced",
                    t.name
                )
            })
        })
        .collect()
}

/// Each map's size, as reported.
fn map_size(i: &ImageInfo) -> String {
    object(&[
        ("width", i.width.map_or("null".into(), |w| w.to_string())),
        ("height", i.height.map_or("null".into(), |h| h.to_string())),
        ("file_bytes", i.file_bytes.to_string()),
        (
            "decoded_bytes_with_mips",
            i.decoded_bytes().map_or("null".into(), |b| b.1.to_string()),
        ),
    ])
}

/// Shrinks the material's maps to `max` texels (0: none) and, with
/// `ktx2`, writes each as KTX2; per map, its kind, name, size before
/// and after (null when kept as it was), and why it could not be read.
fn texture_budget(
    m: &mut Material,
    max: u32,
    (ktx2, etc1s, rdo): (bool, bool, f64),
    webp: bool,
) -> String {
    let ktx2 = ktx2 || etc1s || rdo > 0.0;
    if max == 0 && !ktx2 && !webp {
        return "null".into();
    }
    let mut out = Vec::new();
    for (kind, slot) in m.maps_mut() {
        let Some(t) = slot else { continue };
        let before = ImageInfo::from_bytes(&t.name, &t.bytes);
        let d = Downsize {
            max,
            color: Material::is_color_map(kind),
        };
        let changed = downsize(t, &d).and_then(|small| {
            let from = small.as_ref().unwrap_or(t);
            if !ktx2 {
                return Ok(if webp {
                    og_shape_formats::texture::to_webp(from)?.or(small)
                } else {
                    small
                });
            }
            let packed = if etc1s {
                og_shape_formats::texture::to_ktx2_etc1s(from, d.color)?
            } else {
                og_shape_formats::texture::to_ktx2_rdo(from, d.color, rdo)?
            };
            Ok(packed.or(small))
        });
        let (after, error) = match changed {
            Ok(Some(new)) => {
                let info = ImageInfo::from_bytes(&new.name, &new.bytes);
                *t = new;
                (map_size(&info), "null".to_string())
            }
            Ok(None) => ("null".to_string(), "null".to_string()),
            Err(e) => ("null".to_string(), report::string(&e.to_string())),
        };
        out.push(object(&[
            ("kind", report::string(kind)),
            ("name", report::string(&before.name)),
            ("before", map_size(&before)),
            ("after", after),
            ("error", error),
        ]));
    }
    report::array(&out)
}

/// The chain's report: `import`, `repair`, `decimate` (null when
/// skipped), `unwrap`, `material` (what is written, with what was
/// dropped), `textures` (null without a limit), `measures` and
/// `output`.
pub fn process(files: &dyn Files, d: &ProcessDesc) -> Result<String, Failure> {
    if d.prop.is_some() && d.out.is_some() {
        return Err(Failure::Usage(
            "give -o FILE or a folder to write the prop into, not both".into(),
        ));
    }
    let mut want = d.place;
    if d.prop.is_some() && want.pivot == Pivot::Keep {
        want.pivot = Pivot::Base;
    }
    let placing = want != Place::default();
    let (import, place) = if placing {
        let format = match &d.input {
            Input::Path(p) => super::source::extension(p),
            _ => String::new(),
        };
        super::place::plain_and_place(&format, &d.import, &want)?
    } else {
        (d.import, want)
    };
    let mut mesh = load_mesh(files, &d.input, &import)?;
    let imported = report::imported(&mesh.format, &mesh.imported);
    let mut clips = mesh.clips()?;
    let joined = mesh.take_joined()?;
    // A prop reports its edges before the repair, beside after.
    let open_before = d
        .prop
        .is_some()
        .then(|| open_edges(&og_shape::tools::measure(&joined, &Measure::default())));
    let (mut fixed, r) = og_shape::tools::repair_owned(joined, &d.repair).map_err(failed)?;
    // The bake's source: the repaired mesh with its UVs and colours.
    let source = (d.bake > 0).then(|| fixed.clone());
    if source.is_some() {
        // Both are baked into the maps; neither holds the decimation back.
        fixed.colors = None;
    }
    if d.fresh_uvs || source.is_some() {
        // The file's UVs are replaced after decimation, so their seams
        // must not hold the decimation back.
        fixed.uvs = None;
    }
    let budget = if d.ratio == 0.0 {
        d.budget
    } else if d.budget > 0 || !(d.ratio > 0.0 && d.ratio <= 1.0) {
        return Err(Failure::Usage(
            "give --budget N or --ratio R (over 0, at most 1), not both".into(),
        ));
    } else {
        // Of what the repair leaves: the share asked, one triangle at least.
        ((fixed.triangles.len() as f64 * d.ratio).round() as usize).max(1)
    };
    let (low, dec) = if budget > 0 && fixed.triangles.len() > budget {
        let (low, dr) = og_shape::tools::decimate(
            &fixed,
            &Decimate {
                target: budget,
                merge_charts: d.merge_charts,
                cross_color_borders: d.cross_color_borders,
                ..Default::default()
            },
        )
        .map_err(failed)?;
        (low, report::decimate(&dr))
    } else {
        (fixed, "null".to_string())
    };
    let (mut material, defaults, mut dropped) = mesh.material();
    // Baked before placing: the bake's distances are the file's.
    let mut low = low;
    let baked = match &source {
        Some(src) => {
            let (atlas, baked) = bake_onto(&mesh, src, &mut low, d)?;
            dropped.extend(drop_maps(&mut material));
            material = baked.material;
            Some((atlas, baked.report))
        }
        None => None,
    };
    // Placed last of the steps that move vertices, so the box it stands
    // on is the output's.
    let (mut low, placed) = if placing {
        let (mut out, r) =
            og_shape::tools::place(std::slice::from_ref(&low), &place).map_err(failed)?;
        // The clips go where the mesh went.
        clips = clips
            .iter()
            .map(|c| c.moved(&r.matrix))
            .collect::<Option<_>>()
            .ok_or_else(|| Failure::Failed("the placement is singular".into()))?;
        (out.pop().unwrap_or_default(), Some(r.to_json()))
    } else {
        (low, None)
    };
    let unwrapped = if let Some((atlas, _)) = &baked {
        atlas.to_json()
    } else if low.uvs.is_none() || d.fresh_uvs {
        let uvs = og_shape::tools::unwrap(&mut low, &d.unwrap).map_err(failed)?;
        dropped.extend(drop_maps(&mut material));
        report::unwrap(&uvs)
    } else {
        "null".to_string()
    };
    let textures = texture_budget(
        &mut material,
        d.texture_max,
        (d.ktx2, d.etc1s, d.ktx2_rdo),
        d.webp,
    );
    let images_before: Vec<ImageInfo> = if d.prop.is_some() {
        source_maps(&mesh)
    } else {
        Vec::new()
    };
    let images: Vec<ImageInfo> = material
        .maps()
        .into_iter()
        .map(|(_, t)| ImageInfo::from_bytes(&t.name, &t.bytes))
        .collect();
    let m = og_shape::tools::measure(
        &low,
        &Measure {
            images: &images,
            uv_resolution: d.unwrap.resolution,
            intersections: d.intersections,
            materials: core::slice::from_ref(&material),
        },
    );
    let out = match &d.prop {
        Some(dir) => Some(dir.join(format!("{}.glb", prop_name(dir, &mesh.name)))),
        None => d.out.clone(),
    };
    let output = if baked.is_some() {
        super::export::write_solid_tangents(
            files,
            &mesh.name,
            &low,
            &material,
            &clips,
            out.as_deref(),
        )?
    } else {
        super::export::write_solid_clips(
            files,
            &mesh.name,
            &low,
            Some(&material),
            &clips,
            out.as_deref(),
        )?
    };
    let mut fields = vec![
        ("import", imported),
        ("repair", report::repair(&r)),
        ("decimate", dec),
    ];
    if let Some(p) = placed {
        fields.push(("place", p));
    }
    fields.push(("unwrap", unwrapped));
    if let Some((_, b)) = &baked {
        fields.push(("bake", b.to_json()));
    }
    fields.extend([
        ("material", report::material(&material, &defaults, &dropped)),
        ("textures", textures),
        ("measures", m.to_json()),
    ]);
    let Some(dir) = &d.prop else {
        fields.push(("output", output));
        return Ok(object(&fields));
    };
    let name = prop_name(dir, &mesh.name);
    let mut prop = prop(files, dir, &name, &low, &material, &images, &want.size, d)?;
    let before_bytes: u64 = images_before
        .iter()
        .filter_map(|i| i.decoded_bytes())
        .map(|b| b.1)
        .sum();
    prop["textureBytesBefore"] = serde_json::json!(before_bytes);
    prop["edges"] = serde_json::json!({
        "before": open_before.unwrap_or_default(),
        "after": open_edges(&m),
    });
    let mut v: serde_json::Value = serde_json::from_str(&object(&fields)).map_err(failed)?;
    if let (serde_json::Value::Object(o), serde_json::Value::Object(p)) = (&mut v, prop) {
        o.extend(p);
        let scale = o.get("place").and_then(|p| p.get("scale")).cloned();
        o.insert("scale".into(), scale.unwrap_or(serde_json::json!(1.0)));
        let model: serde_json::Value = serde_json::from_str(&output).map_err(failed)?;
        let (settings, collider) = (o.remove("settings"), o.remove("colliderFile"));
        o.insert(
            "output".into(),
            serde_json::json!({ "model": model, "settings": settings, "collider": collider }),
        );
    }
    Ok(v.to_string())
}

/// A prop's name: its folder's, else the input's.
fn prop_name(dir: &std::path::Path, input: &str) -> String {
    file_stem(dir.file_name().and_then(|n| n.to_str()).unwrap_or(input))
}

/// A mesh's open and non-manifold edges, as a prop reports them.
fn open_edges(m: &og_shape::tools::MeshMeasures) -> serde_json::Value {
    serde_json::json!({
        "boundary": m.boundary_edges,
        "nonManifold": m.non_manifold_edges,
        "closed": m.closed,
    })
}

/// The maps of the file's parts as read, each once.
fn source_maps(mesh: &super::source::Mesh) -> Vec<ImageInfo> {
    let mut seen: Vec<&og_shape::tools::Texture> = Vec::new();
    for p in &mesh.imported.parts {
        for (_, t) in p.material.maps() {
            if !seen.contains(&t) {
                seen.push(t);
            }
        }
    }
    seen.into_iter()
        .map(|t| ImageInfo::from_bytes(&t.name, &t.bytes))
        .collect()
}

/// A file name for the model `name` (its path separators and spaces
/// made dashes; `model` when empty).
fn file_stem(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| {
            if c == '/' || c == '\\' || c.is_whitespace() {
                '-'
            } else {
                c
            }
        })
        .collect();
    if s.is_empty() { "model".into() } else { s }
}

/// The prop `name`'s other files and facts: the import settings, the convex
/// hulls, and `size` (metres, the GLB's x, y up, z), `scale`, `leftover`
/// (with `--size`: per GLB axis, the size reached over the size asked,
/// less 1; null for an axis left free), `triangles`, `textureBytes` (the
/// maps decoded with their mips), `lods` (each level's share and the
/// triangles it aims at; the cook builds them), `collider` (the hulls'
/// report, mm) and the look's facts (`og model measure`).
#[allow(clippy::too_many_arguments)]
fn prop(
    files: &dyn Files,
    dir: &std::path::Path,
    name: &str,
    low: &og_shape::mesh::Solid,
    material: &Material,
    images: &[ImageInfo],
    size: &Size,
    d: &ProcessDesc,
) -> Result<serde_json::Value, Failure> {
    use serde_json::{Value, json};
    let stem = name.to_string();
    let (hulls, collider) = og_shape::tools::colliders(low, &og_shape::tools::Collide::default())
        .map_err(|e| Failure::Failed(format!("collider: {e}")))?;
    let named: Vec<(String, og_shape::mesh::Solid)> = hulls
        .into_iter()
        .enumerate()
        .map(|(k, h)| (format!("{stem}.collider.{k}"), h))
        .collect();
    let hull_path = dir.join(format!("{stem}.collider.glb"));
    let bytes =
        og_shape_formats::gltf::write::glb(&super::source::scene(&stem, &named)).map_err(failed)?;
    files.write(&hull_path, &bytes)?;
    let settings_path = dir.join(format!("{stem}.glb.ogt"));
    let lods: Vec<String> = PROP_LODS.iter().map(|r| r.to_string()).collect();
    let settings = format!(
        "type import\nversion 1\n---\n# Written by og model process: the cook builds these levels of detail\n# and bakes the triangle-mesh collider (static bodies). {stem}.collider.glb\n# holds the convex hulls, for a dynamic body's og.Collider convex shapes.\nmodel {{\n  lods {}\n  collider true\n}}\n",
        lods.join(" ")
    );
    files.write(&settings_path, settings.as_bytes())?;
    let part = og_shape::tools::Part {
        material: material.clone(),
        ..og_shape::tools::Part::new(&stem, low)
    };
    let scene = og_shape::tools::Scene {
        name: stem.clone(),
        parts: vec![part],
    };
    let contract = d
        .project
        .as_deref()
        .map(crate::style_check::contract)
        .transpose();
    let (contract, look_error) = match contract {
        Ok(c) => (c.flatten(), None),
        Err(e) => (None, Some(e)),
    };
    let mut facts = super::measure::facts(&scene, contract.as_ref());
    let reached: Vec<f64> = facts["size"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_f64).collect())
        .unwrap_or_default();
    let leftover = match size {
        // Kernel x, y (front to back), z (up) to the GLB's x, y up, z.
        Size::Fit(t) => Value::Array(
            [t.x, t.z, t.y]
                .iter()
                .zip(&reached)
                .map(|(&asked, &got)| {
                    if asked == 0.0 {
                        Value::Null
                    } else {
                        json!(got / (asked * 0.001) - 1.0)
                    }
                })
                .collect(),
        ),
        _ => Value::Null,
    };
    let triangles = low.triangles.len();
    let texture_bytes: u64 = images
        .iter()
        .filter_map(|i| i.decoded_bytes())
        .map(|b| b.1)
        .sum();
    if let Value::Object(o) = &mut facts {
        o.insert("leftover".into(), leftover);
        o.insert("textureBytes".into(), json!(texture_bytes));
        o.insert(
            "lods".into(),
            Value::Array(
                PROP_LODS
                    .iter()
                    .map(|&r| json!({ "share": r, "triangles": ((triangles as f64 * r).round() as u64).max(1) }))
                    .collect(),
            ),
        );
        o.insert(
            "collider".into(),
            serde_json::from_str(&collider.to_json()).map_err(failed)?,
        );
        o.insert(
            "settings".into(),
            json!({ "path": settings_path.display().to_string(), "bytes": settings.len() }),
        );
        o.insert(
            "colliderFile".into(),
            json!({ "path": hull_path.display().to_string(), "bytes": bytes.len(), "hulls": named.len() }),
        );
        if let Some(e) = look_error {
            o.insert("lookError".into(), Value::String(e));
        }
    }
    Ok(facts)
}

/// Packs `low`'s fresh charts at `d.bake` texels and bakes `src` into
/// them, with the file's materials (per part, by the part's face name,
/// when they differ).
fn bake_onto(
    mesh: &super::source::Mesh,
    src: &og_shape::mesh::Solid,
    low: &mut og_shape::mesh::Solid,
    d: &ProcessDesc,
) -> Result<(og_shape::tools::AtlasReport, og_shape::tools::Baked), Failure> {
    let padding = if d.unwrap.padding > 0.0 {
        d.unwrap.padding.ceil() as u32
    } else {
        4
    };
    let atlas = og_shape::tools::unwrap_atlas(
        low,
        &d.unwrap,
        &og_shape::tools::Atlas {
            size: d.bake,
            padding,
            no_turns: false,
        },
    )
    .map_err(failed)?;
    let parts = &mesh.imported.parts;
    let materials: Vec<Material> = parts.iter().map(|p| p.material.clone()).collect();
    let bare = |m: &Material| Material {
        name: String::new(),
        ..m.clone()
    };
    let same = materials.iter().all(|m| bare(m) == bare(&materials[0]));
    let material_of: Vec<u32> = if same {
        Vec::new()
    } else {
        src.face_tags
            .iter()
            .map(|tags| {
                tags.iter()
                    .find_map(|&t| {
                        let name = src.names.name(t);
                        parts.iter().position(|p| p.name == name)
                    })
                    .unwrap_or(0) as u32
            })
            .collect()
    };
    let (materials, material_of) = if same {
        (materials[..materials.len().min(1)].to_vec(), material_of)
    } else {
        (materials, material_of)
    };
    let baked = og_shape::tools::bake(
        &og_shape::tools::BakeSource {
            solid: src,
            materials: &materials,
            material_of: &material_of,
        },
        low,
        &og_shape::tools::Bake {
            size: d.bake,
            distance: d.bake_distance,
            ..Default::default()
        },
        &og_shape_formats::texture::CODEC,
    )
    .map_err(failed)?;
    if baked.report.colors_baked {
        low.colors = None;
    }
    Ok((atlas, baked))
}

/// The command line.
pub fn run(a: &Args) -> Result<String, Failure> {
    let mut known = vec![
        "-o",
        "--out",
        "--budget",
        "--ratio",
        "--merge-charts",
        "--cross-color-borders",
        "--intersections",
        "--unwrap",
        "--max-texture",
        "--ktx2",
        "--etc1s",
        "--webp",
        "--ktx2-rdo",
        "--bake",
        "--bake-distance",
        "--project",
    ];
    known.extend(IMPORT_OPTIONS);
    known.extend(super::repair::OPTIONS);
    known.extend(super::unwrap::OPTIONS);
    known.extend(super::place::OPTIONS);
    a.only(&known)?;
    let mut repair = super::repair::options(a)?;
    repair.check_intersections = false;
    let input = a.input()?;
    let out = a.out_opt()?;
    if let Some(dir) = out.as_ref().filter(|o| is_folder(o)) {
        std::fs::create_dir_all(dir)
            .map_err(|e| Failure::Failed(format!("{}: {e}", dir.display())))?;
    }
    process(
        &Local,
        &ProcessDesc {
            import: a.import()?,
            place: super::place::options(a)?,
            repair,
            budget: a.number("--budget")?.unwrap_or(0),
            ratio: a.number("--ratio")?.unwrap_or(0.0),
            merge_charts: a.number("--merge-charts")?.unwrap_or(0.0),
            cross_color_borders: a.switch("--cross-color-borders"),
            unwrap: super::unwrap::options(a)?,
            fresh_uvs: a.switch("--unwrap"),
            intersections: a.switch("--intersections"),
            texture_max: a.number("--max-texture")?.unwrap_or(0),
            ktx2: a.switch("--ktx2"),
            etc1s: a.switch("--etc1s"),
            webp: a.switch("--webp"),
            ktx2_rdo: a.number("--ktx2-rdo")?.unwrap_or(0.0),
            bake: a.number("--bake")?.unwrap_or(0),
            bake_distance: a.number("--bake-distance")?.unwrap_or(0.0),
            out: out.clone().filter(|o| !is_folder(o)),
            prop: out.filter(|o| is_folder(o)),
            project: super::measure::project_of(a, &input)?,
            input,
        },
    )
}

/// An output that names a folder: one that is, or a path without an
/// extension.
fn is_folder(p: &std::path::Path) -> bool {
    p.is_dir() || p.extension().is_none()
}
