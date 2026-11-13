//! Effect materials (`type effect`, `og_bundle::effect`): the cook checks
//! the file with the runtime's parser and that every texture and shader it
//! names is an asset, and stores the text, which the runtime parses again
//! with the bundle's handles.

use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, Work, fx_field, stamp,
};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;

pub(crate) fn cook_effect(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("an effect material is text (UTF-8)".into()))?;
    let missing = std::cell::RefCell::new(Vec::new());
    let check = |id: &str| {
        if !known(crate::runtime::asset_id(id)) {
            missing.borrow_mut().push(id.to_string());
        }
    };
    let m = og_bundle::effect::parse_effect(
        text,
        &mut og_bundle::effect::EffectLinks {
            texture: &mut |id| {
                check(id);
                Ok(None)
            },
            shader: &mut |id| {
                check(id);
                Ok(None)
            },
        },
    )
    .map_err(CookError)?;
    let missing = missing.into_inner();
    if !missing.is_empty() {
        return Err(CookError(format!(
            "the effect names asset(s) not under assets/ (a texture or shader): {}",
            missing.join(", ")
        )));
    }
    let mut w = BlobWriter::new(
        format::Effect::type_hash(),
        format::Effect::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::EffectMeta {
            text_bytes: text.len() as u32,
        }],
        format::row_hash::<format::EffectMeta>(),
    )
    .raw("text", text.as_bytes().to_vec(), 16);
    let parts = [
        m.ramp.is_some().then_some("ramp"),
        m.mask.is_some().then_some("mask"),
        m.erosion.is_some().then_some("erosion"),
        m.flow.is_some().then_some("flow"),
        m.fresnel.is_some().then_some("fresnel"),
        m.shader.is_some().then_some("hook"),
    ];
    let parts: Vec<&str> = parts.into_iter().flatten().collect();
    Ok((
        w.finish(),
        vec![format!("effect material: {}", parts.join(", "))],
    ))
}

pub(crate) static EMITTER: KindDef = KindDef {
    name: "emitter",
    extensions: &[],
    types: &[crate::fx::EMITTER_TYPE],
    by_content: true,
    streams: false,
    plan: plan_emitter,
};

fn plan_emitter<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let PlanState { parsed_refs, .. } = state;
    let text = String::from_utf8_lossy(&s.bytes);
    let refs: Vec<String> = crate::fx::emitter_references(&text)
        .into_iter()
        .inspect(|r| {
            parsed_refs
                .entry(s.rel.clone())
                .or_default()
                .insert(r.clone());
        })
        .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
        .collect();
    let fields = fx_field::stamp(s, &text);
    let st = stamp(&[
        b"emitter",
        &hash,
        refs.join("\n").as_bytes(),
        fields.as_bytes(),
    ]);
    outs.push((
        s.rel.clone(),
        format::Emitter::NAME,
        st,
        Work::Job(Box::new(EmitterJob(s))),
    ));
}

/// An emitter's one output.
struct EmitterJob<'a>(&'a Source);

impl Job for EmitterJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let EmitterJob(s) = self;
        let Exec { known, .. } = ex;
        let text = std::str::from_utf8(&s.bytes)
            .map_err(|_| CookError("an emitter is text (UTF-8)".into()))?;
        let baked = fx_field::bake(s, text).map_err(CookError)?;
        let text = baked.as_str();
        let missing = std::cell::RefCell::new(Vec::new());
        let check = |id: &str| {
            if !known.contains(crate::runtime::asset_id(id)) {
                missing.borrow_mut().push(id.to_string());
            }
        };
        let em = crate::fx::parse_emitter_linked(
            text,
            &mut crate::fx::EmitterLinks {
                texture: &mut |id| {
                    check(id);
                    Ok(None)
                },
                array: &mut |id| {
                    check(id);
                    Ok(None)
                },
                mesh: &mut |m| {
                    if let crate::fx::ParticleMesh::Asset(id) = m {
                        check(id);
                    }
                    Ok(None)
                },
                emitter: &mut |id| {
                    check(id);
                    Ok(None)
                },
                effect: &mut |id| {
                    check(id);
                    Ok(None)
                },
            },
        )
        .map_err(CookError)?;
        let missing = missing.into_inner();
        if !missing.is_empty() {
            return Err(CookError(format!(
                "the emitter names asset(s) not under assets/ (a texture, texture array, mesh model or sub-emitter): {}",
                missing.join(", ")
            )));
        }
        let mut w = og_blob::BlobWriter::new(
            format::Emitter::type_hash(),
            format::Emitter::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::EmitterMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::EmitterMeta>(),
        )
        .raw("text", text.as_bytes().to_vec(), 16);
        Ok((
            w.finish(),
            vec![format!(
                "{} particles, {} burst(s)",
                em.max_particles,
                em.bursts.len()
            )],
        ))
    }
}

pub(crate) static EFFECT: KindDef = KindDef {
    name: "effect",
    extensions: &[],
    types: &[og_bundle::effect::EFFECT_TYPE],
    by_content: false,
    streams: false,
    plan: plan_effect,
};

fn plan_effect<'a>(
    cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let text = String::from_utf8_lossy(&s.bytes);
    let refs: Vec<String> = og_bundle::effect::effect_references(&text)
        .into_iter()
        .map(|r| format!("{r}={}", known.contains(crate::runtime::asset_id(&r))))
        .collect();
    let st = stamp(&[b"effect", &hash, refs.join("\n").as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Effect::NAME,
        st,
        Work::Job(Box::new(EffectJob(s))),
    ));
}

/// An effect's one output.
struct EffectJob<'a>(&'a Source);

impl Job for EffectJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let EffectJob(s) = self;
        let Exec { known, .. } = ex;
        cook_effect(s, &|id| known.contains(id), p.stamp)
    }
}

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&EMITTER, &EFFECT];
