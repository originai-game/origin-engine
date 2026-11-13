//! `og bake projection <step> --run <dir>`: a level's one texture painted
//! from views an image model paints (og-bake's `projection`, after trompe).
//!
//! The caller owns the model: for each view it runs `prepare`, sends the
//! state image (and, for models that take one, the edit mask) with a prompt
//! built from `prepare`'s parts, and hands the answer to `project`. Every
//! step reads and writes the run directory, so a step can be rerun and a run
//! resumed; nothing here reads a clock or a network.
//!
//! ```text
//! run.json            the level, texture and view sizes, the quality ratio
//! level.glb           the level as given (y up, TEXCOORD_0 painted)
//! texels.bin          every covered texel's world point and normal
//! paint.bin           every texel's colour and capture quality
//! plan.json           player points and the views in paint order
//! views/<k>/          state.png, edit.png, clay.png, masks.png, prepare.json,
//!                     painted.png (the model's), comp.png, project.json
//! texture.png         padded; textured.glb after export
//! ```

use glam::{Mat4, Vec2, Vec3};
use og_bake::projection::composite::{ViewMasks, finish};
use og_bake::projection::plan::{PlanOptions, hole_candidates, plan};
use og_bake::projection::{
    ClayLight, GBuffer, Level, Paint, Texels, ViewKind, ViewSpec, clay, padded, prepare, project,
    silhouette, unpainted_pixels,
};
use og_bake::{PackSettings, UnwrapMesh, unwrap};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const USAGE: &str = "usage: og bake projection <step> --run <dir> [options]
  init <level.glb> [--size N] [--view-size N] [--ratio R]
                     load the level (y up; without TEXCOORD_0 it is charted
                     by xatlas) and rasterise its texels
  plan [--views N] [--spacing M] [--aerial N] [--roofs N] [--seed N]
                     choose the cameras in paint order into plan.json
  prepare --view K   render view K against the texture so far: state.png,
                     edit.png (white: paint here), prepare.json (mode, parts)
  project --view K --image <painted.png>
                     composite the model's painting and project it
  holes [--max N]    add the hole view that sees the most unpainted pixels
                     to plan.json (prints its index, or null)
  export [--out <file.glb>] [--padding N]
                     texture.png (gutters padded) and the level with it as
                     every material's base colour
  options: --threads N (0: the machine's)";

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("og bake projection: {msg}");
    ExitCode::FAILURE
}

struct Args<'a> {
    positional: Vec<&'a str>,
    flags: Vec<(&'a str, &'a str)>,
}

impl<'a> Args<'a> {
    fn parse(rest: &[&'a str]) -> Result<Args<'a>, String> {
        let (mut positional, mut flags) = (Vec::new(), Vec::new());
        let mut i = 0;
        while i < rest.len() {
            if let Some(name) = rest[i].strip_prefix("--") {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| format!("--{name} takes a value"))?;
                flags.push((name, *v));
                i += 2;
            } else {
                positional.push(rest[i]);
                i += 1;
            }
        }
        Ok(Args { positional, flags })
    }
    fn get(&self, name: &str) -> Option<&'a str> {
        self.flags.iter().rev().find(|f| f.0 == name).map(|f| f.1)
    }
    fn num<T: std::str::FromStr>(&self, name: &str, default: T) -> Result<T, String> {
        match self.get(name) {
            None => Ok(default),
            Some(v) => v
                .parse()
                .map_err(|_| format!("--{name}: not a number: {v}")),
        }
    }
}

pub fn run(args: &[&str]) -> ExitCode {
    let Some((&step, rest)) = args.split_first() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let a = match Args::parse(rest) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("og bake projection: {e}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    let Some(dir) = a.get("run").map(PathBuf::from) else {
        eprintln!("og bake projection {step}: --run <dir> is required\n{USAGE}");
        return ExitCode::from(2);
    };
    let result = match step {
        "init" => init(&dir, &a),
        "plan" => plan_step(&dir, &a),
        "prepare" => prepare_step(&dir, &a),
        "project" => project_step(&dir, &a),
        "holes" => holes_step(&dir, &a),
        "export" => export_step(&dir, &a),
        _ => {
            eprintln!("og bake projection: unknown step {step}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(v) => {
            println!("{v}");
            ExitCode::SUCCESS
        }
        Err(e) => fail(e),
    }
}

// ---- the level -------------------------------------------------------------

/// The level's triangles in world space, labelled by material name (else
/// mesh name), with TEXCOORD_0, and whether every primitive had one (when
/// not, the uvs are zero and the level needs [`unwrap_level`]).
pub fn load_level(path: &Path) -> Result<(Level, bool), String> {
    let (doc, buffers, _) = gltf::import(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (mut pos, mut nrm, mut uvs, mut idx, mut tri_label) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut labels: Vec<String> = Vec::new();
    let label_of = |name: String, labels: &mut Vec<String>| -> u16 {
        match labels.iter().position(|l| *l == name) {
            Some(i) => i as u16,
            None => {
                labels.push(name);
                (labels.len() - 1) as u16
            }
        }
    };
    let mut missing_uv = false;
    let scene = doc
        .default_scene()
        .or_else(|| doc.scenes().next())
        .ok_or("the glTF has no scene")?;
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::IDENTITY)).collect();
    while let Some((node, parent)) = stack.pop() {
        let world = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        let normal_m = world.inverse().transpose();
        if let Some(mesh) = node.mesh() {
            for prim in mesh.primitives() {
                if prim.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let r = prim.reader(|b| buffers.get(b.index()).map(|d| &d.0[..]));
                let Some(p) = r.read_positions() else {
                    continue;
                };
                let base = pos.len() as u32;
                let p: Vec<[f32; 3]> = p.collect();
                let count = p.len();
                pos.extend(p.iter().map(|v| world.transform_point3(Vec3::from(*v))));
                match r.read_normals() {
                    Some(n) => nrm.extend(n.map(|v| {
                        normal_m
                            .transform_vector3(Vec3::from(v))
                            .normalize_or(Vec3::Y)
                    })),
                    None => nrm.extend(std::iter::repeat_n(Vec3::ZERO, count)),
                }
                match r.read_tex_coords(0) {
                    Some(t) => uvs.extend(t.into_f32().map(Vec2::from)),
                    None => {
                        missing_uv = true;
                        uvs.extend(std::iter::repeat_n(Vec2::ZERO, count));
                    }
                }
                let name = prim
                    .material()
                    .name()
                    .map(str::to_string)
                    .or_else(|| mesh.name().map(str::to_string))
                    .unwrap_or_else(|| "level".into());
                let label = label_of(name, &mut labels);
                let ind: Vec<u32> = match r.read_indices() {
                    Some(i) => i.into_u32().collect(),
                    None => (0..count as u32).collect(),
                };
                let flip = world.determinant() < 0.0;
                for t in ind.as_chunks::<3>().0 {
                    let t = if flip {
                        [t[0], t[2], t[1]]
                    } else {
                        [t[0], t[1], t[2]]
                    };
                    idx.extend(t.map(|i| base + i));
                    tri_label.push(label);
                }
            }
        }
        stack.extend(node.children().map(|c| (c, world)));
    }
    if idx.is_empty() {
        return Err("the level has no triangles".into());
    }
    if nrm.contains(&Vec3::ZERO) {
        nrm.clear();
    }
    Ok((
        Level::new(pos, nrm, uvs, idx, tri_label, labels),
        !missing_uv,
    ))
}

/// The level with new uvs from xatlas, charted in world space at one texel
/// density and packed into one `size`² page: the vertices are split along
/// the chart seams, the triangles and their labels keep their order.
pub fn unwrap_level(level: &Level, size: u32) -> Result<Level, String> {
    let mesh = UnwrapMesh {
        positions: level.positions.iter().map(|p| p.to_array()).collect(),
        normals: level.normals.iter().map(|n| n.to_array()).collect(),
        indices: level.indices.clone(),
    };
    let area: f32 = level
        .indices
        .as_chunks::<3>()
        .0
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|i| level.positions[i as usize]);
            (b - a).cross(c - a).length() * 0.5
        })
        .sum();
    let padding = (size / 1024).max(2);
    // Charts fill about half a page; shrink the density until they fit one.
    let mut tpu = (0.5 * (size as f32).powi(2) / area.max(1e-6)).sqrt();
    for _ in 0..16 {
        let settings = PackSettings {
            texels_per_unit: tpu,
            max_size: size,
            padding,
        };
        let a = unwrap(std::slice::from_ref(&mesh), &settings)?;
        if a.pages == 1 && a.width <= size && a.height <= size {
            let m = &a.meshes[0];
            if m.indices.len() != level.indices.len() {
                return Err(format!(
                    "xatlas returned {} indices for {}",
                    m.indices.len(),
                    level.indices.len()
                ));
            }
            let side = a.width.max(a.height) as f32;
            let at = |v: &og_bake::AtlasVertex| v.xref as usize;
            let normals = if level.normals.is_empty() {
                Vec::new()
            } else {
                m.vertices.iter().map(|v| level.normals[at(v)]).collect()
            };
            return Ok(Level::new(
                m.vertices.iter().map(|v| level.positions[at(v)]).collect(),
                normals,
                m.vertices.iter().map(|v| Vec2::from(v.uv) / side).collect(),
                m.indices.clone(),
                level.triangle_label.clone(),
                level.labels.clone(),
            ));
        }
        tpu *= 0.85;
    }
    Err(format!(
        "xatlas could not pack the level into one {size}² page"
    ))
}

/// A binary glTF of `level` as it stands: one node, one primitive per label
/// (a material of that name), world-space positions, normals when it has
/// them, TEXCOORD_0.
pub fn level_glb(level: &Level) -> Vec<u8> {
    let mut bin: Vec<u8> = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let mut push = |bin: &mut Vec<u8>, bytes: Vec<u8>, accessor: Value| {
        while !bin.len().is_multiple_of(4) {
            bin.push(0);
        }
        views.push(json!({"buffer": 0, "byteOffset": bin.len(), "byteLength": bytes.len()}));
        bin.extend(bytes);
        let mut acc = accessor;
        acc["bufferView"] = json!(views.len() - 1);
        accessors.push(acc);
        accessors.len() - 1
    };
    let f32s = |v: &mut dyn Iterator<Item = f32>| v.flat_map(f32::to_le_bytes).collect::<Vec<u8>>();
    let n = level.positions.len();
    let position = push(
        &mut bin,
        f32s(&mut level.positions.iter().flat_map(|p| p.to_array())),
        json!({"componentType": 5126, "count": n, "type": "VEC3",
            "min": level.min.to_array(), "max": level.max.to_array()}),
    );
    let normal = (!level.normals.is_empty()).then(|| {
        push(
            &mut bin,
            f32s(&mut level.normals.iter().flat_map(|p| p.to_array())),
            json!({"componentType": 5126, "count": n, "type": "VEC3"}),
        )
    });
    let uv = push(
        &mut bin,
        f32s(&mut level.uvs.iter().flat_map(|p| p.to_array())),
        json!({"componentType": 5126, "count": n, "type": "VEC2"}),
    );
    let mut primitives = Vec::new();
    for (l, _) in level.labels.iter().enumerate() {
        let tris: Vec<u32> = level
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .zip(&level.triangle_label)
            .filter(|(_, lab)| **lab as usize == l)
            .flat_map(|(t, _)| *t)
            .collect();
        if tris.is_empty() {
            continue;
        }
        let count = tris.len();
        let indices = push(
            &mut bin,
            tris.iter().flat_map(|i| i.to_le_bytes()).collect(),
            json!({"componentType": 5125, "count": count, "type": "SCALAR"}),
        );
        let mut attributes = json!({"POSITION": position, "TEXCOORD_0": uv});
        if let Some(nrm) = normal {
            attributes["NORMAL"] = json!(nrm);
        }
        primitives.push(json!({"attributes": attributes, "indices": indices, "material": l}));
    }
    let doc = json!({
        "asset": {"version": "2.0", "generator": "og bake projection"},
        "scene": 0, "scenes": [{"nodes": [0]}], "nodes": [{"mesh": 0, "name": "level"}],
        "meshes": [{"name": "level", "primitives": primitives}],
        "materials": level.labels.iter().map(|l| json!({"name": l})).collect::<Vec<_>>(),
        "buffers": [{"byteLength": bin.len()}],
        "bufferViews": views, "accessors": accessors,
    });
    glb_container(&serde_json::to_vec(&doc).unwrap_or_default(), &bin)
}

/// The GLB container around a JSON chunk and a binary chunk.
fn glb_container(json: &[u8], bin: &[u8]) -> Vec<u8> {
    let mut js = json.to_vec();
    while !js.len().is_multiple_of(4) {
        js.push(b' ');
    }
    let mut bin = bin.to_vec();
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let mut out = Vec::with_capacity(28 + js.len() + bin.len());
    out.extend(b"glTF");
    out.extend(2u32.to_le_bytes());
    out.extend(((12 + 8 + js.len() + 8 + bin.len()) as u32).to_le_bytes());
    out.extend((js.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(js);
    out.extend((bin.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(bin);
    out
}

struct Run {
    dir: PathBuf,
    size: u32,
    view_size: u32,
    ratio: f32,
    threads: usize,
}

fn read_json(p: &Path) -> Result<Value, String> {
    let s = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
    serde_json::from_str(&s).map_err(|e| format!("{}: {e}", p.display()))
}

fn write_json(p: &Path, v: &Value) -> Result<(), String> {
    std::fs::write(p, serde_json::to_string_pretty(v).unwrap_or_default())
        .map_err(|e| format!("{}: {e}", p.display()))
}

fn open_run(dir: &Path, a: &Args) -> Result<Run, String> {
    let r = read_json(&dir.join("run.json"))
        .map_err(|e| format!("{e} (run `og bake projection init` first)"))?;
    Ok(Run {
        dir: dir.to_path_buf(),
        size: r["size"].as_u64().unwrap_or(4096) as u32,
        view_size: r["viewSize"].as_u64().unwrap_or(1024) as u32,
        ratio: r["ratio"].as_f64().unwrap_or(0.4) as f32,
        threads: a.num("threads", 0usize)?,
    })
}

impl Run {
    fn level(&self) -> Result<Level, String> {
        match load_level(&self.dir.join("level.glb"))? {
            (level, true) => Ok(level),
            (_, false) => {
                Err("the run's level.glb has no uvs: run `og bake projection init` again".into())
            }
        }
    }
    fn texels(&self) -> Result<Texels, String> {
        let f = std::fs::File::open(self.dir.join("texels.bin"))
            .map_err(|e| format!("texels.bin: {e}"))?;
        Texels::read(&mut std::io::BufReader::new(f)).map_err(|e| format!("texels.bin: {e}"))
    }
    fn paint(&self, texels: &Texels) -> Result<Paint, String> {
        match std::fs::File::open(self.dir.join("paint.bin")) {
            Ok(f) => {
                Paint::read(&mut std::io::BufReader::new(f)).map_err(|e| format!("paint.bin: {e}"))
            }
            Err(_) => Ok(Paint::new(texels)),
        }
    }
    fn save_paint(&self, p: &Paint) -> Result<(), String> {
        let tmp = self.dir.join("paint.bin.tmp");
        let f = std::fs::File::create(&tmp).map_err(|e| format!("paint.bin: {e}"))?;
        p.write(&mut std::io::BufWriter::new(f))
            .map_err(|e| format!("paint.bin: {e}"))?;
        std::fs::rename(&tmp, self.dir.join("paint.bin")).map_err(|e| format!("paint.bin: {e}"))
    }
    fn views(&self) -> Result<Vec<Value>, String> {
        Ok(read_json(&self.dir.join("plan.json"))
            .map_err(|e| format!("{e} (run `og bake projection plan` first)"))?["views"]
            .as_array()
            .cloned()
            .unwrap_or_default())
    }
    fn view(&self, a: &Args) -> Result<(usize, ViewSpec, PathBuf), String> {
        let k: usize = a
            .get("view")
            .ok_or("--view K is required")?
            .parse()
            .map_err(|_| "--view: not an index")?;
        let views = self.views()?;
        let v = views
            .get(k)
            .ok_or_else(|| format!("no view {k} (plan.json has {})", views.len()))?;
        let dir = self.dir.join("views").join(k.to_string());
        std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        Ok((k, spec_from(v)?, dir))
    }
}

fn spec_json(s: &ViewSpec) -> Value {
    json!({"position": s.position.to_array(), "yaw": s.yaw, "pitch": s.pitch, "fov": s.fov, "kind": s.kind.name()})
}

fn spec_from(v: &Value) -> Result<ViewSpec, String> {
    let f = |k: &str| {
        v[k].as_f64()
            .map(|x| x as f32)
            .ok_or_else(|| format!("view without {k}"))
    };
    let p = v["position"].as_array().ok_or("view without position")?;
    let c = |i: usize| p.get(i).and_then(Value::as_f64).unwrap_or(0.0) as f32;
    Ok(ViewSpec {
        position: Vec3::new(c(0), c(1), c(2)),
        yaw: f("yaw")?,
        pitch: f("pitch")?,
        fov: f("fov")?,
        kind: v["kind"]
            .as_str()
            .and_then(ViewKind::parse)
            .unwrap_or(ViewKind::Ground),
    })
}

fn save_rgba(path: &Path, size: u32, px: &[[u8; 4]]) -> Result<(), String> {
    let buf: Vec<u8> = px.iter().flatten().copied().collect();
    image::save_buffer(path, &buf, size, size, image::ExtendedColorType::Rgba8)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn save_rgb(path: &Path, size: u32, px: &[[u8; 3]]) -> Result<(), String> {
    let buf: Vec<u8> = px.iter().flatten().copied().collect();
    image::save_buffer(path, &buf, size, size, image::ExtendedColorType::Rgb8)
        .map_err(|e| format!("{}: {e}", path.display()))
}

fn save_mask(path: &Path, size: u32, m: &[u8]) -> Result<(), String> {
    let buf: Vec<u8> = m.iter().map(|&v| if v != 0 { 255 } else { 0 }).collect();
    image::save_buffer(path, &buf, size, size, image::ExtendedColorType::L8)
        .map_err(|e| format!("{}: {e}", path.display()))
}

// ---- steps -------------------------------------------------------------------

fn init(dir: &Path, a: &Args) -> Result<Value, String> {
    let src = a.positional.first().ok_or("init <level.glb>")?;
    let size: u32 = a.num("size", 4096)?;
    let view_size: u32 = a.num("view-size", 1024)?;
    let ratio: f32 = a.num("ratio", 0.4)?;
    let threads: usize = a.num("threads", 0)?;
    if !(256..=16384).contains(&size) || !(128..=4096).contains(&view_size) {
        return Err("--size must be 256..16384 and --view-size 128..4096".into());
    }
    let t0 = Instant::now();
    let (level, has_uvs) = load_level(Path::new(src))?;
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    // A level without uvs is charted here; the run's level.glb is then the
    // charted geometry, which every later step and the export read.
    let level = if has_uvs {
        std::fs::copy(src, dir.join("level.glb")).map_err(|e| format!("{src}: {e}"))?;
        level
    } else {
        let charted = unwrap_level(&level, size)?;
        std::fs::write(dir.join("level.glb"), level_glb(&charted))
            .map_err(|e| format!("level.glb: {e}"))?;
        charted
    };
    let texels = Texels::rasterise(&level, size, threads);
    let f =
        std::fs::File::create(dir.join("texels.bin")).map_err(|e| format!("texels.bin: {e}"))?;
    texels
        .write(&mut std::io::BufWriter::new(f))
        .map_err(|e| format!("texels.bin: {e}"))?;
    let _ = std::fs::remove_file(dir.join("paint.bin"));
    let info = json!({
        "size": size, "viewSize": view_size, "ratio": ratio, "unwrapped": !has_uvs,
        "triangles": level.triangles(), "labels": level.labels,
        "bounds": [level.min.to_array(), level.max.to_array()],
        "texels": texels.len(), "uvCoverage": texels.len() as f64 / (size as f64 * size as f64),
    });
    write_json(&dir.join("run.json"), &info)?;
    Ok(json!({"step": "init", "seconds": t0.elapsed().as_secs_f32(), "run": info}))
}

fn plan_step(dir: &Path, a: &Args) -> Result<Value, String> {
    let r = open_run(dir, a)?;
    let t0 = Instant::now();
    let level = r.level()?;
    let o = PlanOptions {
        max_views: a.num("views", 36)?,
        spacing: a.num("spacing", 4.0)?,
        aerial: a.num("aerial", 0)?,
        roofs: a.num("roofs", 0)?,
        seed: a.num("seed", 7)?,
        threads: r.threads,
        grid: (r.size / 16).clamp(256, 1024),
        ..PlanOptions::default()
    };
    let p = plan(&level, &o)?;
    let doc = json!({
        "points": p.points.iter().map(|v| v.to_array()).collect::<Vec<_>>(),
        "views": p.views.iter().map(spec_json).collect::<Vec<_>>(),
        "predicted": p.predicted, "reachableCells": p.reachable_cells,
        "aerial": o.aerial,
    });
    write_json(&dir.join("plan.json"), &doc)?;
    let kinds = |k: ViewKind| p.views.iter().filter(|v| v.kind == k).count();
    Ok(
        json!({"step": "plan", "seconds": t0.elapsed().as_secs_f32(), "points": p.points.len(), "views": p.views.len(),
        "ground": kinds(ViewKind::Ground), "roof": kinds(ViewKind::Roof), "aerial": kinds(ViewKind::Aerial),
        "predicted": p.predicted.last().copied().unwrap_or(0.0)}),
    )
}

/// Whether the run paints roofs in passes of their own.
fn roofs_by_pass(views: &[Value]) -> bool {
    views.iter().any(|v| v["kind"] == "roof")
}

fn prepare_step(dir: &Path, a: &Args) -> Result<Value, String> {
    let r = open_run(dir, a)?;
    let t0 = Instant::now();
    let (k, spec, vdir) = r.view(a)?;
    let level = r.level()?;
    let texels = r.texels()?;
    let paint = r.paint(&texels)?;
    let g = GBuffer::cast(&level, spec, r.view_size, r.threads);
    let t_cast = t0.elapsed().as_secs_f32();
    let light = ClayLight {
        threads: r.threads,
        ..ClayLight::default()
    };
    let c = clay(&level, &g, &light);
    let t_clay = t0.elapsed().as_secs_f32() - t_cast;
    let start = paint.coverage() == 0.0;
    let p = prepare(
        &level,
        &texels,
        &paint,
        &g,
        &c,
        r.ratio,
        start,
        roofs_by_pass(&r.views()?),
    );
    let s = r.view_size;
    save_rgb(&vdir.join("clay.png"), s, &c)?;
    save_rgba(&vdir.join("state.png"), s, &p.state)?;
    save_mask(&vdir.join("edit.png"), s, &p.edit)?;
    // masks.png: R geometry, G known, B keep (all 255 when the view keeps everything).
    let masks: Vec<[u8; 3]> = (0..p.edit.len())
        .map(|i| {
            let keep = p.keep.as_ref().map_or(1, |k| k[i]);
            [
                p.masks.geometry[i] * 255,
                p.masks.known[i] * 255,
                keep * 255,
            ]
        })
        .collect();
    save_rgb(&vdir.join("masks.png"), s, &masks)?;
    let out = json!({
        "step": "prepare", "view": k, "kind": spec.kind.name(), "mode": p.mode, "skip": p.skip,
        "unknownFraction": p.unknown_fraction, "parts": p.description.parts,
        "roofShare": p.description.roof_share, "floorShare": p.description.floor_share,
        "state": vdir.join("state.png"), "edit": vdir.join("edit.png"),
        "seconds": t0.elapsed().as_secs_f32(), "castSeconds": t_cast, "claySeconds": t_clay,
    });
    write_json(&vdir.join("prepare.json"), &out)?;
    Ok(out)
}

fn read_rgba(path: &Path, size: u32) -> Result<Vec<[u8; 4]>, String> {
    let img = image::open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .to_rgba8();
    let img = if img.width() != size || img.height() != size {
        image::imageops::resize(&img, size, size, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    Ok(img.pixels().map(|p| p.0).collect())
}

fn project_step(dir: &Path, a: &Args) -> Result<Value, String> {
    let r = open_run(dir, a)?;
    let t0 = Instant::now();
    let (k, spec, vdir) = r.view(a)?;
    let painted_path = PathBuf::from(a.get("image").ok_or("--image <painted.png> is required")?);
    let s = r.view_size;
    let painted = read_rgba(&painted_path, s)?;
    let state = read_rgba(&vdir.join("state.png"), s)
        .map_err(|e| format!("{e} (run prepare --view {k} first)"))?;
    let m = read_rgba(&vdir.join("masks.png"), s)?;
    let masks = ViewMasks {
        geometry: m.iter().map(|p| (p[0] > 127) as u8).collect(),
        known: m.iter().map(|p| (p[1] > 127) as u8).collect(),
    };
    let keep: Vec<u8> = m.iter().map(|p| (p[2] > 127) as u8).collect();
    let fin = finish(
        &state,
        &painted,
        &masks,
        Some(&keep),
        s as usize,
        s as usize,
    );
    save_rgba(&vdir.join("comp.png"), s, &fin.image)?;
    if painted_path != vdir.join("painted.png") {
        let _ = std::fs::copy(&painted_path, vdir.join("painted.png"));
    }
    let level = r.level()?;
    let texels = r.texels()?;
    let mut paint = r.paint(&texels)?;
    let g = GBuffer::cast(&level, spec, s, r.threads);
    let edge = silhouette(&g);
    let t1 = Instant::now();
    let pr = project(
        &texels, &mut paint, &g, &edge, &fin.image, r.ratio, r.threads,
    );
    let t_project = t1.elapsed().as_secs_f32();
    r.save_paint(&paint)?;
    let out = json!({
        "step": "project", "view": k, "seen": pr.seen, "written": pr.written, "left": fin.left,
        "coverage": paint.coverage(), "projectSeconds": t_project, "seconds": t0.elapsed().as_secs_f32(),
    });
    write_json(&vdir.join("project.json"), &out)?;
    Ok(out)
}

fn holes_step(dir: &Path, a: &Args) -> Result<Value, String> {
    let r = open_run(dir, a)?;
    let t0 = Instant::now();
    let mut doc = read_json(&dir.join("plan.json"))?;
    let level = r.level()?;
    let texels = r.texels()?;
    let paint = r.paint(&texels)?;
    let points: Vec<Vec3> = doc["points"]
        .as_array()
        .map(|p| {
            p.iter()
                .filter_map(|v| serde_json::from_value::<[f32; 3]>(v.clone()).ok())
                .map(Vec3::from)
                .collect()
        })
        .unwrap_or_default();
    let ring = doc["aerial"].as_u64().unwrap_or(0) as usize;
    let used: Vec<ViewSpec> = doc["views"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| spec_from(v).ok())
        .collect();
    let probe = 128u32;
    let mut best: Option<(usize, ViewSpec)> = None;
    for c in hole_candidates(&level, &points, ring) {
        if used
            .iter()
            .any(|u| u.position == c.position && u.yaw == c.yaw && u.pitch == c.pitch)
        {
            continue;
        }
        let g = GBuffer::cast(&level, c, probe, r.threads);
        let n = unpainted_pixels(&texels, &paint, &g);
        if best.is_none_or(|b| n > b.0) {
            best = Some((n, c));
        }
    }
    let min = (0.004 * (probe * probe) as f32) as usize;
    let Some((n, spec)) = best.filter(|b| b.0 >= min) else {
        return Ok(json!({"step": "holes", "view": null, "seconds": t0.elapsed().as_secs_f32()}));
    };
    let views = doc["views"]
        .as_array_mut()
        .ok_or("plan.json without views")?;
    views.push(spec_json(&spec));
    let k = views.len() - 1;
    write_json(&dir.join("plan.json"), &doc)?;
    Ok(
        json!({"step": "holes", "view": k, "unpaintedProbePixels": n, "seconds": t0.elapsed().as_secs_f32()}),
    )
}

fn export_step(dir: &Path, a: &Args) -> Result<Value, String> {
    let r = open_run(dir, a)?;
    let texels = r.texels()?;
    let paint = r.paint(&texels)?;
    let rings: u32 = a.num("padding", 8)?;
    let img = padded(&texels, &paint, rings, [128, 128, 128]);
    let tex = dir.join("texture.png");
    save_rgb(&tex, r.size, &img)?;
    let out = a
        .get("out")
        .map(PathBuf::from)
        .unwrap_or_else(|| dir.join("textured.glb"));
    let png = std::fs::read(&tex).map_err(|e| format!("texture.png: {e}"))?;
    let glb = std::fs::read(dir.join("level.glb")).map_err(|e| format!("level.glb: {e}"))?;
    std::fs::write(&out, with_texture(&glb, &png)?)
        .map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(json!({"step": "export", "texture": tex, "glb": out, "coverage": paint.coverage()}))
}

/// The binary glTF with `png` appended to its buffer as every material's
/// base colour texture (TEXCOORD_0), base colour factor white. A level
/// without materials gets one.
pub fn with_texture(glb: &[u8], png: &[u8]) -> Result<Vec<u8>, String> {
    let u32_at = |o: usize| {
        glb.get(o..o + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    };
    if glb.get(0..4) != Some(b"glTF") {
        return Err("level.glb is not a binary glTF".into());
    }
    let json_len = u32_at(12).ok_or("truncated glb")? as usize;
    let mut doc: Value = serde_json::from_slice(glb.get(20..20 + json_len).ok_or("truncated glb")?)
        .map_err(|e| format!("glb JSON: {e}"))?;
    let mut bin = Vec::new();
    let next = 20 + json_len;
    if let (Some(len), Some(kind)) = (u32_at(next), u32_at(next + 4))
        && kind == 0x004E_4942
    {
        bin = glb
            .get(next + 8..next + 8 + len as usize)
            .ok_or("truncated glb")?
            .to_vec();
    }
    if doc["buffers"].as_array().is_none_or(|b| b.is_empty()) {
        doc["buffers"] = json!([{"byteLength": 0}]);
    }
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let offset = bin.len();
    bin.extend_from_slice(png);
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    doc["buffers"][0]["byteLength"] = json!(bin.len());
    let push = |doc: &mut Value, key: &str, v: Value| -> usize {
        if !doc[key].is_array() {
            doc[key] = json!([]);
        }
        let arr = doc[key].as_array_mut().unwrap_or_else(|| unreachable!());
        arr.push(v);
        arr.len() - 1
    };
    let view = push(
        &mut doc,
        "bufferViews",
        json!({"buffer": 0, "byteOffset": offset, "byteLength": png.len()}),
    );
    let image = push(
        &mut doc,
        "images",
        json!({"bufferView": view, "mimeType": "image/png", "name": "projection"}),
    );
    let sampler = push(
        &mut doc,
        "samplers",
        json!({"magFilter": 9729, "minFilter": 9987, "wrapS": 33071, "wrapT": 33071}),
    );
    let texture = push(
        &mut doc,
        "textures",
        json!({"source": image, "sampler": sampler}),
    );
    if doc["materials"].as_array().is_none_or(|m| m.is_empty()) {
        doc["materials"] = json!([{"name": "level"}]);
        if let Some(meshes) = doc["meshes"].as_array_mut() {
            for m in meshes {
                for p in m["primitives"].as_array_mut().into_iter().flatten() {
                    p["material"] = json!(0);
                }
            }
        }
    }
    for m in doc["materials"].as_array_mut().into_iter().flatten() {
        if !m["pbrMetallicRoughness"].is_object() {
            m["pbrMetallicRoughness"] = json!({});
        }
        let pbr = &mut m["pbrMetallicRoughness"];
        pbr["baseColorTexture"] = json!({"index": texture, "texCoord": 0});
        pbr["baseColorFactor"] = json!([1.0, 1.0, 1.0, 1.0]);
        pbr["metallicFactor"] = json!(0.0);
        if pbr["roughnessFactor"].is_null() {
            pbr["roughnessFactor"] = json!(0.9);
        }
    }
    let text = serde_json::to_vec(&doc).map_err(|e| e.to_string())?;
    Ok(glb_container(&text, &bin))
}
