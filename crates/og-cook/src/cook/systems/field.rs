//! Sampled fields share one validated source and lossless cooked layout.
use super::*;
use og_blob::BlobWriter;
use og_bundle::field::FieldData;
use std::sync::Arc;

pub(crate) struct Prepared {
    pub data: Arc<FieldData>,
    pub hash: u64,
}

pub(crate) type Fields = BTreeMap<String, Prepared>;

impl Prepared {
    pub fn new(data: FieldData) -> Result<Self, String> {
        data.validate()?;
        let mut hash = og_core::hash::StableHasher::new();
        for value in data.dimensions.into_iter().chain([data.channels]) {
            hash.write_u32(value);
        }
        for value in data.origin.iter().chain(&data.spacing).chain(&data.values) {
            hash.write_u32(value.to_bits());
        }
        Ok(Self {
            data: Arc::new(data),
            hash: hash.finish(),
        })
    }
}

/// Consumers share the checked value used by the field writer, not a second
/// evaluation or a file read during execution.
pub(crate) fn bind(sources: &mut [Source], fields: Fields) {
    let fields = Arc::new(fields);
    for s in sources {
        s.fields = fields.clone();
    }
}

pub(crate) fn get<'a>(fields: &'a Fields, id: &str) -> Result<&'a Prepared, String> {
    fields.get(id).ok_or_else(|| format!(
        "field '{id}' is not a checked project field; declare file.field under content/ and use its canonical module/content ID"
    ))
}

pub(crate) static FIELD: KindDef = KindDef {
    name: "field",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&FIELD];

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
        let data = &get(&self.0.fields, &self.0.rel).map_err(CookError)?.data;
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
