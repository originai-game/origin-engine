use super::*;

// ── the contract ────────────────────────────────────────────────────────────

/// The art contract of the project's look, as the check reads it from the
/// cooked look (flattened by the cook) and the palette it names.
#[derive(Clone, Debug, Default)]
pub struct Contract {
    /// The look's asset id (project.ogt's `look`).
    pub look: String,
    /// The look's chain of bases, nearest first (a preset's slug last).
    pub extends: Vec<String>,
    /// The art resources the look sets.
    pub sets: Vec<String>,
    /// `ArtPalette.colors`: its id and sRGB colours.
    pub palette: Option<(String, Vec<[u8; 3]>)>,
    pub strict: bool,
    pub max_colors: u32,
    /// `ArtPixels` sets a grid (`unit`, `logicalHeight` or `spriteSizes`).
    pub pixels: bool,
    /// `ArtLine.width`.
    pub line: f64,
    /// `ArtLine.color` in sRGB (the IDL's default when the look sets none).
    pub line_rgb: [u8; 3],
    pub prop_faces: u32,
    pub character_faces: u32,
    /// `ArtShape.normals`: `any`, `smooth`, `flat` or `angle`.
    pub normals: String,
    pub density: f64,
    pub max_size: u32,
    /// `ArtTexels.filter`: `any`, `nearest` or `linear`.
    pub filter: String,
    /// `ArtTexels.mips`: `any`, `full` or `none`.
    pub mips: String,
    /// Every art block of the cooked (flattened) look as JSON, field by
    /// field: numbers, booleans, words, strings, lists of numbers, and a
    /// colour as sRGB `#rrggbb` (the terms the frames were judged against).
    pub art: Value,
    /// `ArtCamera.projection`: `any`, `perspective` or `orthographic`.
    pub projection: String,
    /// `ArtCamera.fixedAngle`, `pitch` and `yaw` (degrees).
    pub fixed_angle: bool,
    pub pitch: f64,
    pub yaw: f64,
    /// `ArtCamera.fovMin` and `fovMax` (degrees; 0 no limit).
    pub fov_min: f64,
    pub fov_max: f64,
    /// `ArtMotion.stepFps` (0 smooth) and `maxFrames` (0 no limit).
    pub step_fps: f64,
    pub max_frames: u32,
}

fn field<'a>(look: &'a LookFile, resource: &str, name: &str) -> Option<&'a og_core::ogt::Value> {
    look.blocks
        .iter()
        .find(|b| b.resource == resource)?
        .get(name)?
        .first()
}

fn num(look: &LookFile, resource: &str, name: &str) -> f64 {
    field(look, resource, name)
        .and_then(|v| v.as_number())
        .map_or(0.0, |n| n.as_f64())
}

fn word(look: &LookFile, resource: &str, name: &str, zero: &str) -> String {
    field(look, resource, name)
        .and_then(|v| v.as_word())
        .unwrap_or(zero)
        .to_string()
}

/// `ArtLine.color` (linear numbers in the cooked look) in sRGB.
fn line_colour(look: &LookFile) -> [u8; 3] {
    let v = look
        .blocks
        .iter()
        .find(|b| b.resource == "ArtLine")
        .and_then(|b| b.get("color"));
    match v {
        Some([r, g, b]) => {
            let c = |x: &og_core::ogt::Value| x.as_number().map_or(0.0, |n| n.as_f32());
            [srgb8(c(r)), srgb8(c(g)), srgb8(c(b))]
        }
        _ => LINE_RGB,
    }
}

/// One look value as JSON: a number, `true`/`false`, a word or a string.
fn value_json(v: &og_core::ogt::Value) -> Value {
    match v {
        og_core::ogt::Value::Number(n) => json!(n.as_f64()),
        og_core::ogt::Value::String(s) => json!(s),
        og_core::ogt::Value::Word(w) => match w.as_str() {
            "true" => json!(true),
            "false" => json!(false),
            w => json!(w),
        },
    }
}

/// The look's art blocks as JSON (`{ "ArtLine": { "width": 2, "color":
/// "#1a1a2e" }, ... }`), in the order og_cook's ART_RESOURCES lists them.
fn art_blocks(look: &LookFile) -> Value {
    let mut out = serde_json::Map::new();
    for r in og_cook::look::ART_RESOURCES {
        let Some(b) = look.blocks.iter().find(|b| b.resource == *r) else {
            continue;
        };
        let mut fields = serde_json::Map::new();
        for (name, values, _) in &b.fields {
            let numbers: Option<Vec<f32>> = values
                .iter()
                .map(|v| v.as_number().map(|n| n.as_f32()))
                .collect();
            let v = match (name.as_str(), values.as_slice(), numbers) {
                // A colour is cooked linear: written back as the sRGB a look writes.
                ("color", [_, _, _, ..], Some(n)) => json!(format!(
                    "#{:02x}{:02x}{:02x}",
                    srgb8(n[0]),
                    srgb8(n[1]),
                    srgb8(n[2])
                )),
                (_, [one], _) => value_json(one),
                (_, many, _) => json!(many.iter().map(value_json).collect::<Vec<_>>()),
            };
            fields.insert(name.clone(), v);
        }
        out.insert(r.to_string(), Value::Object(fields));
    }
    Value::Object(out)
}

/// Reads a palette asset's sRGB colours by id.
type PaletteOf<'a> = dyn Fn(&str) -> Result<Vec<[u8; 3]>, String> + 'a;

impl Contract {
    /// The contract of `look` (a cooked, flattened look), its palette read
    /// through `palette` (an id to its colours).
    pub fn of(id: &str, look: &LookFile, palette: &PaletteOf) -> Result<Contract, String> {
        let colors = field(look, "ArtPalette", "colors").and_then(|v| v.as_str());
        let palette = match colors {
            Some(name) if !name.is_empty() => {
                let id = og_cook::cook::look::builtin(name).map_or_else(
                    || og_cook::runtime::asset_id(name).to_string(),
                    |(id, _)| id.to_string(),
                );
                Some((id.clone(), palette(&id)?))
            }
            _ => None,
        };
        let sizes = look
            .blocks
            .iter()
            .find(|b| b.resource == "ArtPixels")
            .and_then(|b| b.get("spriteSizes"))
            .is_some_and(|v| !v.is_empty());
        Ok(Contract {
            look: id.to_string(),
            extends: look.extends.clone(),
            sets: look
                .resources()
                .into_iter()
                .filter(|r| og_cook::look::ART_RESOURCES.contains(r))
                .map(str::to_string)
                .collect(),
            palette,
            strict: field(look, "ArtPalette", "strict").and_then(|v| v.as_bool()) == Some(true),
            max_colors: num(look, "ArtPalette", "maxColors") as u32,
            pixels: num(look, "ArtPixels", "unit") > 0.0
                || num(look, "ArtPixels", "logicalHeight") > 0.0
                || sizes,
            line: num(look, "ArtLine", "width"),
            line_rgb: line_colour(look),
            prop_faces: num(look, "ArtShape", "propFaces") as u32,
            character_faces: num(look, "ArtShape", "characterFaces") as u32,
            normals: word(look, "ArtShape", "normals", "any"),
            density: num(look, "ArtTexels", "density"),
            max_size: num(look, "ArtTexels", "maxSize") as u32,
            filter: word(look, "ArtTexels", "filter", "any"),
            mips: word(look, "ArtTexels", "mips", "any"),
            art: art_blocks(look),
            projection: word(look, "ArtCamera", "projection", "any"),
            fixed_angle: field(look, "ArtCamera", "fixedAngle").and_then(|v| v.as_bool())
                == Some(true),
            pitch: num(look, "ArtCamera", "pitch"),
            yaw: num(look, "ArtCamera", "yaw"),
            fov_min: num(look, "ArtCamera", "fovMin"),
            fov_max: num(look, "ArtCamera", "fovMax"),
            step_fps: num(look, "ArtMotion", "stepFps"),
            max_frames: num(look, "ArtMotion", "maxFrames") as u32,
        })
    }

    /// The catalogue slug the look builds on: the first of its chain (or
    /// its own file's name) that is a catalogue style.
    pub fn slug(&self) -> Option<String> {
        let own = Path::new(&self.look)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string());
        let styles = catalogue_styles();
        own.into_iter()
            .chain(self.extends.iter().cloned())
            .find(|s| styles.iter().any(|e| e["slug"] == s.as_str()))
    }
}

fn catalogue_styles() -> Vec<Value> {
    serde_json::from_str::<Value>(CATALOGUE)
        .ok()
        .and_then(|c| c["styles"].as_array().cloned())
        .unwrap_or_default()
}

/// project.ogt's body, parsed.
pub(super) fn project_body(root: &Path) -> Result<og_core::ogt::Block, String> {
    let path = root.join("project.ogt");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    og_core::ogt::parse(&text)
        .map(|d| d.body)
        .map_err(|e| format!("{}: {}", path.display(), e.reason()))
}

/// Resolve a declaration from project.ogt, then the game's evaluated GameDesc.
/// Importing the bundle records declarations only: no world or start callback.
pub(super) fn declared_look(root: &Path) -> Result<Option<String>, String> {
    let body = project_body(root)?;
    if let Some(id) = og_cook::look::project_look(&body).map_err(|e| format!("project.ogt:{e}"))? {
        return Ok(Some(id));
    }
    let p = crate::project::Project::open(root)?;
    let package = crate::project::engine_package(Some(root)).ok_or(crate::project::NO_ENGINE)?;
    let bundle = crate::project::bundle_native(&p, &package)?;
    let output = bundle.with_file_name("game-look.json");
    let status = crate::project::bun(&package, "src/node/game-look.ts")
        .arg(&bundle)
        .arg(&output)
        .stdout(std::process::Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("could not read GameDesc.look (the error is above)".into());
    }
    let look: String = serde_json::from_slice(&std::fs::read(&output).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    Ok((!look.is_empty()).then(|| og_cook::runtime::asset_id(&look).to_string()))
}

/// The contract of the project at `root`, read from its cooked look.
pub fn contract(root: &Path) -> Result<Option<Contract>, String> {
    declared_look(root)?
        .map(|id| contract_of(root, &id))
        .transpose()
}

pub(super) fn contract_of(root: &Path, id: &str) -> Result<Contract, String> {
    let bundle = Bundle::open(&root.join("cooked")).map_err(|e| e.to_string())?;
    let bytes = bundle.read(id).map_err(|e| e.to_string())?;
    let look = bundle.look(id, &bytes).map_err(|e| e.to_string())?;
    let palette = |pid: &str| -> Result<Vec<[u8; 3]>, String> {
        let bytes = bundle
            .read(pid)
            .map_err(|e| format!("the look's ArtPalette.colors: {e}"))?;
        bundle.palette(pid, &bytes).map_err(|e| e.to_string())
    };
    Contract::of(id, &look, &palette)
}
