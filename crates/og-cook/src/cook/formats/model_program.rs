//! A model whose source is an og-shape program: the program file itself
//! (`chair.shape.json`), or a `type model` text naming one with values
//! for its parameters and material blocks over its materials (a variant):
//!
//! ```text
//! type model
//! version 1
//! ---
//! program "chair.shape.json"     # beside this file
//! params {
//!   height 1100                  # the program's "$height"
//! }
//! material "seat" {              # over the program's material "seat"
//!   color 1 1 1                  # linear, as glTF's baseColorFactor
//!   map "chair-tall.seat.basecolor.png"
//! }
//! ```
//!
//! The cook replays the program with the kernel ([`og_shape::model`]),
//! which is deterministic: the same program and values give the same
//! bytes on every machine. The model is then a glTF binary, and the rest
//! of the cook treats it as any `.glb`. A material block with a `map`
//! replays the program with UVs on every part (its first operation
//! `uvs`), as `og model bake` does when it paints the map.
//!
//! Entry: [`parse`] reads the source; [`Program::model`] replays it and
//! [`Program::glb`] writes it.

use std::path::{Path, PathBuf};

use og_core::ogt::{Block, Document, Entry};
use og_shape::model::Model;
use og_shape::model::program::{self, Params};
use og_shape::tools::{ColorSpace, Material, Texture};

/// Whether a source path is a program file.
pub fn is_program(rel: &str) -> bool {
    rel.to_ascii_lowercase().ends_with(".shape.json")
}

/// Whether a `type model` text names a program.
pub fn names_program(doc: &Document) -> bool {
    doc.body.get("program").is_some()
}

/// One material block's fields; what it leaves out is the program's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MaterialOver {
    /// The program material's name (a part's name when it has none).
    pub name: String,
    /// Linear RGB.
    pub color: Option<[f64; 3]>,
    pub metalness: Option<f64>,
    pub roughness: Option<f64>,
    pub opacity: Option<f64>,
    /// Linear RGB.
    pub emissive: Option<[f64; 3]>,
    pub emissive_intensity: Option<f64>,
    /// Image files beside the text.
    pub map: Option<String>,
    pub normal_map: Option<String>,
    pub normal_scale: Option<f64>,
    pub roughness_map: Option<String>,
    pub emissive_map: Option<String>,
    pub ao_map: Option<String>,
    pub ao_map_intensity: Option<f64>,
    pub clearcoat: Option<f64>,
    pub clearcoat_roughness: Option<f64>,
    /// Linear RGB sheen colour.
    pub sheen: Option<[f64; 3]>,
    pub sheen_roughness: Option<f64>,
    pub transmission: Option<f64>,
    pub iridescence: Option<f64>,
    pub alpha_test: Option<f64>,
    pub transparent: Option<bool>,
    pub double_sided: Option<bool>,
}

/// A program source, read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {
    /// The program file, relative to the source's folder.
    pub file: String,
    /// The program's text as written.
    pub text: String,
    /// The parameters the text sets.
    pub params: Params,
    /// The material blocks, in order.
    pub materials: Vec<MaterialOver>,
    /// `atlas <px>`: every part's UVs packed into one atlas on this grid
    /// (what `og model bake` paints its maps on); 0 is none.
    pub atlas: u32,
}

/// Texels between an atlas's charts, enough for the bake's gutter.
pub const ATLAS_PADDING: u32 = 4;

/// The fields a material block takes.
const MATERIAL_FIELDS: [&str; 22] = [
    "color",
    "metalness",
    "roughness",
    "opacity",
    "emissive",
    "emissiveIntensity",
    "map",
    "normalMap",
    "normalScale",
    "roughnessMap",
    "metalnessMap",
    "emissiveMap",
    "aoMap",
    "aoMapIntensity",
    "clearcoat",
    "clearcoatRoughness",
    "sheen",
    "sheenRoughness",
    "transmission",
    "iridescence",
    "alphaTest",
    "doubleSided",
];

fn at(e: &Entry, msg: impl std::fmt::Display) -> String {
    format!("line {}: {} {msg}", e.line, e.key)
}

fn material(e: &Entry) -> Result<MaterialOver, String> {
    let name = e.string(0).map_err(|x| at(e, x))?.to_string();
    let empty = Block::new();
    let b = e.block.as_ref().unwrap_or(&empty);
    let mut m = MaterialOver {
        name,
        ..Default::default()
    };
    for f in &b.entries {
        let num = |i: usize| -> Result<f64, String> {
            f.number(i)
                .map(|n| n.as_f64())
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| at(f, "takes numbers"))
        };
        let unit = |v: f64| -> Result<f64, String> {
            if (0.0..=1.0).contains(&v) {
                Ok(v)
            } else {
                Err(at(f, format!("is 0 to 1, got {v}")))
            }
        };
        let non_neg = |v: f64| -> Result<f64, String> {
            if v >= 0.0 {
                Ok(v)
            } else {
                Err(at(f, format!("cannot be negative, got {v}")))
            }
        };
        let rgb = || -> Result<[f64; 3], String> {
            if f.values.len() != 3 {
                return Err(at(f, "takes r g b, linear 0 to 1"));
            }
            Ok([num(0)?, num(1)?, num(2)?])
        };
        let rel_image = || -> Result<String, String> {
            let file = f.string(0).map_err(|x| at(f, x))?;
            if file.is_empty() || file.starts_with('/') || file.contains('\\') {
                return Err(at(f, "names an image file beside the model, in quotes"));
            }
            Ok(file.to_string())
        };
        let flag = || -> Result<bool, String> {
            f.values
                .first()
                .and_then(|v| v.as_bool())
                .ok_or_else(|| at(f, "takes true or false"))
        };
        match f.key.as_str() {
            "color" => m.color = Some(rgb()?),
            "emissive" => m.emissive = Some(rgb()?),
            "emissiveIntensity" | "emissive_intensity" => {
                m.emissive_intensity = Some(non_neg(num(0)?)?)
            }
            "metalness" => m.metalness = Some(unit(num(0)?)?),
            "roughness" => m.roughness = Some(unit(num(0)?)?),
            "opacity" => m.opacity = Some(unit(num(0)?)?),
            "map" => m.map = Some(rel_image()?),
            "normalMap" | "normal_map" => m.normal_map = Some(rel_image()?),
            "normalScale" | "normal_scale" => m.normal_scale = Some(non_neg(num(0)?)?),
            "roughnessMap" | "roughness_map" | "metalnessMap" | "metalness_map" => {
                m.roughness_map = Some(rel_image()?)
            }
            "emissiveMap" | "emissive_map" => m.emissive_map = Some(rel_image()?),
            "aoMap" | "ao_map" => m.ao_map = Some(rel_image()?),
            "aoMapIntensity" | "ao_map_intensity" => m.ao_map_intensity = Some(unit(num(0)?)?),
            "clearcoat" => m.clearcoat = Some(unit(num(0)?)?),
            "clearcoatRoughness" | "clearcoat_roughness" => {
                m.clearcoat_roughness = Some(unit(num(0)?)?)
            }
            "sheen" | "sheenColor" | "sheen_color" => m.sheen = Some(rgb()?),
            "sheenRoughness" | "sheen_roughness" => m.sheen_roughness = Some(unit(num(0)?)?),
            "transmission" => m.transmission = Some(unit(num(0)?)?),
            "iridescence" => m.iridescence = Some(unit(num(0)?)?),
            "alphaTest" | "alpha_test" => m.alpha_test = Some(unit(num(0)?)?),
            "transparent" => m.transparent = Some(flag()?),
            "doubleSided" | "double_sided" => m.double_sided = Some(flag()?),
            other => {
                return Err(at(
                    f,
                    format!(
                        "is not a field of a program's material block ({other}); it takes {}",
                        MATERIAL_FIELDS.join(", ")
                    ),
                ));
            }
        }
    }
    Ok(m)
}

/// Reads a program source: `rel` names it (a `.shape.json`, or an `.ogt`),
/// `bytes` are its own, and `read` gives a file by its path relative to
/// the source's folder (for a variant's program).
pub fn parse(
    rel: &str,
    bytes: &[u8],
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Program, String> {
    let utf8 = |b: Vec<u8>, what: &str| {
        String::from_utf8(b).map_err(|_| format!("{what}: not UTF-8 text"))
    };
    if is_program(rel) {
        let file = Path::new(rel)
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or(rel)
            .to_string();
        return Ok(Program {
            file,
            text: utf8(bytes.to_vec(), rel)?,
            ..Default::default()
        });
    }
    let doc = Document::parse(&utf8(bytes.to_vec(), rel)?).map_err(|e| e.to_string())?;
    let mut p = Program::default();
    for e in &doc.body.entries {
        match e.key.as_str() {
            "program" => {
                let file = e.string(0).map_err(|x| at(e, x))?;
                if !is_program(file) || file.starts_with('/') || file.contains('\\') {
                    return Err(at(
                        e,
                        "names a program file beside the model, in quotes (\"chair.shape.json\")",
                    ));
                }
                p.file = file.to_string();
            }
            "params" => {
                for f in e.block.iter().flat_map(|b| &b.entries) {
                    let v = f
                        .number(0)
                        .map(|n| n.as_f64())
                        .ok()
                        .filter(|v| v.is_finite() && f.values.len() == 1)
                        .ok_or_else(|| at(f, "takes one number, the parameter's value"))?;
                    p.params.insert(f.key.clone(), v);
                }
            }
            "material" => p.materials.push(material(e)?),
            "atlas" => {
                p.atlas = e
                    .number(0)
                    .ok()
                    .and_then(|n| n.as_u64().ok())
                    .filter(|&n| (16..=8192).contains(&n))
                    .ok_or_else(|| at(e, "takes the atlas's texels across, 16 to 8192"))?
                    as u32;
            }
            // Read by the cook's conform step.
            "conform" | "process" => {}
            other => {
                return Err(at(
                    e,
                    format!(
                        "is not an entry of a program model ({other}); it takes program, params, material, atlas and conform"
                    ),
                ));
            }
        }
    }
    if p.file.is_empty() {
        return Err("a program model names its program: program \"chair.shape.json\"".into());
    }
    p.text = utf8(
        read(&p.file).map_err(|e| format!("{}: {e}", p.file))?,
        &p.file,
    )?;
    Ok(p)
}

/// Files read beside a source: (path relative to its folder, bytes).
pub type Read = Vec<(String, Vec<u8>)>;

/// What [`Program::glb`] wrote.
pub struct Written {
    /// The glTF binary.
    pub glb: Vec<u8>,
    /// Every file read beside the source (the program, its fonts and
    /// textures, the material blocks' maps), relative to the source's
    /// folder: part of its hash, consumed with it.
    pub read: Read,
}

impl Program {
    /// Whether a material block paints any texture map, so every part needs UVs.
    pub fn wants_uvs(&self) -> bool {
        self.materials.iter().any(|m| {
            m.map.is_some()
                || m.normal_map.is_some()
                || m.roughness_map.is_some()
                || m.emissive_map.is_some()
                || m.ao_map.is_some()
        })
    }

    /// The program replayed at its parameters, with UVs on every part when
    /// `uvs` or [`Program::wants_uvs`]; the files it read.
    pub fn model(
        &self,
        uvs: bool,
        read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    ) -> Result<(Model, Read), String> {
        let fail = |e: og_shape::model::ModelError| format!("{}: {e}", self.file);
        let mut text = program::with_params(&self.text, &self.params).map_err(fail)?;
        if uvs || self.wants_uvs() {
            let mut v: serde_json::Value =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", self.file))?;
            if let Some(ops) = v.get_mut("ops").and_then(serde_json::Value::as_array_mut) {
                ops.insert(0, serde_json::json!({ "op": "uvs" }));
            }
            text = v.to_string();
        }
        // The program's fonts and maps are beside the program.
        let base = Path::new(&self.file)
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let rel = |name: &str| -> String { join(&base, name) };
        let mut files = vec![(self.file.clone(), self.text.clone().into_bytes())];
        let mut named = |names: Vec<String>| -> Result<program::Fonts, String> {
            let mut out = program::Fonts::new();
            for name in names {
                let bytes = read(&rel(&name)).map_err(|e| format!("{}: {name}: {e}", self.file))?;
                files.push((rel(&name), bytes.clone()));
                out.insert(name, bytes);
            }
            Ok(out)
        };
        let fonts = named(program::fonts_named(&text).map_err(fail)?)?;
        let textures = named(program::textures_named(&text).map_err(fail)?)?;
        let model = program::run_files(&text, &fonts, &textures).map_err(fail)?;
        Ok((model, files))
    }

    /// The model as a glTF binary: [`Program::scene`] at its own atlas.
    pub fn glb(&self, read: &dyn Fn(&str) -> Result<Vec<u8>, String>) -> Result<Written, String> {
        let file = self.file.clone();
        let (glb, files) = self.scene(self.atlas, read, |scene, model| {
            og_shape_formats::gltf::write::glb_clips(scene, &model.scene_clips())
                .map_err(|e| format!("{file}: {e}"))
        })?;
        Ok(Written { glb, read: files })
    }

    /// Runs `f` on the model's scene: [`Program::model`]'s, with every
    /// part's UVs packed into one atlas of `atlas` texels when above 0
    /// (`og_shape_formats::pack`; materials alike then share the first's
    /// name), and the material blocks over the program's materials. Also
    /// the files read.
    pub fn scene<T>(
        &self,
        atlas: u32,
        read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
        f: impl FnOnce(&og_shape::tools::Scene, &Model) -> Result<T, String>,
    ) -> Result<(T, Read), String> {
        let (model, mut files) = self.model(atlas > 0, read)?;
        let built = model.build().map_err(|e| format!("{}: {e}", self.file))?;
        let mut scene = built.scene();
        let packed = if atlas > 0 {
            Some(
                og_shape_formats::pack(
                    &scene,
                    &og_shape::tools::Atlas {
                        size: atlas,
                        padding: ATLAS_PADDING,
                        no_turns: false,
                    },
                )
                .map_err(|e| format!("{}: atlas: {e}", self.file))?,
            )
        } else {
            None
        };
        if let Some(p) = &packed {
            p.apply(&mut scene);
        }
        // Materials with a procedural surface: every part painted on a
        // fresh atlas (`og_shape::tools::paint`).
        let painted = og_shape::tools::paint_scene(
            &scene,
            &og_shape::tools::Paint::default(),
            &og_shape_formats::texture::decode::CODEC,
        )
        .map_err(|e| format!("{}: {e}", self.file))?;
        if let Some(p) = &painted {
            p.apply(&mut scene);
        }
        for m in &self.materials {
            let key = |p: &og_shape::tools::Part| {
                if p.material.name.is_empty() {
                    p.name.clone()
                } else {
                    p.material.name.clone()
                }
            };
            if !scene.parts.iter().any(|p| key(p) == m.name) {
                let mut names: Vec<String> = scene.parts.iter().map(key).collect();
                names.dedup();
                return Err(format!(
                    "material {:?}: the program has no such material (it has {names:?})",
                    m.name
                ));
            }
            let mut load_tex = |slot: &Option<String>| -> Result<Option<Texture>, String> {
                match slot {
                    Some(path) => {
                        let bytes = read(path)
                            .map_err(|e| format!("material {:?}: {path}: {e}", m.name))?;
                        files.push((path.clone(), bytes.clone()));
                        Ok(Some(Texture::from_bytes(path, bytes)))
                    }
                    None => Ok(None),
                }
            };
            let maps = LoadedMaps {
                map: load_tex(&m.map)?,
                normal: load_tex(&m.normal_map)?,
                roughness: load_tex(&m.roughness_map)?,
                emissive: load_tex(&m.emissive_map)?,
                ao: load_tex(&m.ao_map)?,
            };
            for p in scene.parts.iter_mut().filter(|p| key(p) == m.name) {
                over(&mut p.material, m, &maps);
            }
        }
        Ok((f(&scene, &model)?, files))
    }
}

/// `base/name`, with `/` (a path relative to the source's folder).
fn join(base: &Path, name: &str) -> String {
    let p: PathBuf = base.join(name);
    p.to_string_lossy().replace('\\', "/")
}

/// sRGB to linear, as glTF reads colours.
fn linear(c: f64) -> f64 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        libm::pow((c + 0.055) / 1.055, 2.4)
    }
}

struct LoadedMaps {
    map: Option<Texture>,
    normal: Option<Texture>,
    roughness: Option<Texture>,
    emissive: Option<Texture>,
    ao: Option<Texture>,
}

/// `m` over `material`: its colours linear from here on.
fn over(material: &mut Material, m: &MaterialOver, maps: &LoadedMaps) {
    if material.color_space == ColorSpace::Srgb {
        material.color = material.color.map(|c| c.map(linear));
        material.emissive = material.emissive.map(linear);
        material.sheen = material.sheen.map(linear);
        material.color_space = ColorSpace::Linear;
    }
    if let Some(c) = m.color {
        material.color = Some(c);
    }
    if let Some(e) = m.emissive {
        material.emissive = e;
    }
    if let Some(v) = m.emissive_intensity {
        material.emissive_intensity = Some(v);
    }
    if let Some(v) = m.metalness {
        material.metalness = v;
    }
    if let Some(v) = m.roughness {
        material.roughness = Some(v);
    }
    if let Some(v) = m.opacity {
        material.opacity = Some(v);
    }
    if let Some(t) = &maps.map {
        material.map = Some(t.clone());
    }
    if let Some(t) = &maps.normal {
        material.normal_map = Some(t.clone());
    }
    if let Some(v) = m.normal_scale {
        material.normal_scale = Some(v);
    }
    if let Some(t) = &maps.roughness {
        material.metalness_map = Some(t.clone());
    }
    if let Some(t) = &maps.emissive {
        material.emissive_map = Some(t.clone());
    }
    if let Some(t) = &maps.ao {
        material.ao_map = Some(t.clone());
    }
    if let Some(v) = m.ao_map_intensity {
        material.ao_map_intensity = Some(v);
    }
    if let Some(v) = m.clearcoat {
        material.clearcoat = v;
    }
    if let Some(v) = m.clearcoat_roughness {
        material.clearcoat_roughness = v;
    }
    if let Some(c) = m.sheen {
        material.sheen = c;
    }
    if let Some(v) = m.sheen_roughness {
        material.sheen_roughness = v;
    }
    if let Some(v) = m.transmission {
        material.transmission = v;
    }
    if let Some(v) = m.iridescence {
        material.iridescence = v;
    }
    if let Some(v) = m.alpha_test {
        material.alpha_test = Some(v);
    }
    if let Some(v) = m.transparent {
        material.transparent = Some(v);
    }
    if let Some(v) = m.double_sided {
        material.double_sided = v;
    }
}
