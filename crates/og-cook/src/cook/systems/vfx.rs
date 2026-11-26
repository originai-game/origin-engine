//! Composed effects (`type vfx`, `og_bundle::vfx`): the cook checks the
//! file with the runtime's parser and that every emitter, effect material,
//! model, sound, flipbook and texture it names is an asset, and stores the text,
//! which the runtime parses again.

use super::{CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, stamp};
use crate::format::{self, Area, BlobKind};
use og_blob::BlobWriter;

pub(crate) fn cook_vfx(
    s: &Source,
    known: &dyn Fn(&str) -> bool,
    stamp: u64,
) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError("a vfx is text (UTF-8)".into()))?;
    let v = og_bundle::vfx::parse_vfx(text).map_err(CookError)?;
    let r = v.references();
    let mut missing = Vec::new();
    for (what, ids) in [
        ("emitter", &r.emitters),
        ("effect material", &r.effects),
        ("model", &r.models),
        ("sound", &r.sounds),
        ("flipbook", &r.flipbooks),
        ("texture", &r.textures),
    ] {
        for id in ids {
            if !known(crate::runtime::asset_id(id)) {
                missing.push(format!("{what} {id}"));
            }
        }
    }
    if !missing.is_empty() {
        return Err(CookError(format!(
            "the vfx names asset(s) not under assets/: {}",
            missing.join(", ")
        )));
    }
    let mut w = BlobWriter::new(
        format::Vfx::type_hash(),
        format::Vfx::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::VfxMeta {
            text_bytes: text.len() as u32,
        }],
        format::row_hash::<format::VfxMeta>(),
    )
    .raw("text", text.as_bytes().to_vec(), 16);
    let mut kinds: Vec<&str> = v.elements.iter().map(|e| e.kind.name()).collect();
    kinds.dedup();
    Ok((
        w.finish(),
        vec![format!(
            "vfx: {} s, {} elements ({}), {} events",
            v.duration,
            v.elements.len(),
            kinds.join(", "),
            v.events.len()
        )],
    ))
}

pub(crate) static VFX: KindDef = KindDef {
    name: "vfx",
    extensions: &["efkproj", "efkefc"],
    types: &[og_bundle::vfx::VFX_TYPE],
    by_content: false,
    streams: false,
    plan: plan_vfx,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&VFX];

fn plan_vfx<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let Cx { known, .. } = *cx;
    let text = String::from_utf8_lossy(&s.bytes);
    let st = if let Ok(v) = og_bundle::vfx::parse_vfx(&text) {
        let r = v.references();
        let entry = state.parsed_refs.entry(s.rel.clone()).or_default();
        let mut lines = Vec::new();
        for id in [
            r.emitters,
            r.effects,
            r.models,
            r.sounds,
            r.flipbooks,
            r.textures,
        ]
        .concat()
        {
            entry.insert(id.clone());
            lines.push(format!(
                "{id}={}",
                known.contains(crate::runtime::asset_id(&id))
            ));
        }
        lines.join("\n")
    } else {
        String::new()
    };
    let st = stamp(&[Area::Effect], &[b"vfx", &hash, st.as_bytes()]);
    outs.push((s.rel.clone(), format::Vfx::NAME, st, Box::new(VfxJob(s))));
}

/// A VFX graph's one output.
struct VfxJob<'a>(&'a Source);

impl Job for VfxJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let VfxJob(s) = self;
        let Exec { known, .. } = ex;
        cook_vfx(s, &|id| known.contains(id), p.stamp)
    }
}
