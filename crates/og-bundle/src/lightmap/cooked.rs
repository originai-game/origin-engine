//! Native lighting tables shared by the cook and both runtime hosts.
use super::*;
use crate::format::*;
use crate::runtime::{CookedLightmap, LoadError, invalid, one, table};
use og_blob::{Blob, BlobWriter};

impl LightmapFile {
    /// Check topology and numeric domains before writing or using a bake.
    pub fn validate(&self, pages: usize) -> Result<(), String> {
        if self.scene.is_empty() || !self.seconds.is_finite() || self.seconds < 0.0 {
            return Err("a lightmap needs a scene and nonnegative finite seconds".into());
        }
        let mut entities = std::collections::BTreeSet::new();
        let mut proxies = std::collections::BTreeSet::new();
        for model in &self.models {
            if model.asset.is_empty()
                || if model.entity.is_empty() {
                    !proxies.insert(&model.asset)
                } else {
                    !entities.insert(&model.entity)
                }
            {
                return Err(
                    "lightmap models need assets and unique entity or proxy bindings".into(),
                );
            }
            for part in &model.parts {
                if part.xref.len() != part.uv.len()
                    || !part.indices.len().is_multiple_of(3)
                    || part.xref.iter().any(|i| *i >= part.vertices)
                    || part.indices.iter().any(|i| *i as usize >= part.xref.len())
                {
                    return Err(format!(
                        "model '{}': invalid vertex remapping or triangle indices",
                        model.entity
                    ));
                }
                if part.uv.iter().any(|uv| {
                    !uv.iter().all(|v| v.is_finite())
                        || !(0.0..=1.0).contains(&uv[0])
                        || !(0.0..=1.0).contains(&uv[1])
                        || uv[2] < 0.0
                        || uv[2].fract() != 0.0
                        || uv[2] as usize >= pages
                }) {
                    return Err(format!(
                        "model '{}': invalid lightmap UV or page",
                        model.entity
                    ));
                }
            }
        }
        entities.clear();
        for volume in &self.volumes {
            let count = volume
                .counts
                .iter()
                .try_fold(1usize, |n, c| n.checked_mul(*c as usize));
            if volume.entity.is_empty()
                || !entities.insert(&volume.entity)
                || !volume.fade.is_finite()
                || volume.fade < 0.0
                || volume.counts.contains(&0)
                || count != Some(volume.probes.len())
                || (0..3).any(|i| {
                    !volume.min[i].is_finite()
                        || !volume.max[i].is_finite()
                        || volume.min[i] > volume.max[i]
                })
                || !volume
                    .probes
                    .iter()
                    .flatten()
                    .flatten()
                    .all(|v| v.is_finite())
            {
                return Err(format!(
                    "volume '{}': invalid bounds, counts or irradiance probes",
                    volume.entity
                ));
            }
        }
        Ok(())
    }

    /// Encode native rows with the exact captured pages, never author text.
    pub fn to_blob(&self, pages: &[u8], stamp: u64) -> Result<Vec<u8>, String> {
        let (width, height, layers) = pages_from_ktx2(pages)?;
        self.validate(layers.len())?;
        let mut strings = Vec::new();
        let mut string = |s: &str| {
            let start = strings.len() as u32;
            strings.extend_from_slice(s.as_bytes());
            (start, s.len() as u32)
        };
        let (scene_offset, scene_len) = string(&self.scene);
        let (source_offset, source_len) = string(&self.pages);
        let mut models = Vec::new();
        let mut parts = Vec::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for model in &self.models {
            let (entity_offset, entity_len) = string(&model.entity);
            let (asset_offset, asset_len) = string(&model.asset);
            models.push(LightmapModelRow {
                entity_offset,
                entity_len,
                asset_offset,
                asset_len,
                hash: model.hash,
                parts_start: parts.len() as u32,
                parts_count: model.parts.len() as u32,
            });
            for part in &model.parts {
                parts.push(LightmapPartRow {
                    vertices: part.vertices,
                    xref_start: vertices.len() as u32,
                    xref_count: part.xref.len() as u32,
                    indices_start: (indices.len() / 4) as u32,
                    indices_count: part.indices.len() as u32,
                });
                vertices.extend(part.xref.iter().zip(&part.uv).map(|(xref, uv)| {
                    LightmapVertexRow {
                        xref: *xref,
                        u: uv[0],
                        v: uv[1],
                        page: uv[2],
                    }
                }));
                indices.extend(part.indices.iter().flat_map(|v| v.to_le_bytes()));
            }
        }
        let mut volumes = Vec::new();
        let mut probes = Vec::new();
        for volume in &self.volumes {
            let (entity_offset, entity_len) = string(&volume.entity);
            volumes.push(LightmapVolumeRow {
                entity_offset,
                entity_len,
                min_x: volume.min[0],
                min_y: volume.min[1],
                min_z: volume.min[2],
                max_x: volume.max[0],
                max_y: volume.max[1],
                max_z: volume.max[2],
                fade: volume.fade,
                count_x: volume.counts[0],
                count_y: volume.counts[1],
                count_z: volume.counts[2],
                probes_start: (probes.len() / 48) as u32,
                probes_count: volume.probes.len() as u32,
            });
            probes.extend(
                volume
                    .probes
                    .iter()
                    .flatten()
                    .flatten()
                    .flat_map(|v| v.to_le_bytes()),
            );
        }
        let mut writer = BlobWriter::new(Lightmap::type_hash(), Lightmap::schema_version(), stamp);
        writer
            .table(
                "meta",
                &[LightmapMeta {
                    scene_offset,
                    scene_len,
                    scene_hash: self.scene_hash,
                    source_offset,
                    source_len,
                    samples: self.samples,
                    seconds: self.seconds,
                    pages_bytes: pages.len() as u32,
                    pages: layers.len() as u32,
                    width,
                    height,
                }],
                row_hash::<LightmapMeta>(),
            )
            .table("models", &models, row_hash::<LightmapModelRow>())
            .table("parts", &parts, row_hash::<LightmapPartRow>())
            .table("vertices", &vertices, row_hash::<LightmapVertexRow>())
            .table("volumes", &volumes, row_hash::<LightmapVolumeRow>())
            .raw("indices", indices, 4)
            .raw("probes", probes, 4)
            .raw("strings", strings, 1)
            .raw("pages", pages.to_vec(), 16);
        Ok(writer.finish())
    }
}

fn range<T>(values: &[T], start: u32, count: u32) -> Result<&[T], &'static str> {
    let start = start as usize;
    values
        .get(
            start
                ..start
                    .checked_add(count as usize)
                    .ok_or("lightmap range overflow")?,
        )
        .ok_or("lightmap table range is out of bounds")
}

pub(crate) fn decode(blob: &Blob<'_>) -> Result<CookedLightmap, LoadError> {
    let meta = one::<LightmapMeta>(blob)?;
    let strings = blob.raw("strings")?;
    let text = |start, count| {
        std::str::from_utf8(range(strings, start, count).map_err(|e| invalid(blob, e))?)
            .map(str::to_owned)
            .map_err(|_| invalid(blob, "lightmap string is not UTF-8"))
    };
    let rows = table::<LightmapModelRow>(blob, "models")?;
    let parts = table::<LightmapPartRow>(blob, "parts")?;
    let vertices = table::<LightmapVertexRow>(blob, "vertices")?;
    let volumes = table::<LightmapVolumeRow>(blob, "volumes")?;
    let indices = blob.raw("indices")?;
    let probes = blob.raw("probes")?;
    if !indices.len().is_multiple_of(4) || !probes.len().is_multiple_of(48) {
        return Err(invalid(blob, "lightmap indices or probes are cut short"));
    }
    let indices: Vec<_> = indices
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| u32::from_le_bytes(*v))
        .collect();
    let probes: Vec<[[f32; 4]; 3]> = probes
        .as_chunks::<48>()
        .0
        .iter()
        .map(|p| {
            std::array::from_fn(|c| {
                std::array::from_fn(|i| {
                    let at = (c * 4 + i) * 4;
                    f32::from_le_bytes(p[at..at + 4].try_into().expect("four bytes"))
                })
            })
        })
        .collect();
    let mut file = LightmapFile {
        scene: text(meta.scene_offset, meta.scene_len)?,
        scene_hash: meta.scene_hash,
        pages: text(meta.source_offset, meta.source_len)?,
        samples: meta.samples,
        seconds: meta.seconds,
        ..Default::default()
    };
    for model in rows.iter() {
        let mut output = BakedModel {
            entity: text(model.entity_offset, model.entity_len)?,
            asset: text(model.asset_offset, model.asset_len)?,
            hash: model.hash,
            parts: Vec::new(),
        };
        for part in
            range(&parts, model.parts_start, model.parts_count).map_err(|e| invalid(blob, e))?
        {
            let v =
                range(&vertices, part.xref_start, part.xref_count).map_err(|e| invalid(blob, e))?;
            output.parts.push(BakedPart {
                vertices: part.vertices,
                xref: v.iter().map(|v| v.xref).collect(),
                uv: v.iter().map(|v| [v.u, v.v, v.page]).collect(),
                indices: range(&indices, part.indices_start, part.indices_count)
                    .map_err(|e| invalid(blob, e))?
                    .to_vec(),
            });
        }
        file.models.push(output);
    }
    for v in volumes.iter() {
        file.volumes.push(BakedVolume {
            entity: text(v.entity_offset, v.entity_len)?,
            min: [v.min_x, v.min_y, v.min_z],
            max: [v.max_x, v.max_y, v.max_z],
            fade: v.fade,
            counts: [v.count_x, v.count_y, v.count_z],
            probes: range(&probes, v.probes_start, v.probes_count)
                .map_err(|e| invalid(blob, e))?
                .to_vec(),
        });
    }
    let pages = blob.raw("pages")?;
    if pages.len() != meta.pages_bytes as usize {
        return Err(invalid(blob, "lightmap page byte count differs"));
    }
    let (width, height, pages) = pages_from_ktx2(pages).map_err(|e| invalid(blob, e))?;
    if (width, height, pages.len()) != (meta.width, meta.height, meta.pages as usize) {
        return Err(invalid(blob, "lightmap page dimensions differ"));
    }
    file.validate(pages.len()).map_err(|e| invalid(blob, e))?;
    Ok(CookedLightmap {
        file,
        width,
        height,
        pages,
    })
}
