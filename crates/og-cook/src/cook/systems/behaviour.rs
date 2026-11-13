//! Behaviours (`type behaviour`): validated by og-ai and stored as text.

use super::{
    CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, Work, assets_root, stamp,
};
use crate::format::{self, BlobKind};
use og_blob::BlobWriter;

pub(crate) const BEHAVIOUR_TYPE: &str = "behaviour";

/// Parses a behaviour source, reading its `use` subtrees from the assets
/// root; `read` sees every subtree file (its path and bytes) as it is read.
fn parse_behaviour(
    s: &Source,
    text: &str,
    read: &mut dyn FnMut(&str, &[u8]),
) -> Result<og_ai::Behaviour, String> {
    let root = assets_root(s);
    let mut resolve = |p: &str| -> Result<String, String> {
        let id = crate::runtime::asset_id(p);
        let bytes = std::fs::read(root.join(id)).map_err(|e| format!("cannot read it: {e}"))?;
        read(id, &bytes);
        String::from_utf8(bytes).map_err(|_| "a behaviour is text (UTF-8)".to_string())
    };
    og_ai::Behaviour::parse_with(text, crate::runtime::asset_id(&s.rel), &mut resolve)
}

/// What a behaviour's stamp covers beyond its own bytes: every subtree
/// file its `use` nodes reach, by path and hash (so editing a subtree
/// re-validates the trees that use it).
pub(crate) fn behaviour_dependencies(s: &Source) -> String {
    let mut deps = Vec::new();
    let text = String::from_utf8_lossy(&s.bytes);
    let _ = parse_behaviour(s, &text, &mut |p, b| {
        deps.push(format!("{p}={:016x}", super::hash_bytes(b)));
    });
    deps.join("\n")
}

pub(crate) fn cook_behaviour(s: &Source, stamp: u64) -> Result<(Vec<u8>, Vec<String>), CookError> {
    let text = std::str::from_utf8(&s.bytes)
        .map_err(|_| CookError("a behaviour is text (UTF-8)".into()))?;
    let b = parse_behaviour(s, text, &mut |_, _| {}).map_err(CookError)?;
    let mut w = BlobWriter::new(
        format::Behaviour::type_hash(),
        format::Behaviour::schema_version(),
        stamp,
    );
    w.table(
        "meta",
        &[format::BehaviourMeta {
            text_bytes: text.len() as u32,
        }],
        format::row_hash::<format::BehaviourMeta>(),
    )
    .raw("text", text.as_bytes().to_vec(), 16);
    let kind = match &b.root {
        og_ai::Root::Tree(_) => "a tree".to_string(),
        og_ai::Root::Machine { states, .. } => format!("a machine of {} states", states.len()),
    };
    Ok((
        w.finish(),
        vec![if b.subtrees.is_empty() {
            format!("behaviour '{}': {kind}, {} nodes", b.name, b.nodes.len())
        } else {
            format!(
                "behaviour '{}': {kind}, {} nodes, using {}",
                b.name,
                b.nodes.len(),
                b.subtrees.join(", ")
            )
        }],
    ))
}

pub(crate) static BEHAVIOUR: KindDef = KindDef {
    name: "behaviour",
    extensions: &[],
    types: &[BEHAVIOUR_TYPE],
    by_content: false,
    streams: false,
    plan,
};

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&BEHAVIOUR];

fn plan<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    // The stamp covers every subtree its `use` nodes reach.
    let deps = behaviour_dependencies(s);
    let st = stamp(&[b"behaviour", &hash, deps.as_bytes()]);
    outs.push((
        s.rel.clone(),
        format::Behaviour::NAME,
        st,
        Work::Job(Box::new(Cook(s))),
    ));
}

/// A behaviour's one output.
struct Cook<'a>(&'a Source);

impl Job for Cook<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        cook_behaviour(self.0, p.stamp)
    }
}
