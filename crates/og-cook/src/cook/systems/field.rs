//! Sampled fields share one validated source and lossless cooked layout.
use super::*;
use og_blob::BlobWriter;
use og_bundle::field::FieldData;

pub(crate) static FIELD: KindDef = KindDef {
    name: "field",
    extensions: &[],
    types: &["field"],
    by_content: true,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&FIELD];

pub(crate) fn source(root: &Path, rel: &str, bytes: &[u8]) -> Result<FieldData, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != "field" || doc.version != 1 {
        return Err("a field is type field version 1".into());
    }
    let entries: Vec<_> = doc.body.get_all("generate").collect();
    if entries.is_empty() {
        return FieldData::parse(text);
    }
    if entries.len() != 1 || doc.body.entries.len() != 1 || !entries[0].values.is_empty() {
        return Err("field.generate is exclusive with authored lattice data".into());
    }
    let spec =
        generator::Spec::standalone(entries[0].block.as_ref().ok_or("generate is a block")?)?;
    let data: FieldData =
        serde_json::from_value(generator::Program::prepare(root, rel, &spec)?.run()?)
            .map_err(|e| e.to_string())?;
    data.validate()?;
    Ok(data)
}

pub(crate) fn expand(sources: &mut [Source]) {
    for s in sources {
        if s.kind != &FIELD || s.error.is_some() {
            continue;
        }
        match source(&assets_root(s), &s.rel, &s.bytes).and_then(|data| data.to_text()) {
            Ok(text) => s.bytes = text.into_bytes(),
            Err(e) => s.error = Some(e),
        }
    }
}

fn plan<'a>(_: &Cx<'a>, _: &mut PlanState, s: &'a Source, hash: [u8; 8], outs: &mut Vec<Out<'a>>) {
    let st = stamp(&[Area::Data], &[b"field", &hash]);
    outs.push((
        s.rel.clone(),
        format::Field::NAME,
        st,
        Box::new(FieldJob(s)),
    ));
}

struct FieldJob<'a>(&'a Source);
impl Job for FieldJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let data = if self.0.path.extension().is_some_and(|ext| ext == "ts") {
            let data =
                generator::value::Data::from_bytes(self.0.bytes.clone()).map_err(CookError)?;
            let mut envelope = data.root().into_json();
            serde_json::from_value::<FieldData>(envelope["value"].take())
                .map_err(|e| CookError(e.to_string()))?
        } else {
            let text = std::str::from_utf8(&self.0.bytes).map_err(|e| CookError(e.to_string()))?;
            FieldData::parse(text).map_err(CookError)?
        };
        data.validate().map_err(CookError)?;
        let mut w = BlobWriter::new(
            format::Field::type_hash(),
            format::Field::schema_version(),
            p.stamp,
        );
        let [width, height, depth] = data.dimensions;
        let [origin_x, origin_y, origin_z] = data.origin;
        let [spacing_x, spacing_y, spacing_z] = data.spacing;
        w.table(
            "meta",
            &[format::FieldMeta {
                width,
                height,
                depth,
                channels: data.channels,
                origin_x,
                origin_y,
                origin_z,
                spacing_x,
                spacing_y,
                spacing_z,
            }],
            format::row_hash::<format::FieldMeta>(),
        );
        let values: Vec<u8> = data.values.iter().flat_map(|v| v.to_le_bytes()).collect();
        w.raw("values", values, 4);
        Ok((
            w.finish(),
            vec![format!(
                "field {width}×{height}×{depth}, {} channels",
                data.channels
            )],
        ))
    }
    fn action(&self) -> Action<'_> {
        Action::Input(&self.0.bytes, false)
    }
}
