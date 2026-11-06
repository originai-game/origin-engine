//! Composed effects (`type vfx`, `og_bundle::vfx`): the cook checks the
//! file with the runtime's parser and that every emitter, effect material,
//! model and sound it names is an asset, and stores the text,
//! which the runtime parses again.

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;

/// Every asset id the file names with whether it is known, so the stamp
/// moves when one appears or goes (an unparsable file stamps on its bytes).
pub(super) fn stamp_inputs(s: &Source, known: &dyn Fn(&str) -> bool) -> String {
    let text = String::from_utf8_lossy(&s.bytes);
    let Ok(v) = og_bundle::vfx::parse_vfx(&text) else {
        return String::new();
    };
    let r = v.references();
    [r.emitters, r.effects, r.models, r.sounds]
        .concat()
        .iter()
        .map(|id| format!("{id}={}", known(crate::runtime::asset_id(id))))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn cook_vfx(
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
