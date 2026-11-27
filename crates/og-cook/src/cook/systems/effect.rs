//! Checked FX declarations compile to native fields and typed resource tables.
//! Runtime loading only binds those resources to host handles.
use super::fx_field;
use super::*;
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;
use og_bundle::fx::cooked::{Effect, Emitter, Resource};

/// Native boundary shared by checked author inputs and external importers.
#[derive(Clone, Debug, PartialEq)]
pub enum Prepared {
    Emitter(Emitter),
    Effect(Effect),
    Vfx(og_bundle::vfx::Vfx),
}

impl Prepared {
    fn bytes(&self) -> Result<Vec<u8>, String> {
        match self {
            Self::Emitter(v) => v.to_bytes(),
            Self::Effect(v) => v.to_bytes(),
            Self::Vfx(v) => v.to_bytes(),
        }
    }

    fn references(&self) -> Vec<String> {
        match self {
            Self::Emitter(v) => v
                .resources
                .iter()
                .filter_map(Resource::asset)
                .map(str::to_owned)
                .collect(),
            Self::Effect(v) => v
                .resources
                .iter()
                .filter_map(Resource::asset)
                .map(str::to_owned)
                .collect(),
            Self::Vfx(v) => {
                let r = v.references();
                [
                    r.emitters,
                    r.effects,
                    r.models,
                    r.sounds,
                    r.flipbooks,
                    r.textures,
                ]
                .concat()
            }
        }
    }
}

/// Build after the complete content graph and field snapshots are available.
/// Imported FX already carry native fields and need no author evaluation.
pub(crate) fn prepare(sources: &mut [Source], graph: &source_content::Graph) -> Result<(), String> {
    use source_schemas::procedural::{EffectInput, ParticlesInput, VfxInput};
    let builtins: BTreeSet<_> = sources
        .iter()
        .filter(|s| s.rel.starts_with('@'))
        .map(|s| s.rel.clone())
        .collect();
    let asset = |id: &str| {
        if id.starts_with('@') {
            return builtins
                .get(id)
                .cloned()
                .ok_or_else(|| format!("FX reference '{id}' has no built-in output"));
        }
        graph
            .outputs
            .get(id)
            .map(|o| o.asset.clone())
            .ok_or_else(|| format!("FX reference '{id}' has no checked output"))
    };
    for i in 0..sources.len() {
        let s = &sources[i];
        if s.fx.is_some() || ![&EMITTER, &EFFECT, &vfx::VFX].contains(&s.kind) {
            continue;
        }
        let prepared = (|| {
            let mut value = if s.kind == &EMITTER {
                let input: ParticlesInput = s.input()?;
                Prepared::Emitter(Emitter::capture(|links| {
                    input.build(links, &|r, strength, size| {
                        let output = graph
                            .outputs
                            .get(&r.id)
                            .ok_or_else(|| format!("field '{}' has no checked output", r.id))?;
                        fx_field::bake(sources, &s.fields, output, strength, size)
                    })
                })?)
            } else if s.kind == &EFFECT {
                let input: EffectInput = s.input()?;
                Prepared::Effect(Effect::capture(|links| input.build(links), &|id| {
                    let id = asset(id)?;
                    let shader = sources.iter().find(|s| s.rel == id).ok_or_else(|| {
                        format!("effect shader '{id}' has no checked declaration")
                    })?;
                    Ok(shader_check::material_shader(&id, shader.input()?)?.params)
                })?)
            } else {
                Prepared::Vfx(s.input::<VfxInput>()?.build()?)
            };
            match &mut value {
                Prepared::Emitter(v) => bind_resources(&mut v.resources, &asset)?,
                Prepared::Effect(v) => bind_resources(&mut v.resources, &asset)?,
                Prepared::Vfx(v) => {
                    use og_bundle::vfx::{VfxKind as K, VfxShape};
                    for e in &mut v.elements {
                        match &mut e.kind {
                            K::Emitter { id, .. }
                            | K::Sound { id, .. }
                            | K::Flipbook { id, .. } => *id = asset(id)?,
                            K::Mesh { shape, effect, .. } => {
                                *effect = asset(effect)?;
                                if let VfxShape::Model(id) = shape {
                                    *id = asset(id)?;
                                }
                            }
                            K::Trail { effect, .. } | K::Beam { effect, .. } => {
                                if let Some(id) = effect {
                                    *id = asset(id)?;
                                }
                            }
                            K::Decal {
                                texture, normal, ..
                            } => {
                                for id in [texture, normal].into_iter().flatten() {
                                    *id = asset(id)?;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(value)
        })()
        .map_err(|e: String| format!("{}: {e}", s.path.display()))?;
        sources[i].fx = Some(prepared);
    }
    Ok(())
}

fn bind_resources(
    resources: &mut [Resource],
    asset: &dyn Fn(&str) -> Result<String, String>,
) -> Result<(), String> {
    for resource in resources {
        let id = match resource {
            Resource::Builtin(_) => continue,
            Resource::Texture(id)
            | Resource::Array(id)
            | Resource::Model(id)
            | Resource::Emitter(id)
            | Resource::Effect(id)
            | Resource::Shader(id) => id,
        };
        *id = asset(id)?;
    }
    Ok(())
}

pub(crate) static EMITTER: KindDef = KindDef {
    name: "emitter",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan,
};
pub(crate) static EFFECT: KindDef = KindDef {
    name: "effect",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&EMITTER, &EFFECT];

pub(crate) fn plan<'a>(
    cx: &Cx<'a>,
    _: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let prepared = s.fx.as_ref();
    let data = prepared
        .ok_or_else(|| "FX source has no prepared native fields".to_string())
        .and_then(Prepared::bytes);
    let mut references = String::new();
    if let Some(value) = prepared {
        for id in value.references() {
            references.push_str(&format!("{id}={}\n", cx.known.contains(&id)));
        }
    }
    let ty = if s.kind == &EMITTER {
        format::Emitter::NAME
    } else if s.kind == &EFFECT {
        format::Effect::NAME
    } else {
        format::Vfx::NAME
    };
    let st = stamp(
        &[Area::Effect],
        &[
            ty.as_bytes(),
            &hash,
            data.as_ref().map_or(&[], Vec::as_slice),
            references.as_bytes(),
        ],
    );
    outs.push((s.rel.clone(), ty, st, Box::new(FxJob { prepared, data })));
}

struct FxJob<'a> {
    prepared: Option<&'a Prepared>,
    data: Result<Vec<u8>, String>,
}
impl Job for FxJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let data = self.data.as_ref().map_err(|e| CookError(e.clone()))?;
        let prepared = self.prepared.expect("encoded native FX");
        let missing: Vec<_> = prepared
            .references()
            .into_iter()
            .filter(|id| !ex.known.contains(id))
            .collect();
        if !missing.is_empty() {
            return Err(CookError(format!(
                "FX references missing assets: {}",
                missing.join(", ")
            )));
        }
        let (mut writer, notes) = match prepared {
            Prepared::Emitter(v) => {
                let mut w = BlobWriter::new(
                    format::Emitter::type_hash(),
                    format::Emitter::schema_version(),
                    p.stamp,
                );
                w.table(
                    "meta",
                    &[format::EmitterMeta {
                        data_bytes: data.len() as u32,
                    }],
                    format::row_hash::<format::EmitterMeta>(),
                );
                (
                    w,
                    format!(
                        "{} particles, {} bursts",
                        v.data.max_particles,
                        v.data.bursts.len()
                    ),
                )
            }
            Prepared::Effect(v) => {
                let mut w = BlobWriter::new(
                    format::Effect::type_hash(),
                    format::Effect::schema_version(),
                    p.stamp,
                );
                w.table(
                    "meta",
                    &[format::EffectMeta {
                        data_bytes: data.len() as u32,
                    }],
                    format::row_hash::<format::EffectMeta>(),
                );
                (
                    w,
                    format!("effect material, {} resources", v.resources.len()),
                )
            }
            Prepared::Vfx(v) => {
                let mut w = BlobWriter::new(
                    format::Vfx::type_hash(),
                    format::Vfx::schema_version(),
                    p.stamp,
                );
                w.table(
                    "meta",
                    &[format::VfxMeta {
                        data_bytes: data.len() as u32,
                    }],
                    format::row_hash::<format::VfxMeta>(),
                );
                (
                    w,
                    format!(
                        "vfx: {} s, {} elements, {} events",
                        v.duration,
                        v.elements.len(),
                        v.events.len()
                    ),
                )
            }
        };
        writer.raw("data", data.clone(), 16);
        Ok((writer.finish(), vec![notes]))
    }
}
