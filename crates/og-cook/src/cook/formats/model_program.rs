//! Native model-kernel replay and material overrides. Checked TS declarations
//! supply the cook input; standalone modelling tools may read kernel JSON.

use std::path::{Path, PathBuf};

use og_shape::model::Model;
use og_shape::model::program::{self, Params};
use og_shape::tools::{ColorSpace, Material, Texture};

/// Whether a source path is a program file.
pub fn is_program(rel: &str) -> bool {
    rel.to_ascii_lowercase().ends_with(".shape.json")
}

/// One material block's fields; what it leaves out is the program's.
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
    /// Captured kernel font/image bytes, keyed by their explicit input name.
    pub inputs: std::collections::BTreeMap<String, (String, Vec<u8>)>,
    /// Checked variant descriptor retained by authoring tools when baking maps.
    pub authoring: Option<serde_json::Value>,
}

/// Texels between an atlas's charts, enough for the bake's gutter.
pub const ATLAS_PADDING: u32 = 4;

/// Model tools read the cook's checked declaration and captured maps, not an
/// adjacent program guessed from the declaration's filename.
pub fn read(root: &Path, file: &str) -> Result<(Program, Read), String> {
    let project = crate::cook::source_project::Project::open(root).map_err(|e| e.to_string())?;
    let index =
        crate::cook::source_index::Index::open(root, &project).map_err(|e| e.to_string())?;
    let id = index
        .files
        .get(file)
        .ok_or_else(|| format!("{file}: not a content declaration"))?;
    let mut sources = Vec::new();
    crate::cook::pipeline::author::models(root, &project, &mut sources, &mut Default::default())
        .map_err(|e| e.to_string())?;
    let prepared = sources
        .iter()
        .find(|s| &s.rel == id)
        .and_then(|s| s.authored_model.as_ref())
        .ok_or_else(|| format!("{file}: not a model declaration"))?;
    let program = prepared
        .program
        .as_ref()
        .ok_or_else(|| format!("{file}: not a program model"))?;
    Ok((
        program.clone(),
        prepared
            .images
            .iter()
            .map(|(p, b)| (p.clone(), b.clone()))
            .collect(),
    ))
}

/// Reads standalone kernel JSON for modelling tools, not a cook author entry.
pub fn parse(
    rel: &str,
    bytes: &[u8],
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Program, String> {
    let utf8 = |b: Vec<u8>, what: &str| {
        String::from_utf8(b).map_err(|_| format!("{what}: not UTF-8 text"))
    };
    let file = Path::new(rel)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(rel)
        .to_string();
    let mut program = Program {
        file,
        text: utf8(bytes.to_vec(), rel)?,
        ..Default::default()
    };
    program.capture(read)?;
    Ok(program)
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
    /// Capture native kernel inputs once; replay and budget passes never reopen them.
    pub fn capture(
        &mut self,
        read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
    ) -> Result<(), String> {
        let text = program::with_params(&self.text, &self.params).map_err(|e| e.to_string())?;
        let base = Path::new(&self.file).parent().unwrap_or(Path::new(""));
        for name in program::fonts_named(&text)
            .map_err(|e| e.to_string())?
            .into_iter()
            .chain(program::textures_named(&text).map_err(|e| e.to_string())?)
        {
            let path = join(base, &name);
            let bytes = read(&path)?;
            self.inputs.insert(name, (path, bytes));
        }
        Ok(())
    }

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
    pub fn model(&self, uvs: bool) -> Result<(Model, Read), String> {
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
        let mut files = vec![(self.file.clone(), self.text.clone().into_bytes())];
        let mut named = |names: Vec<String>| -> Result<program::Fonts, String> {
            let mut out = program::Fonts::new();
            for name in names {
                let (path, bytes) = self.inputs.get(&name).ok_or_else(|| {
                    format!("{}: {name}: not a captured program input", self.file)
                })?;
                files.push((path.clone(), bytes.clone()));
                out.insert(name, bytes.clone());
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
        let (model, mut files) = self.model(atlas > 0)?;
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
