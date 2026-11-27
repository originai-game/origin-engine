//! Checked named graphs resolve canonical model bindings to native clip slots.
use super::*;
use og_anim::GraphSource;
use og_blob::BlobWriter;

pub(crate) static GRAPH: KindDef = KindDef {
    name: "animgraph",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&GRAPH];

pub(crate) fn declaration(
    input: source_schemas::media::animgraph::Animgraph,
) -> Result<GraphSource, String> {
    let definition = og_anim::GraphDefinition {
        params: input.params,
        motions: input.motions,
        layers: input.layers,
    };
    let graph = og_anim::SpriteGraph::new(&definition).map_err(|e| e.to_string())?;
    let source = GraphSource {
        asset: graph.asset,
        clips: graph.clips,
        sprite: input.sprite,
        bindings: input
            .bindings
            .into_iter()
            .map(|b| og_anim::ClipBinding {
                name: b.name,
                source: b.source.id,
                clip: b.clip,
            })
            .collect(),
    };
    source.validate().map_err(|e| e.to_string())?;
    Ok(source)
}

fn plan<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let result = (|| -> Result<(Vec<u8>, Vec<u8>), String> {
        let graph = s
            .animgraph
            .as_ref()
            .ok_or("animation graph requires file.animgraph")?;
        let mut deps = Vec::new();
        for b in &graph.bindings {
            let source = cx
                .sources
                .iter()
                .find(|s| s.rel == b.source)
                .ok_or_else(|| {
                    format!(
                        "bind '{}': source '{}' is not a model asset",
                        b.name, b.source
                    )
                })?;
            let g = source.gltf.as_ref().ok_or_else(|| {
                format!(
                    "bind '{}': '{}' did not import as a model",
                    b.name, b.source
                )
            })?;
            let settings = anim_settings(source)?;
            let names = anim::clip_names(g, &settings, cx.assets_dir)?;
            if !names.contains(&b.clip) {
                return Err(format!(
                    "bind '{}': '{}' has no clip '{}'; available: {}",
                    b.name,
                    b.source,
                    b.clip,
                    names.join(", ")
                ));
            }
            deps.extend_from_slice(&source.hash.to_le_bytes());
            deps.extend_from_slice(
                anim::dependencies(cx.assets_dir, &settings, cx.rigs).as_bytes(),
            );
            state
                .parsed_refs
                .entry(s.rel.clone())
                .or_default()
                .insert(format!("{}#anim", b.source));
        }
        Ok((graph.to_bytes(), deps))
    })();
    match result {
        Ok((text, deps)) => {
            let st = stamp(&[Area::Anim], &[b"animgraph", &hash, &deps, &text]);
            outs.push((
                s.rel.clone(),
                format::AnimGraph::NAME,
                st,
                Box::new(GraphJob(text)),
            ));
        }
        Err(e) => state.early_errors.push((s.rel.clone(), "animgraph", e)),
    }
}

struct GraphJob(Vec<u8>);
impl Job for GraphJob {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let mut w = BlobWriter::new(
            format::AnimGraph::type_hash(),
            format::AnimGraph::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::AnimGraphMeta {
                data_bytes: self.0.len() as u32,
            }],
            format::row_hash::<format::AnimGraphMeta>(),
        )
        .raw("data", self.0.clone(), 16);
        Ok((w.finish(), Vec::new()))
    }
    fn action(&self) -> Action<'_> {
        Action::Input(&self.0, false)
    }
}
