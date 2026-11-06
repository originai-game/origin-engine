//! Effect materials (`type effect`, `og_bundle::effect`): the cook checks
//! the file with the runtime's parser and that every texture and shader it
//! names is an asset, and stores the text, which the runtime parses again
//! with the bundle's handles.

use super::{CookError, Source};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;

pub(super) fn cook_effect(
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
