//! A `pack.json` style pack, cooked into its one blob.

use super::{
    BlobKind, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, format, stamp,
};
use crate::format::Area;

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&PACK];

pub(crate) static PACK: KindDef = KindDef {
    name: "pack",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan: plan_pack,
};

/// Whether `rel` is a style pack's `pack.json`.
pub(crate) fn is_pack(rel: &str) -> bool {
    rel.rsplit('/').next() == Some("pack.json") && rel.contains('/')
}

/// A `pack.json` read and its asset paths resolved against its directory.
pub(crate) fn pack_layer(s: &Source) -> Result<og_ui_next::pack::Layer, CookError> {
    let text =
        std::str::from_utf8(&s.bytes).map_err(|_| CookError(format!("{}: not UTF-8", s.rel)))?;
    let mut layer =
        og_ui_next::pack::Layer::parse(text).map_err(|e| CookError(format!("{}: {e}", s.rel)))?;
    layer.resolve(s.rel.rsplit_once('/').map_or("", |(dir, _)| dir));
    Ok(layer)
}

fn plan_pack<'a>(
    cx: &Cx<'a>,
    _: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let refs: Vec<String> = pack_layer(s)
        .map(|l| l.assets().into_iter().map(str::to_owned).collect())
        .unwrap_or_default();
    let refs: Vec<String> = refs
        .iter()
        .map(|r| format!("{r}={}", cx.known.contains(crate::runtime::asset_id(r))))
        .collect();
    // The default pack is in the stamp: a token change cooks every pack's
    // class check again.
    let st = stamp(
        &[Area::Ui],
        &[
            b"style pack 2",
            &hash,
            refs.join("\n").as_bytes(),
            og_ui_next::pack::design::DEFAULT_PACK.as_bytes(),
        ],
    );
    outs.push((s.rel.clone(), format::Pack::NAME, st, Box::new(PackJob(s))));
}

/// A `pack.json`'s one output: the pack checked (its form, every variant's
/// classes against its theme over the default in both schemes, its assets
/// present) and written with its paths resolved.
struct PackJob<'a>(&'a Source);

impl Job for PackJob<'_> {
    fn cook(&self, p: &Planned, ex: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let PackJob(s) = self;
        let layer = pack_layer(s)?;
        let missing: Vec<&str> = layer
            .assets()
            .into_iter()
            .filter(|r| !ex.known.contains(crate::runtime::asset_id(r)))
            .collect();
        if !missing.is_empty() {
            return Err(CookError(format!(
                "{}: assets are missing: {} (a path resolves beside pack.json; @styles/… names a built-in one)",
                s.rel,
                missing.join(", ")
            )));
        }
        let pack =
            og_ui_next::pack::Pack::new(&[og_ui_next::pack::Layer::default_pack(), layer.clone()]);
        let mut errors = Vec::new();
        for scheme in og_ui_next::pack::Scheme::ALL {
            let (mut classes, ignored) =
                og_ui_next::classes::Classes::new(Some(&pack.theme_json(scheme)))
                    .map_err(CookError)?;
            for e in ignored {
                errors.push(format!("theme ({}): {e}", scheme.name()));
            }
            for (key, class) in &layer.variants {
                // `nine-<name>` names the pack's nine-slice art, not a utility.
                let (images, class) = og_ui_next::pack::split_nine(class);
                for name in images {
                    if !pack.images(scheme).contains_key(name) {
                        errors.push(format!(
                            "variants.{key}: `nine-{name}` names no image in `images` ({})",
                            scheme.name()
                        ));
                    }
                }
                for u in classes.unknown(&class) {
                    errors.push(format!(
                        "variants.{key}: `{u}` is not a class ({})",
                        scheme.name()
                    ));
                }
            }
        }
        errors.dedup();
        if !errors.is_empty() {
            return Err(CookError(format!("{}: {}", s.rel, errors.join("; "))));
        }
        let text = layer.to_json().into_bytes();
        let mut w = og_blob::BlobWriter::new(
            format::Pack::type_hash(),
            format::Pack::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::PackMeta {
                text_bytes: text.len() as u32,
            }],
            format::row_hash::<format::PackMeta>(),
        )
        .raw("text", text, 16);
        Ok((
            w.finish(),
            vec![format!(
                "style pack: {} variant(s), {} image(s), {} face(s), {} asset(s)",
                layer.variants.len(),
                layer.images[0].len().max(layer.images[1].len()),
                layer.fonts.len(),
                layer.assets().len()
            )],
        ))
    }
}
