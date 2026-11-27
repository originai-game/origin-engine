//! `og model bake <model> --project <view>=<image.png> … [--out <folder>]`:
//! a model's base colour painted from concept images, one image a view
//! (`front`, `back`, `left`, `right`, `top`, `three_quarter` or
//! `camera:{…}`, framed as `og model preview` frames them,
//! [`super::views`]).
//!
//! Steps, each named below: [`surface`] lays the model's triangles out as
//! og-bake's projection level (the GLB's metres, +Y up, glTF UVs, one
//! label a material); [`view_image`] fits each image onto the model's
//! silhouette in its view (the image's alpha is its silhouette, as in
//! `og model compare`); og-bake's `project` writes every texel a view
//! sees into its material's texture, keeping per texel the view that sees
//! it most squarely (the cosine of the view angle) and seeing an edge of
//! the silhouette at less; `fill_unpainted` gives a texel no view saw the
//! colour of the painted texel nearest it on the model; `padded` grows
//! each chart's gutter. [`write`] puts the textures beside the model.
//!
//! A program is baked on its parts' own UVs. A new checked `<name>.baked.ts`
//! declaration retains its parameters and overrides, with captured maps under
//! the owning module's sources/. The original declaration is never rewritten.
//! A material no view saw keeps its colour. A mesh file keeps its UVs (one without is
//! refused: `og model process` or `og model unwrap` gives it some), and
//! is rewritten with the textures as its materials' base-colour maps.
//!
//! A variant of a baked program (`og model bake content/tall.ts --from
//! content/chair.baked.ts`) is not painted from images: [`super::transfer`] carries the
//! base's maps onto it part by part.
//!
//! The report: `views` (each view's image, its fitted scale, and
//! `written`), `coverage` per material (`texels` in its charts, `covered`
//! the share a view painted, `filled` the share filled from neighbours)
//! and `output`.

use std::path::{Path, PathBuf};

use glam::{Vec2, Vec3};
use image::RgbaImage;
use og_bake::projection::{
    GBuffer, Level, Paint, Texels, ViewKind, ViewSpec, fill_unpainted, padded, project, silhouette,
};
use og_shape::tools::{Import, Material, Part, Scene, Texture, ViewCamera};
use serde_json::{Value, json};

use super::args::Args;
use og_cook::cook::model_program::Program;

use super::source::{Input, Local, Source};
use super::views::{self, Look, View};
use super::{Failure, Files, failed};

/// What to bake. The zero value has no input and no views.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BakeDesc {
    /// A program or a mesh file with UVs.
    pub input: Input,
    /// Each view and the image drawn in it.
    pub project: Vec<(View, PathBuf)>,
    /// Texels across each texture; 0 is 1024.
    pub size: u32,
    /// Pixels across each view's render; 0 is 1024.
    pub view_size: u32,
    /// Where to write; `None` beside the model.
    pub out: Option<PathBuf>,
}

/// The model's triangles as og-bake's level, every corner its own vertex
/// (so each material's texture sees its triangles alone), with each
/// triangle's material: the level, and the materials in label order.
pub(super) struct Surface {
    pub(super) positions: Vec<Vec3>,
    pub(super) uvs: Vec<Vec2>,
    pub(super) labels: Vec<u16>,
    pub(super) materials: Vec<(String, Material)>,
    /// Per triangle, its part's index in the scene.
    pub(super) parts: Vec<u32>,
}

/// A part's material key: its material's name, else the part's.
pub(super) fn key(p: &Part) -> String {
    if p.material.name.is_empty() {
        p.name.clone()
    } else {
        p.material.name.clone()
    }
}

/// The scene as a [`Surface`]; a part without UVs fails.
pub(super) fn surface(scene: &Scene) -> Result<Surface, Failure> {
    let mut s = Surface {
        positions: Vec::new(),
        uvs: Vec::new(),
        labels: Vec::new(),
        materials: Vec::new(),
        parts: Vec::new(),
    };
    for (part, p) in scene.parts.iter().enumerate() {
        if p.solid.triangles.is_empty() {
            continue;
        }
        let Some(uvs) = &p.solid.uvs else {
            return Err(Failure::Failed(format!(
                "part {:?} has no UVs: og model process or og model unwrap gives it some",
                p.name
            )));
        };
        let k = key(p);
        let label = match s.materials.iter().position(|(n, _)| *n == k) {
            Some(i) => i,
            None => {
                s.materials.push((k, p.material.clone()));
                s.materials.len() - 1
            }
        };
        for (t, corner_uvs) in p.solid.triangles.iter().zip(uvs) {
            for (c, uv) in t.iter().zip(corner_uvs) {
                let q = p.solid.positions[*c as usize];
                // Kernel mm, +Z up, to the GLB's metres, +Y up; kernel UVs
                // (v up) to glTF's (v down).
                s.positions.push(Vec3::new(
                    (q.x * 0.001) as f32,
                    (q.z * 0.001) as f32,
                    (-q.y * 0.001) as f32,
                ));
                s.uvs.push(Vec2::new(uv.x as f32, (1.0 - uv.y) as f32));
            }
            s.labels.push(label as u16);
            s.parts.push(part as u32);
        }
    }
    if s.labels.is_empty() {
        return Err(Failure::Failed("the model has no triangles".into()));
    }
    Ok(s)
}

impl Surface {
    /// The level, with the UVs of material `only`'s triangles (the others'
    /// collapsed to a point, which the rasteriser skips), or of all.
    pub(super) fn level(&self, only: Option<u16>) -> Level {
        let uvs = match only {
            None => self.uvs.clone(),
            Some(m) => self
                .uvs
                .iter()
                .enumerate()
                .map(|(i, &uv)| {
                    if self.labels[i / 3] == m {
                        uv
                    } else {
                        Vec2::ZERO
                    }
                })
                .collect(),
        };
        Level::new(
            self.positions.clone(),
            Vec::new(),
            uvs,
            (0..self.positions.len() as u32).collect(),
            self.labels.clone(),
            self.materials.iter().map(|(n, _)| n.clone()).collect(),
        )
    }
}

/// The og-bake camera of a view's camera (kernel mm to the GLB's metres).
/// og-bake's cameras take their up from +Y; a camera looking straight
/// down is turned a hair toward its own up, so its image's up is the
/// view's.
fn spec(c: &ViewCamera) -> ViewSpec {
    let glb = |v: og_shape::math::DVec3, k: f64| {
        Vec3::new((v.x * k) as f32, (v.z * k) as f32, (-v.y * k) as f32)
    };
    let (mut eye, target, up) = (glb(c.position, 0.001), glb(c.target, 0.001), glb(c.up, 1.0));
    let d = target - eye;
    if Vec2::new(d.x, d.z).length() < 1e-3 * d.length() {
        eye -= up * (1e-3 * d.length());
    }
    let mut s = ViewSpec::looking(
        eye,
        target,
        c.fov.map_or(0.0, |f| f.to_degrees() as f32),
        ViewKind::Ground,
    );
    if c.fov.is_none() {
        s.ortho = (c.ortho_height * 0.0005) as f32;
    }
    s
}

/// The image fitted onto the model's silhouette in `g` (bounding boxes
/// centred, scaled by the square root of their areas' ratio, as compare
/// aligns them): a `g.size`² RGBA image, alpha 0 where the model or the
/// image shows nothing. Also the image's pixels per view pixel.
fn view_image(g: &GBuffer, img: &RgbaImage) -> Result<(Vec<[u8; 4]>, f64), Failure> {
    use crate::commands::compare::{aligned, bbox};
    let s = g.size;
    let model: Vec<bool> = g.hits.iter().map(Option::is_some).collect();
    let (mask, _) = crate::commands::compare::silhouette(img, 8.0);
    let (iw, ih) = img.dimensions();
    let to_image = aligned(&model, s, &mask, iw).ok_or_else(|| {
        Failure::Failed(
            "no silhouette to fit: the model draws nothing in this view, or the image has no object"
                .into(),
        )
    })?;
    let scale = match (bbox(&model, s), bbox(&mask, iw)) {
        (Some(a), Some(b)) => (((b.2 - b.0) * (b.3 - b.1)) / ((a.2 - a.0) * (a.3 - a.1))).sqrt(),
        _ => 1.0,
    };
    let out = (0..s * s)
        .map(|i| {
            if !model[i as usize] {
                return [0; 4];
            }
            let (x, y) = to_image(f64::from(i % s) + 0.5, f64::from(i / s) + 0.5);
            let (x, y) = (x.floor(), y.floor());
            if x < 0.0 || y < 0.0 || x >= f64::from(iw) || y >= f64::from(ih) {
                return [0; 4];
            }
            let (x, y) = (x as u32, y as u32);
            if !mask[(y * iw + x) as usize] {
                return [0; 4];
            }
            let p = img.get_pixel(x, y);
            [p[0], p[1], p[2], 255]
        })
        .collect();
    Ok((out, scale))
}

/// The baked report (see the module doc).
pub fn bake(files: &dyn Files, d: &BakeDesc) -> Result<String, Failure> {
    if d.project.is_empty() {
        return Err(Failure::Usage(
            "give at least one --project <view>=<image.png>".into(),
        ));
    }
    let Input::Path(input) = &d.input else {
        return Err(Failure::Usage(
            "no input: give a program or a mesh file".into(),
        ));
    };
    let size = if d.size == 0 { 1024 } else { d.size };
    let images: Vec<RgbaImage> = d
        .project
        .iter()
        .map(|(_, p)| {
            image::load_from_memory(&files.read(p)?)
                .map(|i| i.to_rgba8())
                .map_err(|e| Failure::Failed(format!("{}: {e}", p.display())))
        })
        .collect::<Result<_, _>>()?;
    let paint = |scene: &Scene| paint(scene, d, &images, size);
    let (report, program) = match super::source::program_of(files, input)? {
        // Paint the same packed atlas the checked model declaration cooks.
        Some((p, read)) => {
            let (r, _) = p
                .scene(size, &read, |scene, _| {
                    paint(scene)
                        .map(|r| (r, scene_parts(scene)))
                        .map_err(|e| e.to_string())
                })
                .map_err(Failure::Failed)?;
            (r, Some(p))
        }
        None => {
            let source = super::source::load(files, &d.input, &Import::default())?;
            if let Source::Mesh(m) = &source
                && m.skinned()
            {
                return Err(Failure::Usage(
                    "bake --project paints a model at rest; this file is skinned".into(),
                ));
            }
            let r = super::preview::with_scene(&source, None, &[], |scene| {
                Ok((paint(scene)?, scene_parts(scene)))
            })?;
            (r, None)
        }
    };
    let ((views_report, coverage, maps), parts) = report;
    if maps.is_empty() {
        return Err(Failure::Failed(format!(
            "no view painted a texel: the images do not fall on the model's UVs (coverage {})",
            Value::Array(coverage)
        )));
    }
    let output = write(
        files,
        input,
        program.as_ref(),
        size,
        &parts,
        &maps,
        d.out.as_deref(),
    )?;
    Ok(json!({
        "size": size,
        "views": views_report,
        "coverage": coverage,
        "output": output,
    })
    .to_string())
}

/// What a mesh's rewrite needs of the scene: each part's name, solid and
/// material.
pub(super) type Parts = Vec<(String, og_shape::mesh::Solid, Material)>;

fn scene_parts(scene: &Scene) -> Parts {
    scene
        .parts
        .iter()
        .map(|p| (p.name.clone(), p.solid.clone(), p.material.clone()))
        .collect()
}

/// The views' reports, the coverage per material, and each painted
/// material's texture as PNG.
type Painted = (Vec<Value>, Vec<Value>, Vec<(String, Vec<u8>)>);

/// Paints every view's image into each material's texture.
fn paint(scene: &Scene, d: &BakeDesc, images: &[RgbaImage], size: u32) -> Result<Painted, Failure> {
    let vs = if d.view_size == 0 { 1024 } else { d.view_size };
    let surf = surface(scene)?;
    let whole = surf.level(None);
    let mut textures: Vec<(Texels, Paint, Vec<Value>)> = (0..surf.materials.len())
        .map(|m| {
            let t = Texels::rasterise(&surf.level(Some(m as u16)), size, 0);
            let p = Paint::new(&t);
            (t, p, Vec::new())
        })
        .collect();
    let mut views_report = Vec::new();
    let flat = Look {
        unlit: true,
        edges: false,
    };
    for ((view, path), img) in d.project.iter().zip(images) {
        let (_, camera) = views::render(scene, view, (vs, vs), flat)?;
        let g = GBuffer::cast(&whole, spec(&camera), vs, 0);
        let (painting, scale) = view_image(&g, img)?;
        let edge = silhouette(&g);
        let mut written = 0;
        for (t, p, per_view) in &mut textures {
            let r = project(t, p, &g, &edge, &painting, 1.0, 0);
            written += r.written;
            per_view.push(json!({ "view": view.name(), "written": r.written }));
        }
        views_report.push(json!({
            "view": view.name(),
            "image": path.display().to_string(),
            "imagePixelsPerViewPixel": scale,
            "written": written,
        }));
    }
    let mut coverage = Vec::new();
    let mut maps = Vec::new();
    for ((t, p, per_view), (name, material)) in textures.iter_mut().zip(&surf.materials) {
        let covered = p.coverage();
        let filled = fill_unpainted(t, p, 0);
        coverage.push(json!({
            "material": name,
            "texels": t.len(),
            "covered": covered,
            "filled": filled as f64 / t.len().max(1) as f64,
            "views": per_view,
        }));
        // A material no view saw keeps its colour, and gets no map.
        if covered > 0.0 {
            let rgb = padded(t, p, 4, super::measure::colour(material));
            maps.push((name.clone(), encode(&rgb, size)?));
        }
    }
    Ok((views_report, coverage, maps))
}

/// An RGB texture as PNG.
pub(super) fn encode(rgb: &[[u8; 3]], size: u32) -> Result<Vec<u8>, Failure> {
    let img = image::RgbImage::from_raw(size, size, rgb.concat())
        .ok_or_else(|| Failure::Failed("the texture has the wrong size".into()))?;
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png)
        .map_err(failed)?;
    Ok(out.into_inner())
}

/// The model's name: a program file's stem without `.shape`, or a mesh
/// file's stem.
fn name_of(input: &Path) -> String {
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("model");
    stem.strip_suffix(".shape").unwrap_or(stem).to_string()
}

/// Writes each material's texture as `<name>.<material>.basecolor.png`,
/// then a checked program variant or the mesh file with the maps.
pub(super) fn write(
    files: &dyn Files,
    input: &Path,
    program: Option<&Program>,
    atlas: u32,
    parts: &Parts,
    maps: &[(String, Vec<u8>)],
    out: Option<&Path>,
) -> Result<Value, Failure> {
    let dir = out
        .map(Path::to_path_buf)
        .or_else(|| input.parent().map(Path::to_path_buf))
        .unwrap_or_default();
    let name = name_of(input);
    let module = if program.is_some() {
        let absolute = std::path::absolute(&dir).map_err(failed)?;
        let root = absolute.ancestors().find(|p| p.join("og-project.json").is_file())
            .ok_or_else(|| Failure::Usage("baked program declarations must be written inside a project's content/ directory".into()))?;
        let project = og_cook::cook::source_project::Project::open(root).map_err(failed)?;
        let owner = project
            .modules
            .values()
            .filter_map(|m| {
                let module = root.join(&m.root);
                absolute
                    .strip_prefix(module.join("content"))
                    .ok()
                    .map(|rel| (module, m.id.clone(), rel.to_path_buf()))
            })
            .max_by_key(|(root, _, _)| root.components().count())
            .ok_or_else(|| {
                Failure::Usage(
                    "write a baked program declaration inside its owning module's content/".into(),
                )
            })?;
        if super::source::extension(input) == "ts" {
            let input = std::path::absolute(input).map_err(failed)?;
            let index = og_cook::cook::source_index::Index::open(root, &project).map_err(failed)?;
            if !index
                .entries
                .values()
                .any(|entry| root.join(&entry.file) == input && entry.module == owner.1)
            {
                return Err(Failure::Usage(
                    "write the baked variant in the input declaration's owning module".into(),
                ));
            }
        }
        Some(owner)
    } else {
        None
    };
    let mut textures = Vec::new();
    let mut files_of = Vec::new();
    for (material, png) in maps {
        let file = format!(
            "{name}.{}.basecolor.png",
            super::process::file_stem(material)
        );
        let file = if let Some((_, _, local)) = &module {
            Path::new("sources/baked")
                .join(local)
                .join(&name)
                .join(file)
                .to_string_lossy()
                .replace('\\', "/")
        } else {
            file
        };
        let path = module
            .as_ref()
            .map_or_else(|| dir.join(&file), |(root, _, _)| root.join(&file));
        if module.is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(failed)?;
        }
        files.write(&path, png)?;
        textures.push(
            json!({ "material": material, "path": path.display().to_string(), "bytes": png.len() }),
        );
        files_of.push((material.clone(), file, png));
    }
    let model = match program {
        Some(p) => {
            let path = dir.join(format!("{name}.baked.ts"));
            let (module_root, module_id, local) = module.as_ref().unwrap();
            let mut desc = p.authoring.clone().unwrap_or_else(|| {
                json!({
                    "program":{"$ogRef":p.file}, "params":p.params, "materials":[]
                })
            });
            if p.file.ends_with(".shape.json") {
                let program_file = dir.join(format!("{name}.program.ts"));
                let mut value: Value = serde_json::from_str(&p.text).map_err(failed)?;
                fn inputs(value: &mut Value, names: &std::collections::BTreeMap<String, String>) {
                    match value {
                        Value::Object(o) => {
                            for (key, v) in o {
                                if matches!(key.as_str(), "font" | "map") {
                                    if let Some(path) = v.as_str().and_then(|s| names.get(s)) {
                                        *v = Value::String(path.clone());
                                    }
                                } else {
                                    inputs(v, names);
                                }
                            }
                        }
                        Value::Array(a) => {
                            for v in a {
                                inputs(v, names);
                            }
                        }
                        _ => {}
                    }
                }
                let mut names = std::collections::BTreeMap::new();
                for (i, (key, (_, bytes))) in p.inputs.iter().enumerate() {
                    let source = Path::new("sources/baked")
                        .join(local)
                        .join(&name)
                        .join(format!("input-{i}"))
                        .to_string_lossy()
                        .replace('\\', "/");
                    files.write(&module_root.join(&source), bytes)?;
                    names.insert(key.clone(), source);
                }
                inputs(&mut value, &names);
                let text = format!(
                    "import {{file}} from '@origingame/engine/author';\nexport default file['model-program']({});\n",
                    serde_json::to_string_pretty(&value).map_err(failed)?
                );
                files.write(&program_file, text.as_bytes())?;
                let id = local
                    .join(format!("{name}.program"))
                    .to_string_lossy()
                    .replace('\\', "/");
                desc["program"] = json!({"$ogRef":format!("{module_id}/{id}")});
            }
            desc["atlas"] = json!(atlas);
            let materials = desc
                .as_object_mut()
                .unwrap()
                .entry("materials")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .unwrap();
            for (name, file, _) in &files_of {
                if let Some(old) = materials.iter_mut().find(|m| m["name"] == *name) {
                    old["color"] = json!([1, 1, 1]);
                    old["map"] = json!(file);
                } else {
                    materials.push(json!({"name":name,"color":[1,1,1],"map":file}));
                }
            }
            let value = json!({"kind":"program","desc":desc});
            let text = format!(
                "import {{file}} from '@origingame/engine/author';\nexport default file.model({});\n",
                serde_json::to_string_pretty(&value).map_err(failed)?
            );
            files.write(&path, text.as_bytes())?;
            json!({ "path": path.display().to_string(), "bytes": text.len() })
        }
        None => {
            let path = dir.join(format!("{name}.glb"));
            let parts: Vec<Part> = parts
                .iter()
                .map(|(n, s, m)| {
                    let mut p = Part::new(n, s);
                    p.material = m.clone();
                    if let Some((_, file, png)) = files_of.iter().find(|(k, ..)| *k == key(&p)) {
                        p.material.map = Some(Texture::from_bytes(file, png.to_vec()));
                        p.material.color = None;
                    }
                    p
                })
                .collect();
            let bytes = og_shape_formats::gltf::write::glb(&Scene {
                name: name.clone(),
                parts,
            })
            .map_err(failed)?;
            files.write(&path, &bytes)?;
            json!({ "path": path.display().to_string(), "bytes": bytes.len() })
        }
    };
    Ok(json!({ "textures": textures, "model": model }))
}

/// `--project <view>=<image>`, each.
fn projections(a: &Args) -> Result<Vec<(View, PathBuf)>, Failure> {
    a.values("--project")?
        .into_iter()
        .map(|v| {
            let (view, image) = v.split_once('=').ok_or_else(|| {
                Failure::Usage(format!("--project {v:?}: give <view>=<image.png>"))
            })?;
            Ok((View::parse(view)?, PathBuf::from(image)))
        })
        .collect()
}

/// The command line.
pub fn run(a: &Args) -> Result<String, Failure> {
    a.only(&[
        "--project",
        "--from",
        "--size",
        "--view-size",
        "-o",
        "--out",
    ])?;
    let out = a.out_opt()?;
    if let Some(dir) = &out {
        std::fs::create_dir_all(dir)
            .map_err(|e| Failure::Failed(format!("{}: {e}", dir.display())))?;
    }
    let project = projections(a)?;
    if let Some(base) = a.value(&["--from"])? {
        if !project.is_empty() {
            return Err(Failure::Usage(
                "give --project images or --from a baked base, not both".into(),
            ));
        }
        return super::transfer::transfer(
            &Local,
            &super::transfer::TransferDesc {
                input: a.input()?,
                base: PathBuf::from(base),
                size: a.number("--size")?.unwrap_or(0),
                out,
            },
        );
    }
    bake(
        &Local,
        &BakeDesc {
            input: a.input()?,
            project,
            size: a.number("--size")?.unwrap_or(0),
            view_size: a.number("--view-size")?.unwrap_or(0),
            out,
        },
    )
}
