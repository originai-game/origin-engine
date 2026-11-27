//! Typed terrain declarations share a checked surface with navigation/scatter.
//! Existing native kernels own noise, erosion, rivers, controls and edits.
//! The writer keeps independently compressed tile/LOD heights, horizons,
//! controls and holes. No author OGT parser or script-path frontend remains.

use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, assets_root, stamp,
};
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;

pub(crate) struct Prepared {
    pub doc: TerrainDoc,
    pub hash: u64,
    pub files: Vec<std::path::PathBuf>,
    field: std::sync::OnceLock<Result<Field, String>>,
    inputs: std::collections::BTreeMap<String, Vec<u8>>,
    fields: std::sync::Arc<super::field::Fields>,
    settings: Option<og_core::ogt::Document>,
    id: String,
    origin: crate::cook::generator::source::Diagnostic,
}

impl Prepared {
    /// Warm output plans need only the input key. Actual consumers share one
    /// immutable surface, computed on their worker, never a quantized reload.
    pub(crate) fn surface(&self) -> Result<&Field, String> {
        self.field
            .get_or_init(|| {
                let mut field = field(&self.doc, &self.inputs, &self.fields)?;
                super::district::terrain::apply(self.settings.as_ref(), &self.id, &mut field)?;
                Ok(field)
            })
            .as_ref()
            .map_err(|message| {
                crate::cook::generator::source::Diagnostic {
                    message: message.clone(),
                    ..self.origin.clone()
                }
                .to_string()
            })
    }
}

pub(crate) type Terrains = std::collections::BTreeMap<String, Prepared>;

pub(crate) fn get<'a>(s: &'a Source, id: &str) -> Result<&'a Prepared, String> {
    s.terrains.get(id).ok_or_else(|| {
        format!("terrain '{id}' is not checked TS content; use its canonical module/content ID")
    })
}

pub(crate) fn layer_files(doc: &TerrainDoc) -> impl Iterator<Item = &String> {
    doc.layers
        .iter()
        .flat_map(|(a, n, _)| [a, n])
        .chain([&doc.albedo_array, &doc.normal_array])
        .flatten()
}

pub(crate) fn prepare(
    s: &Source,
    doc: TerrainDoc,
    origin: crate::cook::generator::source::Diagnostic,
) -> Result<Prepared, String> {
    let root = assets_root(s);
    let files: Vec<_> = doc
        .heightmap
        .iter()
        .chain(&doc.control)
        .chain(&doc.holes)
        .cloned()
        .collect();
    let mut hash = og_core::hash::StableHasher::new();
    hash.write_u64(s.hash);
    let mut inputs = std::collections::BTreeMap::new();
    let mut paths = Vec::new();
    for path in &files {
        let checked =
            crate::cook::source_path::checked(&root, path).map_err(|e| format!("{path}: {e}"))?;
        let bytes = std::fs::read(&checked).map_err(|e| format!("{path}: {e}"))?;
        hash.write_u64(super::hash_bytes(&bytes));
        inputs.insert(path.clone(), bytes);
        paths.push(checked);
    }
    // Imported mesh operands can change without changing the declaration.
    // Hash their actual projected geometry before any costly terrain stages.
    for step in &doc.edits {
        if let super::terrain_edit::Step::Stamp { triangles, .. } = step {
            for value in triangles.iter().flatten().flat_map(|v| v.to_array()) {
                hash.write_u32(value.to_bits());
            }
        }
    }
    if let Some(id) = &doc.preserve {
        hash.write_u64(super::field::get(&s.fields, id)?.hash);
    }
    if let Some((settings, _)) = &s.settings {
        hash.write_u64(super::hash_bytes(settings.as_bytes()));
    }
    Ok(Prepared {
        doc,
        hash: stamp(
            &[Area::Terrain],
            &[b"terrain surface 1", &hash.finish().to_le_bytes()],
        ),
        files: paths,
        field: Default::default(),
        inputs,
        fields: s.fields.clone(),
        settings: s.settings.as_ref().map(|(_, d)| d.clone()),
        id: s.rel.clone(),
        origin,
    })
}

/// The stamp's dependencies: each file the terrain reads, with its hash,
/// and whether each layer texture is an asset.
pub(crate) fn terrain_dependencies(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    provided: &crate::cook::provided::Provided,
) -> String {
    let Ok(terrain) = get(s, &s.rel) else {
        return String::new();
    };
    let mut out = vec![format!("source={:016x}", terrain.hash)];
    out.extend(layer_files(&terrain.doc).map(|t| format!("{t}={}", named(known, provided, t))));
    out.join("\n")
}

/// What a terrain's stamp records of a layer asset: the project's own
/// (`true`), a used bundle's with its type, or neither (`false`).
fn named(
    known: &dyn Fn(&str) -> bool,
    provided: &crate::cook::provided::Provided,
    t: &str,
) -> String {
    let id = crate::runtime::asset_id(t);
    match (known(id), provided.ty(id)) {
        (true, _) => "true".into(),
        (false, Some(ty)) => format!("uses {ty}"),
        (false, None) => "false".into(),
    }
}

/// A decoded heightmap: samples in 0..1 (PNG) or as stored (EXR), row-major.
pub(crate) struct Heightmap {
    pub columns: u32,
    pub rows: u32,
    pub values: Vec<f32>,
    pub exr: bool,
}

/// Decodes a heightmap: a grey (or any) PNG to its first channel at 16
/// bits in 0..1, an EXR to its first channel as stored.
pub(crate) fn decode_heightmap(name: &str, bytes: &[u8]) -> Result<Heightmap, String> {
    let exr = name.to_ascii_lowercase().ends_with(".exr");
    let img = image::load_from_memory(bytes).map_err(|e| format!("{name}: {e}"))?;
    let (columns, rows) = (img.width(), img.height());
    if columns < 2 || rows < 2 {
        return Err(format!(
            "{name} is {columns} × {rows}: a heightmap needs at least 2 × 2"
        ));
    }
    let values: Vec<f32> = if exr {
        img.to_rgb32f().into_raw().chunks(3).map(|p| p[0]).collect()
    } else {
        img.to_luma16()
            .into_raw()
            .into_iter()
            .map(|v| v as f32 / 65535.0)
            .collect()
    };
    if values.iter().any(|v| !v.is_finite()) {
        return Err(format!("{name} holds a value that is not a finite number"));
    }
    Ok(Heightmap {
        columns,
        rows,
        values,
        exr,
    })
}

/// A terrain file's settings, read and checked.
pub(crate) struct TerrainDoc {
    /// Explicit raw height image under the owning module, if image-sourced.
    pub heightmap: Option<String>,
    pub generate: Option<super::terrain_generate::Generate>,
    pub heightfield: Option<super::super::generator::Heightfield>,
    pub preserve: Option<String>,
    pub edits: Vec<super::terrain_edit::Step>,
    /// The control maps (asset paths), one per four layers.
    pub control: Vec<String>,
    /// The hole map (an asset path).
    pub holes: Option<String>,
    pub height: [f32; 2],
    pub size: Option<[f32; 2]>,
    pub tile: u32,
    pub height_blend: f32,
    pub cast_shadows: bool,
    pub collision: bool,
    pub lod_spacing: f32,
    pub lod_ring: u32,
    pub layers: Vec<(Option<String>, Option<String>, og_render_data::TerrainLayer)>,
    /// Resolved texture-array output IDs.
    pub albedo_array: Option<String>,
    pub normal_array: Option<String>,
}

pub(crate) fn declaration(
    input: crate::cook::source_schemas::procedural::terrain::TerrainInput,
    models: &crate::cook::formats::geometry::Models<'_>,
) -> Result<TerrainDoc, String> {
    use crate::cook::source_schemas::procedural::terrain::TerrainSource;
    let mut g = super::terrain_generate::Generate {
        seed: input.seed.unwrap_or(1),
        ..Default::default()
    };
    let mut heightmap = None;
    let mut heightfield = None;
    let noise_source = matches!(&input.source, TerrainSource::Noise { .. });
    let height = match input.source {
        TerrainSource::Image { source, height } => {
            let exr = source.to_ascii_lowercase().ends_with(".exr");
            heightmap = Some(source);
            height.unwrap_or(if exr { [0.0, 1.0] } else { [0.0, 100.0] })
        }
        TerrainSource::Heightfield { field } => {
            field.validate()?;
            g.columns = field.columns;
            g.rows = field.rows;
            heightfield = Some(field);
            [0.0, 1.0]
        }
        TerrainSource::Noise {
            columns,
            rows,
            height,
            noise,
            mask,
        } => {
            g.columns = columns.unwrap_or(g.columns);
            g.rows = rows.unwrap_or(g.rows);
            if let Some(n) = noise {
                if let Some(v) = n.octaves {
                    g.octaves = v;
                }
                if let Some(v) = n.frequency {
                    g.frequency = v;
                }
                if let Some(v) = n.lacunarity {
                    g.lacunarity = v;
                }
                if let Some(v) = n.gain {
                    g.gain = v;
                }
                if let Some(v) = n.warp {
                    g.warp = v;
                }
            }
            g.mask = super::terrain_generate::mask(mask);
            height.unwrap_or([0.0, 100.0])
        }
    };
    let processed = input.process.is_some() || noise_source;
    let preserve = input
        .process
        .map(|p| super::terrain_generate::processing(&mut g, p))
        .transpose()?
        .flatten();
    g.check()?;
    let albedo_array = input.albedo_array.map(|r| r.id);
    let normal_array = input.normal_array.map(|r| r.id);
    let arrayed = albedo_array.is_some() || normal_array.is_some();
    let mut layers = Vec::new();
    for v in input.layers.unwrap_or_default() {
        let mut layer = og_render_data::TerrainLayer::default();
        if (arrayed || layers.len() >= og_render_data::TERRAIN_TEXTURED_LAYERS)
            && (v.albedo.is_some() || v.normal.is_some())
        {
            return Err("arrayed terrains and layers after the fourth use array slices, not individual albedo/normal textures".into());
        }
        let index = |value: Option<u32>, array: &Option<String>| match (value, array) {
            (Some(_), None) => {
                Err("a layer slice index requires its terrain texture array".to_string())
            }
            (v, Some(_)) => Ok(Some(v.unwrap_or(layers.len() as u32))),
            (None, None) => Ok(None),
        };
        layer.albedo_layer = index(v.albedo_layer, &albedo_array)?;
        layer.normal_layer = index(v.normal_layer, &normal_array)?;
        if let Some(color) = v.color {
            layer.color = color.into();
        }
        macro_rules! set { ($($key:ident),+) => { $(if let Some(value) = v.$key { layer.$key = value; })+ }; }
        set!(
            roughness,
            tile,
            triplanar,
            normal_strength,
            slope,
            slope_fade,
            altitude,
            altitude_fade,
            project_slope,
            anti_tiling,
            height_contrast,
            macro_variation,
            displacement,
            cell_variation
        );
        layers.push((v.albedo.map(|r| r.id), v.normal.map(|r| r.id), layer));
    }
    let control = input.control.unwrap_or_default();
    if control.len() > layers.len().div_ceil(4).max(1) {
        return Err("control takes at most one map per four layers".into());
    }
    if !g.biomes.is_empty() && !control.is_empty() {
        return Err("biomes write control; do not also name control images".into());
    }
    if g.biomes.iter().any(|b| b.layer as usize >= layers.len()) {
        return Err("biome.layer must name an existing terrain material layer".into());
    }
    let (lod_spacing, lod_ring) = input.lod.map_or((0.0, 0), |l| {
        (l.spacing.unwrap_or(0.0), l.ring.unwrap_or(0))
    });
    Ok(TerrainDoc {
        edits: super::terrain_edit::prepare(input.edits.unwrap_or_default(), layers.len(), models)?,
        heightmap,
        heightfield,
        height,
        generate: processed.then_some(g),
        preserve,
        size: input.size,
        tile: input.tile.unwrap_or(og_render_data::TERRAIN_TILE),
        height_blend: input.height_blend.unwrap_or(0.0),
        cast_shadows: input.cast_shadows.unwrap_or(true),
        collision: input.collision.unwrap_or(true),
        control,
        holes: input.holes,
        lod_spacing,
        lod_ring,
        layers,
        albedo_array,
        normal_array,
    })
}

/// A terrain's samples, read from its heightmap (and control map) or made
/// by its generator.
#[derive(Clone)]
pub(crate) struct Field {
    pub columns: u32,
    pub rows: u32,
    /// Metres above the origin (what the runtime's terrain draws, before
    /// quantisation): a heightmap's `min + v · (max - min)`.
    pub heights: Vec<f32>,
    pub control: Option<og_render_data::TerrainControl>,
    /// Its cells cut out (`og_render_data::TerrainData::holes`; `None`: none).
    pub holes: Option<Vec<u8>>,
    /// Its extent: `size`, or a metre per sample.
    pub size: [f32; 2],
    pub exr: bool,
    /// Made by `generate`: its sea level and its rivers' length in metres.
    pub generated: Option<(Option<f32>, f32)>,
}

/// Reads (or makes) a terrain file's samples.
pub(crate) fn field(
    doc: &TerrainDoc,
    inputs: &std::collections::BTreeMap<String, Vec<u8>>,
    fields: &super::field::Fields,
) -> Result<Field, String> {
    // The control maps, one page each, all of the first's size.
    let control = || -> Result<Option<og_render_data::TerrainControl>, String> {
        let mut out: Option<(u32, u32, Vec<u8>)> = None;
        for c in &doc.control {
            let img = super::texture::decode(&inputs[c]).map_err(|e| format!("{c}: {e}"))?;
            match &mut out {
                None => out = Some((img.width, img.height, img.rgba)),
                Some((w, h, rgba)) => {
                    if (img.width, img.height) != (*w, *h) {
                        return Err(format!(
                            "{c} is {} × {}: every control map of a terrain is the first's size, {w} × {h}",
                            img.width, img.height
                        ));
                    }
                    rgba.extend_from_slice(&img.rgba);
                }
            }
        }
        Ok(
            out.map(|(width, height, rgba)| og_render_data::TerrainControl {
                width,
                height,
                rgba: rgba.into(),
            }),
        )
    };
    // Each cell a hole where the map is black or transparent at its centre.
    let holes = |columns: u32, rows: u32| -> Result<Option<Vec<u8>>, String> {
        let Some(h) = &doc.holes else {
            return Ok(None);
        };
        let img = super::texture::decode(&inputs[h]).map_err(|e| format!("{h}: {e}"))?;
        let (cx, cz) = (columns - 1, rows - 1);
        let mut out = vec![0u8; (cx * cz) as usize];
        for r in 0..cz {
            let y = ((r as f32 + 0.5) / cz as f32 * img.height as f32) as u32;
            for c in 0..cx {
                let x = ((c as f32 + 0.5) / cx as f32 * img.width as f32) as u32;
                let at = ((y.min(img.height - 1) * img.width + x.min(img.width - 1)) * 4) as usize;
                let p = &img.rgba[at..at + 4];
                let black = p[0].max(p[1]).max(p[2]) < 128;
                out[(r * cx + c) as usize] = u8::from(black || p[3] < 128);
            }
        }
        Ok(out.contains(&1).then_some(out))
    };
    let size_of = |columns: u32, rows: u32| {
        doc.size
            .unwrap_or([(columns - 1) as f32, (rows - 1) as f32])
    };
    let base = if let Some(field) = &doc.heightfield {
        Some(Heightmap {
            columns: field.columns,
            rows: field.rows,
            values: field.heights.clone(),
            exr: true,
        })
    } else if let Some(name) = &doc.heightmap {
        let mut map = decode_heightmap(name, &inputs[name])?;
        let [lo, hi] = doc.height;
        for h in &mut map.values {
            *h = lo + *h * (hi - lo);
        }
        Some(map)
    } else {
        None
    };
    if let Some(mut g) = doc.generate.clone() {
        if let Some(base) = &base {
            g.columns = base.columns;
            g.rows = base.rows;
        }
        let size = size_of(g.columns, g.rows);
        if let Some(rel) = &doc.preserve {
            let data = &super::field::get(fields, rel)?.data;
            if data.dimensions != [g.columns, g.rows, 1] || data.channels != 1 {
                return Err("preserve field must match terrain columns × rows × 1, one channel; no resampling".into());
            }
            g.preserve = data.values.clone();
        }
        g.check()?;
        let made = match base {
            Some(map) => {
                let field = super::super::generator::Heightfield {
                    columns: map.columns,
                    rows: map.rows,
                    heights: map.values,
                };
                field.validate()?;
                super::terrain_generate::process(&g, size, None, field.heights)
            }
            None => super::terrain_generate::generate(&g, size, doc.height),
        };
        if made.heights.iter().any(|h| !h.is_finite()) {
            return Err("terrain processing produced non-finite heights; reduce the input magnitude or stage parameters".into());
        }
        // The biomes' pages, then empty ones up to the layers'.
        let control = match made.control {
            Some(mut rgba) => {
                let page = (g.columns * g.rows * 4) as usize;
                let pages = doc.layers.len().div_ceil(4).max(rgba.len() / page);
                rgba.resize(page * pages, 0);
                Some(og_render_data::TerrainControl {
                    width: g.columns,
                    height: g.rows,
                    rgba: rgba.into(),
                })
            }
            None => control()?,
        };
        return super::terrain_edit::apply(
            Field {
                columns: g.columns,
                rows: g.rows,
                heights: made.heights,
                control,
                holes: holes(g.columns, g.rows)?,
                size,
                exr: false,
                generated: Some((g.sea, made.river_length)),
            },
            &doc.edits,
        );
    }
    let map = base.expect("typed source supplies an image, heightfield or noise");
    super::terrain_edit::apply(
        Field {
            columns: map.columns,
            rows: map.rows,
            heights: map.values,
            control: control()?,
            holes: holes(map.columns, map.rows)?,
            size: size_of(map.columns, map.rows),
            exr: map.exr,
            generated: None,
        },
        &doc.edits,
    )
}

pub(crate) fn cook_terrain(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    provided: &crate::cook::provided::Provided,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let prepared = get(s, &s.rel).map_err(CookError)?;
    let doc = &prepared.doc;
    let source = prepared.surface().map_err(CookError)?.clone();
    let (columns, rows, size) = (source.columns, source.rows, source.size);
    let (sea, river_length) = source.generated.unwrap_or((None, 0.0));
    // A 16-bit heightmap's samples are already steps of the declared
    // `height`: its tiles keep those steps, so terrains declared over one
    // range with equal edge samples meet exactly. An EXR's or a generated
    // field's heights are not steps of anything: they take the field's own.
    let declared = (doc.heightmap.is_some() && doc.generate.is_none() && !source.exr).then(|| {
        (
            doc.height[0].min(doc.height[1]),
            doc.height[0].max(doc.height[1]),
        )
    });
    // A layer's texture or array is the project's own, or one a bundle of
    // project.ts's cook.uses provides with the right type (packed
    // there, not here: the game opens that bundle before og.terrain.load).
    let mut missing = Vec::new();
    let mut wrong = Vec::new();
    let textures = doc.layers.iter().flat_map(|(a, n, _)| [a, n]).flatten();
    let arrays = [&doc.albedo_array, &doc.normal_array].into_iter().flatten();
    for (t, want) in textures
        .map(|t| (t, format::Texture::NAME))
        .chain(arrays.map(|t| (t, format::TextureArray::NAME)))
    {
        if known(crate::runtime::asset_id(t)) {
            continue;
        }
        match (provided.ty(t), provided.wrong_type(t, want)) {
            (None, _) => missing.push(t.clone()),
            (Some(_), Some(why)) => wrong.push(why),
            (Some(_), None) => {}
        }
    }
    if !missing.is_empty() {
        return Err(CookError(format!(
            "the terrain names texture(s) not in project content or a bundle of project.ts cook.uses: {}",
            missing.join(", ")
        )));
    }
    if !wrong.is_empty() {
        return Err(CookError(format!(
            "the terrain names asset(s) of the wrong type: {}",
            wrong.join("; ")
        )));
    }

    let control = source.control;
    let (control_w, control_h) = control.as_ref().map_or((0, 0), |c| (c.width, c.height));
    let hole_cells = source
        .holes
        .as_ref()
        .map_or(0, |h| h.iter().filter(|c| **c != 0).count() as u32);
    let field = og_render_data::TerrainData {
        size: glam::Vec2::new(size[0], size[1]),
        columns,
        rows,
        heights: source.heights.into(),
        control,
        holes: source.holes.map(Into::into),
    };
    field.check().map_err(CookError)?;
    let layout = field.layout(doc.tile).map_err(CookError)?;
    let (lo_all, hi_all) = field.range();
    // A street carve can extend a heightmap below its declared range.
    let (step_lo, step_hi) = declared
        .map(|(lo, hi)| (lo.min(lo_all), hi.max(hi_all)))
        .unwrap_or((lo_all, hi_all));
    let span = (step_hi - step_lo).max(f32::MIN_POSITIVE);
    let pyramid = field
        .control
        .as_ref()
        .and_then(|c| og_render_data::ControlPyramid::new(&layout, c));
    let holes = field.hole_counts();
    let heights = field.height_pyramid(&layout);
    // Every tile of every level: its heights quantised over the steps'
    // range (with holes, followed by its hole mask),
    // its control texels, each zlib'd on its own.
    let mut tiles = Vec::new();
    let mut packed = Vec::new();
    let mut control_packed = Vec::new();
    for index in 0..layout.tile_count() {
        let (level, x, z) = layout.locate(index).expect("in the layout");
        let tile = field.tile_heights(&layout, index, &heights);
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        let mut q = Vec::with_capacity(tile.len() * 2);
        for v in &tile {
            lo = lo.min(*v);
            hi = hi.max(*v);
            let n = (((v - step_lo) / span) * 65535.0)
                .round()
                .clamp(0.0, 65535.0) as u16;
            q.extend_from_slice(&n.to_le_bytes());
        }
        // Then its horizon (og-render's `HeightPyramid::horizon`), in the
        // same stream: a host that inflates the heights inflates it too.
        for w in field.horizon(&layout, index, &heights) {
            q.extend_from_slice(&w.to_le_bytes());
        }
        // With holes, then its hole mask, a byte per sample's cell: the
        // mask the renderer and physics read as it is.
        if let Some(h) = &holes {
            q.extend_from_slice(&h.tile_mask(&layout, index));
        }
        let z_heights = miniz_oxide::deflate::compress_to_vec_zlib(&q, 6);
        let (control_offset, control_bytes) = match &pyramid {
            Some(p) => {
                let zc = miniz_oxide::deflate::compress_to_vec_zlib(&p.tile(&layout, index), 6);
                let at = control_packed.len() as u32;
                control_packed.extend_from_slice(&zc);
                (at, zc.len() as u32)
            }
            None => (0, 0),
        };
        tiles.push(format::TerrainTileRow {
            index,
            level,
            x,
            z,
            min: lo,
            max: hi,
            offset: packed.len() as u32,
            bytes: z_heights.len() as u32,
            control_offset,
            control_bytes,
        });
        packed.extend_from_slice(&z_heights);
    }
    let mut strings = String::new();
    let mut put = |s: &Option<String>| -> (u32, u32) {
        match s {
            None => (format::NONE, 0),
            Some(s) => {
                let at = strings.len() as u32;
                let id = crate::runtime::asset_id(s);
                strings.push_str(id);
                (at, id.len() as u32)
            }
        }
    };
    let layers: Vec<format::TerrainLayerRow> = doc
        .layers
        .iter()
        .map(|(a, n, l)| {
            let (albedo_at, albedo_len) = put(a);
            let (normal_at, normal_len) = put(n);
            format::TerrainLayerRow {
                albedo_at,
                albedo_len,
                normal_at,
                normal_len,
                r: l.color.x,
                g: l.color.y,
                b: l.color.z,
                roughness: l.roughness,
                tile: l.tile,
                triplanar: u32::from(l.triplanar),
                normal_strength: l.normal_strength,
                slope_min: l.slope[0],
                slope_max: l.slope[1],
                slope_fade: l.slope_fade,
                altitude_min: l.altitude[0],
                altitude_max: l.altitude[1],
                altitude_fade: l.altitude_fade,
                albedo_layer: l.albedo_layer.unwrap_or(format::NONE),
                normal_layer: l.normal_layer.unwrap_or(format::NONE),
                project_slope: l.project_slope,
                anti_tiling: l.anti_tiling,
                height_contrast: l.height_contrast,
                macro_variation: l.macro_variation,
                displacement: l.displacement,
                cell_variation: l.cell_variation,
            }
        })
        .collect();
    let (albedo_array_at, albedo_array_len) = put(&doc.albedo_array);
    let (normal_array_at, normal_array_len) = put(&doc.normal_array);
    let meta = format::TerrainMeta {
        size_x: size[0],
        size_z: size[1],
        columns,
        rows,
        tile: layout.tile,
        control: layout.control,
        levels: layout.levels,
        height_min: lo_all,
        height_max: hi_all,
        steps_min: step_lo,
        steps_max: step_hi,
        layers: layers.len() as u32,
        control_width: control_w,
        control_height: control_h,
        height_blend: doc.height_blend,
        lod_spacing: doc.lod_spacing,
        lod_ring: doc.lod_ring,
        cast_shadows: u32::from(doc.cast_shadows),
        collision: u32::from(doc.collision),
        albedo_array_at,
        albedo_array_len,
        normal_array_at,
        normal_array_len,
        generated: u32::from(source.generated.is_some()),
        sea: u32::from(sea.is_some()),
        sea_level: sea.unwrap_or(0.0),
        river_length,
        holes: hole_cells,
        control_pages: layout.pages,
    };
    let mut w = BlobWriter::new(
        format::Terrain::type_hash(),
        format::Terrain::schema_version(),
        stamp,
    );
    w.table("meta", &[meta], format::row_hash::<format::TerrainMeta>())
        .table(
            "tiles",
            &tiles,
            format::row_hash::<format::TerrainTileRow>(),
        )
        .table(
            "layers",
            &layers,
            format::row_hash::<format::TerrainLayerRow>(),
        )
        .raw("strings", strings.into_bytes(), 16)
        .raw("heights", packed, 16)
        .raw("control", control_packed, 16);
    let mut notes = vec![format!(
        "terrain: {columns} × {rows} samples over {} × {} m, heights {lo_all:.2} to {hi_all:.2} m, {} tiles of {} quads in {} levels, {} layer(s){}",
        size[0],
        size[1],
        tiles.len(),
        layout.tile,
        layout.levels,
        layers.len(),
        if control_w > 0 {
            format!(
                ", control map {control_w} × {control_h} ({} page(s))",
                layout.pages
            )
        } else {
            String::new()
        }
    )];
    if hole_cells > 0 {
        notes.push(format!("{hole_cells} cell(s) cut out as holes"));
    }
    if source.exr {
        notes.push("heights from an EXR's first channel".into());
    }
    if let Some(g) = &doc.generate {
        notes.push(format!(
            "generated: seed {}, {}{} m of rivers",
            g.seed,
            sea.map_or(String::new(), |s| format!("sea at {s} m, ")),
            river_length.round()
        ));
    }
    Ok((w.finish(), notes))
}

pub(crate) static TERRAIN: KindDef = KindDef {
    name: "terrain",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_terrain,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&TERRAIN];

fn plan_terrain<'a>(
    cx: &Cx<'a>,
    _: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx {
        known, provided, ..
    } = *cx;
    let deps = terrain_dependencies(s, &|id| known.contains(id), provided);
    let st = stamp(&[Area::Terrain], &[b"terrain 2", &hash, deps.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Terrain::NAME,
        st,
        Box::new(TerrainJob(s, provided)),
    ));
}

/// A terrain's one output, with what the used bundles provide.
struct TerrainJob<'a>(&'a Source, &'a crate::cook::provided::Provided);

impl Job for TerrainJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_terrain(self.0, &|id| ex.known.contains(id), self.1, p.stamp)
    }
}
