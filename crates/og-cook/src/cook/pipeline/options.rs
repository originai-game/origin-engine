use super::*;

/// Mesh processing (meshoptimizer).
#[derive(Clone, Debug, PartialEq)]
pub struct MeshOptions {
    /// LOD levels including the full mesh (LOD 0).
    pub lod_levels: u32,
    /// Each LOD targets this fraction of the previous one's triangles.
    pub lod_ratio: f32,
    /// The largest simplification error allowed, relative to the mesh extent.
    pub lod_max_error: f32,
    /// How much vertex-cache efficiency overdraw ordering may give up (1.05 = 5 %).
    pub overdraw_threshold: f32,
    /// Generate MikkTSpace tangents for normal-mapped primitives that lack them.
    pub tangents: bool,
    /// Each level's fraction of the full mesh's triangles, the first 1: a
    /// model's import settings (`model { lods 1 0.5 0.25 }`). `None`: the
    /// levels `lod_levels` and `lod_ratio` give (1, 0.5, 0.25 by default).
    pub lods: Option<Vec<f32>>,
    /// Cook each primitive's raycast BVH (og.render.raycast) into the
    /// model's blob, so a raycast never builds one at run time. `og cook`
    /// sets it only for a project whose sources name `raycast`; a model's
    /// import settings set it either way (`model { raycast true|false }`).
    /// Without one, the first og.render.raycast that reaches the model
    /// builds it (about 13 bytes a triangle saved).
    pub raycast: bool,
    /// Cut each level into meshlet clusters (the blob's `clusters` and
    /// `cluster_indices`, og-render's `meshlet`), which the meshlet path
    /// draws in a frame culled on the GPU. project.ogt's `cook { clusters
    /// false }` cooks none, for a game whose frames are not culled on the
    /// GPU (`Culling.auto` below `GPU_CULLING_MIN_INSTANCES` instances),
    /// saving their bytes (a garden of 1072 cells: 36 MB of 307 brotli'd);
    /// its models then draw on the per-instance path in every frame.
    pub clusters: bool,
    /// Bake each primitive's triangle-mesh collider shape (rapier's mesh
    /// with its BVH, over LOD 0 as the runtime decodes it, or the proxy's
    /// level for a streamed model) into the blob, so a SHAPE_TRIMESH
    /// collider at any scale naming one of its geometries builds nothing in
    /// the tick that first needs it: `model { collider true }`. Off by
    /// default: it takes about 90 bytes a triangle.
    pub collider: bool,
    pub collider_kind: og_physics_data::trimesh::BakeKind,
    /// The project's art contract as models are held to it (docs/art.md
    /// §2.3, `conform::mesh`): set by the cook from project.ogt's `look`
    /// for every asset that does not opt out; `None` conforms nothing.
    pub conform: Option<conform::mesh::Conform>,
    /// Bake the model's octahedral impostor (`model { impostor true
    /// impostorGrid 8 impostorSize 128 impostorBelow false }`): two
    /// atlases, `<id>#impostor` and `<id>#impostor-normal`, drawn as the
    /// model's last level far away. Off by default.
    pub impostor: Option<model::ImpostorBake>,
}

impl Default for MeshOptions {
    fn default() -> Self {
        MeshOptions {
            lod_levels: 3,
            lod_ratio: 0.5,
            lod_max_error: 0.05,
            overdraw_threshold: 1.05,
            tangents: true,
            lods: None,
            raycast: true,
            clusters: true,
            collider: false,
            collider_kind: og_physics_data::trimesh::BakeKind::Trimesh,
            conform: None,
            impostor: None,
        }
    }
}

impl MeshOptions {
    /// Each level's fraction of the full mesh's triangles, level 0 first (1).
    pub fn fractions(&self) -> Vec<f32> {
        match &self.lods {
            Some(l) => l.clone(),
            None => (0..self.lod_levels.max(1))
                .map(|k| self.lod_ratio.powi(k as i32))
                .collect(),
        }
    }

    /// These options with a model's import settings (`model { lods 1 0.5
    /// 0.25 lodError 0.05 }`) over them: `lods` the levels' triangle
    /// fractions (the first 1, each below the one before; `lods 1` keeps the
    /// full mesh alone), `lodError` the largest simplification error
    /// relative to the mesh's extent.
    pub fn for_model(&self, block: Option<&og_core::ogt::Block>) -> Result<MeshOptions, String> {
        let mut o = self.clone();
        let Some(b) = block else {
            return Ok(o);
        };
        if let Some(e) = b.get("lods") {
            let mut f = Vec::new();
            for i in 0..e.values.len() {
                f.push(
                    e.f32(i)
                        .map_err(|x| format!("line {}: lods: {x}", e.line))?,
                );
            }
            let ok = f.first() == Some(&1.0)
                && f.windows(2).all(|w| w[1] < w[0] && w[1] > 0.0)
                && f.len() <= 8;
            if !ok {
                return Err(format!(
                    "line {}: lods takes each level's fraction of the full mesh's triangles, the first 1 and each below the one before, at most 8 (`lods 1 0.5 0.25`)",
                    e.line
                ));
            }
            o.lods = Some(f);
        }
        if let Some(e) = b.get("lodError") {
            let v = e
                .f32(0)
                .map_err(|x| format!("line {}: lodError: {x}", e.line))?;
            if !(v > 0.0 && v <= 1.0) {
                return Err(format!(
                    "line {}: lodError is the largest error relative to the mesh's extent, above 0 and at most 1 (default 0.05)",
                    e.line
                ));
            }
            o.lod_max_error = v;
        }
        if let Some(e) = b.get("raycast") {
            o.raycast = e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
                format!(
                    "line {}: raycast takes true (cook the raycast BVH, the default) or false (build it at the first og.render.raycast that reaches the model)",
                    e.line
                )
            })?;
        }
        if let Some(e) = b.get("impostor") {
            let on = e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
                format!(
                    "line {}: impostor takes true (bake the model's octahedral impostor, drawn as its last level far away) or false (the default)",
                    e.line
                )
            })?;
            o.impostor = on.then(model::ImpostorBake::default);
        }
        for (key, lo, hi) in [
            ("impostorGrid", 2u32, og_render_data::MAX_IMPOSTOR_GRID),
            ("impostorSize", 16, 512),
        ] {
            let Some(e) = b.get(key) else { continue };
            let v = e
                .number(0)
                .and_then(|n| n.as_u64())
                .map_err(|x| format!("line {}: {key}: {x}", e.line))?;
            let Some(bake) = o.impostor.as_mut() else {
                return Err(format!(
                    "line {}: {key} sizes the impostor; add `impostor true`",
                    e.line
                ));
            };
            if !(u64::from(lo)..=u64::from(hi)).contains(&v) {
                return Err(format!(
                    "line {}: {key} is {} from {lo} to {hi}",
                    e.line,
                    if key == "impostorGrid" {
                        "the frames per side of the atlas"
                    } else {
                        "the pixels per frame"
                    }
                ));
            }
            match key {
                "impostorGrid" => bake.grid = v as u32,
                _ => bake.frame = v as u32,
            }
        }
        if let Some(e) = b.get("impostorBelow") {
            let on = e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
                format!(
                    "line {}: impostorBelow takes true (bake the impostor's frames from below the horizon too, for a model seen from under it: the atlases twice as wide) or false (the default)",
                    e.line
                )
            })?;
            let Some(bake) = o.impostor.as_mut() else {
                return Err(format!(
                    "line {}: impostorBelow shapes the impostor; add `impostor true`",
                    e.line
                ));
            };
            bake.below = on;
        }
        if let Some(bake) = o.impostor
            && bake.columns() * bake.frame > 4096
        {
            return Err(format!(
                "impostorGrid {} × impostorSize {}{} makes an atlas {} pixels wide, past 4096",
                bake.grid,
                bake.frame,
                if bake.below {
                    " with impostorBelow (twice the frames across)"
                } else {
                    ""
                },
                bake.columns() * bake.frame
            ));
        }
        if let Some(e) = b.get("collider") {
            use og_physics_data::trimesh::BakeKind;
            match e.values.first() {
                Some(v) if v.as_bool().is_some() => {
                    o.collider = v.as_bool().expect("matched boolean");
                    o.collider_kind = BakeKind::Trimesh;
                }
                Some(v) if v.as_word() == Some("convex") => {
                    o.collider = true;
                    o.collider_kind = BakeKind::Convex;
                }
                Some(v) if v.as_word() == Some("vhacd") => {
                    o.collider = true;
                    o.collider_kind = BakeKind::Vhacd;
                }
                _ => {
                    return Err(format!(
                        "line {}: collider takes true, false, convex or vhacd",
                        e.line
                    ));
                }
            }
        }
        Ok(o)
    }

    /// These options as a model's stamp holds them: every one its blob
    /// follows, so two cooks that would write different blobs never share
    /// a stamp. The art contract is the stamp's own part
    /// (`Conform::model_desc`); a setting left at its default adds nothing.
    pub(crate) fn describe(&self) -> String {
        let mut d = format!(
            "lods {} ratio {} error {} overdraw {} tangents {} (bitangent up the image) quantize 16/oct16/uv16/rgba8 codec meshopt-v1 sparse paired UV seams, protected boundary fans, switch-footprint shared-source baked fallback and authored-chart atlas repacking (S20){}",
            self.lod_levels,
            self.lod_ratio,
            self.lod_max_error,
            self.overdraw_threshold,
            self.tangents,
            if self.clusters { "" } else { " clusters off" }
        );
        // `og cook` turns the raycast BVH off for a game whose code never
        // raycasts: without this in the stamp, code that began to raycast
        // kept every model as cooked, without one.
        if !self.raycast {
            d.push_str(" raycast off");
        }
        if let Some(lods) = &self.lods {
            d.push_str(&format!(" fractions {lods:?}"));
        }
        if self.collider {
            d.push_str(&format!(" collider {:?}", self.collider_kind));
        }
        if let Some(b) = self.impostor {
            d.push_str(&format!(" impostor {} {} {}", b.grid, b.frame, b.below));
        }
        d
    }
}

/// Texture compression (Basis Universal).
#[derive(Clone, Debug, PartialEq)]
pub struct TextureOptions {
    /// ETC1S quality 1..=255 (colour).
    pub etc1s_quality: u32,
    /// UASTC pack level 0..=4 (normals, linear data).
    pub uastc_level: u32,
    /// UASTC rate-distortion optimisation quality scalar (lower = higher quality), or none.
    pub uastc_rdo: Option<f32>,
    /// Zstandard supercompression of UASTC levels.
    pub zstd: bool,
    pub mipmaps: bool,
    /// Encode with `toktx` even when the basis-universal crate is built in.
    pub force_toktx: bool,
    /// Encode colour textures as UASTC too (about 4x the size of ETC1S, without its 4x4 block colour loss on foliage,
    /// bark and stone detail). Off by default; project.ogt's `cook { color_textures uastc }` turns it on and
    /// `publish { color_textures etc1s }` off again for the web export (og-cli's `cook::texture_options`).
    pub color_uastc: bool,
    /// ETC1S for derived material and normal maps. Source maps stay UASTC.
    /// Normal maps use RGB = X, alpha = Y.
    pub data_etc1s: bool,
    /// This texture is a cut-out, the base colour of an alpha-tested material (a glTF material's `alphaMode`
    /// `MASK`), and the least alpha byte the material keeps: its mips are texture.rs `cutout_mips` and
    /// `keep_coverage`, so each level keeps at least the share of texels at or above the cut-off that the
    /// first has, where the encoder's own filter spreads a card's alpha into the transparent texels around it
    /// until minified levels fall under the cut-off. Set per texture by the cook (execute.rs), not a project
    /// setting; through the basis-universal encoder only (toktx makes its own mips).
    pub cutout: Option<u8>,
}

impl Default for TextureOptions {
    fn default() -> Self {
        TextureOptions {
            etc1s_quality: 128,
            uastc_level: 3,
            uastc_rdo: Some(1.0),
            zstd: true,
            mipmaps: true,
            force_toktx: false,
            color_uastc: false,
            data_etc1s: false,
            cutout: None,
        }
    }
}

impl TextureOptions {
    pub(crate) fn uastc(&self, kind: u32) -> bool {
        if kind == crate::format::kind::COLOR {
            self.color_uastc
        } else {
            !self.data_etc1s
        }
    }

    pub(crate) fn describe(&self) -> String {
        format!(
            "etc1s {} uastc {} rdo {} zstd {} mips {} normal y in alpha{}{}",
            self.etc1s_quality,
            self.uastc_level,
            self.uastc_rdo.map_or("off".to_string(), |r| r.to_string()),
            self.zstd,
            self.mipmaps,
            if self.color_uastc {
                " colour uastc"
            } else {
                ""
            },
            if self.data_etc1s {
                " data etc1s xyra"
            } else {
                ""
            }
        )
    }
}

/// A font's import settings (`<font>.ogt`'s `font { … }` block).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FontSettings {
    /// A pixel font's grid, its pixels to the em (`pixel 12`; 0: an
    /// outline font): the raster snaps the face to the grid at whole
    /// multiples of it, with no anti-aliasing.
    pub pixel: u32,
}

impl FontSettings {
    pub(crate) fn describe(&self) -> String {
        format!("pixel {}", self.pixel)
    }
}

/// Everything the cook's output depends on besides the sources.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CookOptions {
    /// Worker threads (0 = the machine's parallelism). Does not change output.
    pub threads: usize,
    /// Progress on stderr while a cook runs long: each stage and output
    /// that took 5 s or more, and every 10 s how many outputs are cooked
    /// and the one running longest. Does not change output.
    pub progress: bool,
    pub mesh: MeshOptions,
    pub texture: TextureOptions,
    pub font: FontSettings,
    /// Reflected component schemas scene tables are packed with (the engine's
    /// and the game's, as `og schema` exports them). Components without one
    /// get a layout inferred from the scene text.
    pub schemas: Vec<Schema>,
    /// A publish's cook (`og build --publish`): a prefab override that no
    /// longer does anything is refused instead of noted.
    pub publish: bool,
    /// What the game's code reaches, as its bundle resolved it
    /// ([`CodeGraph`]): the built-in UI and string tables the cook adds.
    /// Default: code that reaches none of them.
    pub code: CodeGraph,
}

pub(crate) fn settings_block<'a>(s: &'a Source, key: &str) -> Option<&'a og_core::ogt::Block> {
    s.settings.as_ref()?.1.body.get(key)?.block.as_ref()
}

/// An asset's own `conform off` (docs/art.md §2.3), at the top of its
/// `.ogt` (a `type model`, a flipbook) or of its import settings: why it
/// keeps its source as made. Read before the 2D pass takes the line out
/// of a sheet's text.
pub(crate) fn own_conform_off(s: &Source) -> Option<String> {
    let off = |b: &og_core::ogt::Block| {
        b.get("conform")
            .and_then(|e| e.values.first())
            .and_then(|v| v.as_word())
            == Some("off")
    };
    if s.settings.as_ref().is_some_and(|(_, d)| off(&d.body)) {
        match s.settings_files.as_slice() {
            [] => Some(format!("conform off (assets/{}.ogt)", s.rel)),
            files => Some(format!("conform off (assets/{})", files.join(", assets/"))),
        }
    } else if s.rel.ends_with(".ogt")
        && Document::parse(&String::from_utf8_lossy(&s.bytes)).is_ok_and(|d| off(&d.body))
    {
        Some(format!("conform off (assets/{})", s.rel))
    } else {
        None
    }
}

/// Whether the 3D pass keeps `s` as made: its own `conform off` or
/// project.ogt's `conform { off ... }` (`conform::mesh::Conform::off`).
pub(crate) fn conform_off(s: &Source, options: &CookOptions) -> bool {
    options
        .mesh
        .conform
        .as_ref()
        .is_some_and(|c| c.off.contains_key(&s.rel))
}

/// The art contract a texture `s` cooks under as a mesh's texture
/// (`conform::TextureRole::Mesh`): none when it is not one, opted out, or
/// the contract leaves textures as they are.
///
/// Without the look's paint: paint is for a model's own colour textures,
/// and one no model claims (a sprite in a 3D game, the look's own canvas)
/// keeps its strokes as drawn.
pub(crate) fn mesh_texture(s: &Source, options: &CookOptions) -> Option<conform::mesh::Conform> {
    options
        .mesh
        .conform
        .as_ref()
        .filter(|c| c.textures.contains(&s.rel))
        .filter(|_| !conform_off(s, options))
        .map(|c| conform::mesh::Conform {
            paint: None,
            ..c.clone()
        })
        .filter(|c| !c.texture_desc().is_empty())
}

/// The mesh options of a model `s` under the art contract: its import
/// settings', with the contract unless the model opts out.
pub(crate) fn mesh_for(s: &Source, options: &CookOptions) -> Result<MeshOptions, CookError> {
    let mut m = options
        .mesh
        .for_model(settings_block(s, "model"))
        .map_err(CookError)?;
    if conform_off(s, options) {
        m.conform = None;
    }
    Ok(m)
}

pub(crate) fn texture_kind(s: &Source) -> Result<u32, String> {
    match settings_block(s, "texture").and_then(|b| b.get("kind")) {
        None => Ok(format::kind::COLOR),
        Some(e) => match e.word(0).map_err(|x| x.to_string())? {
            "color" => Ok(format::kind::COLOR),
            "normal" => Ok(format::kind::NORMAL),
            "linear" => Ok(format::kind::LINEAR),
            "single" => Ok(format::kind::SINGLE),
            "pixel" => Ok(format::kind::PIXEL),
            w => Err(format!(
                "line {}: texture kind '{w}' (expected color, normal, linear, single or pixel)",
                e.line
            )),
        },
    }
}

/// A texture file's `texture { stream true }` (G3-3): its levels above
/// the mip tail stream (`<id>` holds the tail, `<id>#mips` the whole
/// chain). False or absent: one output, as ever. A `kind pixel` texture
/// has no mips to stream and is refused.
pub(crate) fn texture_stream(s: &Source, k: u32) -> Result<bool, String> {
    let Some(e) = settings_block(s, "texture").and_then(|b| b.get("stream")) else {
        return Ok(false);
    };
    let on = e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
        format!(
            "line {}: stream takes true (the levels above {} texels are read when the texture is big on screen) or false (every level loads with it, the default)",
            e.line,
            crate::runtime::MIP_TAIL
        )
    })?;
    if on && k == format::kind::PIXEL {
        return Err(format!(
            "line {}: stream true streams a texture's mip levels, and a kind pixel texture has none (RGBA8, one level); leave stream out",
            e.line
        ));
    }
    Ok(on)
}

/// A model's `model { stream_textures true }`: its glTF textures stream
/// their levels above the mip tail, as a texture file's `texture { stream
/// true }` does.
pub(crate) fn model_stream_textures(s: &Source) -> Result<bool, String> {
    let Some(e) = settings_block(s, "model").and_then(|b| b.get("stream_textures")) else {
        return Ok(false);
    };
    e.values.first().and_then(|v| v.as_bool()).ok_or_else(|| {
        format!(
            "line {}: stream_textures takes true (the model's textures stream their levels above {} texels) or false (the default)",
            e.line,
            crate::runtime::MIP_TAIL
        )
    })
}

pub(crate) fn anim_settings(s: &Source) -> Result<anim::AnimSettings, String> {
    anim::AnimSettings::parse(settings_block(s, "animation"))
}

pub(crate) fn font_settings(s: &Source, base: &FontSettings) -> Result<FontSettings, String> {
    let mut f = base.clone();
    if let Some(b) = settings_block(s, "font") {
        for e in &b.entries {
            let err = |x: og_core::Error| format!("line {}: {x}", e.line);
            match e.key.as_str() {
                "pixel" => {
                    f.pixel = e.number(0).map_err(err)?.as_u64().map_err(err)? as u32;
                    if !(4..=64).contains(&f.pixel) {
                        return Err(format!("line {}: pixel takes a grid of 4 to 64", e.line));
                    }
                }
                k => {
                    return Err(format!(
                        "line {}: unknown font setting '{k}' (pixel; og upgrade drops the atlas settings and chars)",
                        e.line
                    ));
                }
            }
        }
    }
    Ok(f)
}
