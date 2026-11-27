//! Checked TS lighting bindings plus one immutable KTX2 page snapshot.
use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::cook::source_schemas::world::bake::{BakedAsset, LightmapInput};
use crate::format::{Area, BlobKind, Lightmap};
use crate::lightmap::{BakedModel, BakedPart, BakedVolume, LightmapFile, pages_from_ktx2};
use std::path::{Path, PathBuf};

pub(crate) struct Prepared {
    pub file: LightmapFile,
    pub path: PathBuf,
    pages: Vec<u8>,
    hash: u64,
    proxies: Vec<Option<u32>>,
    pub at: String,
}

impl Prepared {
    pub fn new(input: LightmapInput, root: &Path, at: String) -> Result<Self, String> {
        let path =
            crate::cook::source_path::checked(root, &input.pages).map_err(|e| e.to_string())?;
        if !(input.pages.starts_with("assets/") || input.pages.starts_with("sources/")) {
            return Err("lightmap pages belong in the owning module's assets/ or sources/".into());
        }
        let pages = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let hex = |value: &str| u64::from_str_radix(value, 16).map_err(|e| e.to_string());
        let mut proxies = Vec::new();
        let file = LightmapFile {
            scene: input.scene.id.clone(),
            scene_hash: hex(&input.scene_hash.0)?,
            pages: input.pages,
            samples: input.samples.unwrap_or(0),
            seconds: input.seconds.unwrap_or(0.0),
            models: input
                .models
                .into_iter()
                .map(|m| {
                    let (asset, proxy) =
                        match m.asset {
                            BakedAsset::Model(r) if !m.entity.is_empty() => (r.id, None),
                            BakedAsset::Proxy(p) if m.entity.is_empty() => (
                                crate::hlod::node_id(&input.scene.id, p.hlod_proxy as usize),
                                Some(p.hlod_proxy),
                            ),
                            _ => return Err(
                                "a model needs an entity; an HLOD proxy must have an empty entity"
                                    .into(),
                            ),
                        };
                    proxies.push(proxy);
                    Ok(BakedModel {
                        entity: m.entity,
                        asset,
                        hash: hex(&m.hash.0)?,
                        parts: m
                            .parts
                            .into_iter()
                            .map(|p| {
                                let (uv, rest) = p.uv.as_chunks::<3>();
                                if !rest.is_empty() {
                                    return Err(
                                        "lightmap UVs need complete u/v/page triples".into()
                                    );
                                }
                                Ok(BakedPart {
                                    vertices: p.vertices.unwrap_or(0),
                                    xref: p.xref,
                                    indices: p.indices,
                                    uv: uv.to_vec(),
                                })
                            })
                            .collect::<Result<_, String>>()?,
                    })
                })
                .collect::<Result<_, String>>()?,
            volumes: input
                .volumes
                .into_iter()
                .map(|v| {
                    let (probes, rest) = v.probes.as_chunks::<12>();
                    if !rest.is_empty() {
                        return Err("a light probe needs twelve RGB coefficients".into());
                    }
                    Ok(BakedVolume {
                        entity: v.entity,
                        min: v.min,
                        max: v.max,
                        fade: v.fade.unwrap_or(0.0),
                        counts: v.counts,
                        probes: probes
                            .iter()
                            .map(|p| std::array::from_fn(|i| std::array::from_fn(|j| p[4 * i + j])))
                            .collect(),
                    })
                })
                .collect::<Result<_, String>>()?,
        };
        let (_, _, layers) = pages_from_ktx2(&pages)?;
        file.validate(layers.len())?;
        let hash = super::hash_bytes(&pages);
        Ok(Self {
            file,
            path,
            pages,
            hash,
            proxies,
            at,
        })
    }

    pub fn bind(
        &mut self,
        outputs: &std::collections::BTreeMap<String, crate::cook::source_content::Output>,
    ) {
        self.file.scene = outputs[&self.file.scene].asset.clone();
        for (model, proxy) in self.file.models.iter_mut().zip(&self.proxies) {
            model.asset = proxy.map_or_else(
                || outputs[&model.asset].asset.clone(),
                |n| crate::hlod::node_id(&self.file.scene, n as usize),
            );
        }
    }
}

pub(crate) static LIGHTMAP: KindDef = KindDef {
    name: "lightmap",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&LIGHTMAP];

fn plan<'a>(
    _: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let prepared = s.lightmap.as_ref().expect("prepared lightmap");
    let mut refs = std::collections::BTreeSet::from([prepared.file.scene.clone()]);
    refs.extend(
        prepared
            .file
            .models
            .iter()
            .filter(|m| !m.entity.is_empty())
            .map(|m| m.asset.clone()),
    );
    state
        .parsed_refs
        .entry(s.rel.clone())
        .or_default()
        .extend(refs);
    // The scene must load its lighting even though author references run from
    // lighting to scene. This runtime residency edge is not an evaluation edge.
    state
        .parsed_refs
        .entry(prepared.file.scene.clone())
        .or_default()
        .insert(s.rel.clone());
    let mut bindings = vec![prepared.file.scene.as_bytes()];
    bindings.extend(prepared.file.models.iter().map(|m| m.asset.as_bytes()));
    let bindings = stamp(&[], &bindings).to_le_bytes();
    let stamp = stamp(
        &[Area::Lighting],
        &[b"lightmap", &hash, &prepared.hash.to_le_bytes(), &bindings],
    );
    outs.push((
        s.rel.clone(),
        Lightmap::NAME,
        stamp,
        Box::new(LightmapJob(prepared)),
    ));
}

struct LightmapJob<'a>(&'a Prepared);
impl Job for LightmapJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let prepared = self.0;
        Ok((
            prepared
                .file
                .to_blob(&prepared.pages, p.stamp)
                .map_err(CookError)?,
            vec![format!(
                "lightmap of '{}': {} static model(s), {} volume(s)",
                prepared.file.scene,
                prepared.file.models.len(),
                prepared.file.volumes.len()
            )],
        ))
    }
}
