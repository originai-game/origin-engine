//! Checked definitions compile to native fields with canonical subtree inputs.
use super::*;
use og_blob::BlobWriter;

pub(crate) static BEHAVIOUR: KindDef = KindDef {
    name: "behaviour",
    extensions: &[],
    types: &[],
    by_content: false,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&BEHAVIOUR];

fn compile(
    source: &Source,
    sources: &[Source],
    chain: &mut Vec<String>,
    deps: &mut Vec<u8>,
) -> Result<og_ai::Behaviour, String> {
    if chain.contains(&source.rel) {
        return Err(format!(
            "behaviour subtree cycle: {} -> {}",
            chain.join(" -> "),
            source.rel
        ));
    }
    chain.push(source.rel.clone());
    deps.extend_from_slice(&source.hash.to_le_bytes());
    let input: source_schemas::media::behaviour::Behaviour = source.input()?;
    let result = og_ai::Behaviour::from_definition(input, &mut |reference| {
        let sub = sources
            .iter()
            .find(|s| s.rel == reference.id && s.kind == &BEHAVIOUR)
            .ok_or_else(|| format!("'{}' is not a behaviour input", reference.id))?;
        Ok((sub.rel.clone(), compile(sub, sources, chain, deps)?))
    });
    chain.pop();
    result
}

fn plan<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let result = (|| -> Result<_, String> {
        let mut deps = Vec::new();
        let b = compile(s, cx.sources, &mut Vec::new(), &mut deps)?;
        let data = b.to_bytes()?;
        let st = stamp(&[Area::Data], &[b"behaviour", &hash, &deps, &data]);
        Ok((st, Cook(data)))
    })();
    match result {
        Ok((st, job)) => outs.push((s.rel.clone(), format::Behaviour::NAME, st, Box::new(job))),
        Err(error) => state.early_errors.push((
            s.rel.clone(),
            "behaviour",
            format!("{}: {error}", s.path.display()),
        )),
    }
}

struct Cook(Vec<u8>);
impl Job for Cook {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let mut w = BlobWriter::new(
            format::Behaviour::type_hash(),
            format::Behaviour::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::BehaviourMeta {
                data_bytes: self.0.len() as u32,
            }],
            format::row_hash::<format::BehaviourMeta>(),
        )
        .raw("data", self.0.clone(), 16);
        Ok((w.finish(), Vec::new()))
    }
    fn action(&self) -> Action<'_> {
        Action::Input(&self.0, false)
    }
}
