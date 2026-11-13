//! Reading the world's rows for extraction.

use super::*;

/// The value of a resource the world holds, decoded with its layout (kept
/// while its bytes are unchanged: [`Layout::decode_kept`]).
pub(crate) fn resource(world: &World, layout: Option<&Layout>) -> Option<std::sync::Arc<Value>> {
    let layout = layout?;
    let id = world.id_by_name(&layout.schema).ok()?;
    let bytes = world.resource_bytes(id).ok()?;
    Some(layout.decode_kept(bytes))
}

/// Every row of one component, read in place from its archetypes' columns
/// in query order ([`column_rows`]).
pub(crate) struct ColumnRows<'a> {
    pub(super) layout: &'a Layout,
    /// Per archetype holding a row: its entities and their values.
    pub(super) archetypes: Vec<(&'a [Entity], Cells<'a>)>,
}

impl ColumnRows<'_> {
    pub(crate) fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }

    /// `(entity bits, row)` for every row, in query order.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (u64, ColRow<'_>)> + '_ {
        let layout = self.layout;
        self.archetypes.iter().flat_map(move |(entities, cells)| {
            entities
                .iter()
                .enumerate()
                .map(move |(i, e)| (e.to_bits(), ColRow::new(layout, cells, i)))
        })
    }
}

/// The rows of `layout`'s component, borrowed from the world's columns: no
/// row is copied or looked up by its entity (a packed copy of each row,
/// found through its entity, was most of reading 20,000 sprites or models).
/// An archetype one of whose columns a view holds mutably has no rows here,
/// as its rows had no bytes before.
pub(crate) fn column_rows<'a>(world: &'a World, layout: &'a Layout) -> ColumnRows<'a> {
    let mut archetypes = Vec::new();
    let q = QueryDesc {
        read: vec![layout.schema.clone()],
        ..QueryDesc::default()
    };
    if let (Ok(id), Ok(all)) = (world.id_by_name(&layout.schema), world.query(&q)) {
        for a in all {
            if a.is_empty() {
                continue;
            }
            let Ok(columns) = a.component_bytes(id) else {
                continue;
            };
            let fields = &layout.fields;
            let cells = if columns.len() == fields.len()
                && columns
                    .iter()
                    .zip(fields.iter())
                    .all(|(c, f)| c.len() == a.len() * f.ty.size())
            {
                Cells::Columns(
                    columns
                        .into_iter()
                        .zip(fields.iter())
                        .map(|(c, f)| Col::of(c, f))
                        .collect(),
                )
            } else {
                // Not the layout's fields: each row packed as the world
                // packs it, read by the layout's offsets as it was.
                let stride = columns.iter().map(|c| c.len()).sum::<usize>() / a.len();
                let mut bytes = Vec::with_capacity(stride * a.len());
                for row in 0..a.len() {
                    for c in &columns {
                        let n = c.len() / a.len();
                        bytes.extend_from_slice(&c[row * n..(row + 1) * n]);
                    }
                }
                Cells::Rows { bytes, stride }
            };
            archetypes.push((a.entities(), cells));
        }
    }
    ColumnRows { layout, archetypes }
}

/// Decoded rows of one component: `(entity bits, value)`.
pub(crate) fn rows(world: &World, layout: &Layout) -> Vec<(u64, Value)> {
    // The bytes are read serially (the world is one thread's), each row
    // packed into one buffer; decoding them is a lane per row natively, the
    // rows staying in entity order.
    let rows = column_rows(world, layout);
    let mut bytes = Vec::new();
    let cells: Vec<(u64, std::ops::Range<usize>)> = rows
        .iter()
        .map(|(e, row)| {
            let at = bytes.len();
            row.pack(&mut bytes);
            (e, at..bytes.len())
        })
        .collect();
    let bytes = bytes.as_slice();
    if cfg!(target_arch = "wasm32") {
        // A row's decode is allocations (a JSON map per row), and the lanes
        // wasm has one allocator behind one lock: 20,000 og.Mesh rows split
        // over eight Workers spent 116 ms of a 137 ms frame waiting for the
        // lanes (owed row 50, the former test sample
        return cells
            .iter()
            .map(|(e, at)| (*e, layout.decode(&bytes[at.clone()])))
            .collect();
    }
    og_lanes::map(og_lanes::Site::Extract, &cells, ROW_GRAIN, |(e, at)| {
        (*e, layout.decode(&bytes[at.clone()]))
    })
}

/// One `og.Mesh` as extraction reads it.
pub(crate) struct MeshRow {
    pub entity: u64,
    /// Indices into `RenderState::geometries` and `::materials`.
    pub geometry: usize,
    pub(super) material: usize,
    pub visible: bool,
    pub(super) flash: f32,
    pub(super) flash_color: Vec3,
    /// `materialLayer`: the texture-array layer its material reads (-1:
    /// the material's own).
    pub(super) layer: i32,
    /// `castShadow`: drawn into the shadow maps.
    pub(super) cast: bool,
    /// `receiveShadow`: darkened by the shadow maps.
    pub(super) receive: bool,
    /// `layers`: its three.js layer mask (`crate::present::views`).
    pub layers: u32,
}

/// A handle field's index, as the JSON rows read it (`as_f64`, then the
/// low 32 bits).
pub(super) fn handle_index(bits: u64) -> usize {
    ((bits as f64) as u64 & 0xffff_ffff) as usize
}

/// The `og.Mesh` rows in query order, read straight from the columns (a
/// mesh field decoded to JSON per row cost about a microsecond natively and
/// several on wasm: owed row 50); the JSON rows when the columns are not
/// the IDL's.
pub(crate) fn mesh_rows(world: &World, layout: &Layout) -> Vec<MeshRow> {
    let columnar = || -> Option<Vec<MeshRow>> {
        let id = world.id_by_name(&layout.schema).ok()?;
        let field = |n: &str| world.field_by_id(id, n).ok();
        let (g, m, v, f, c) = (
            field("geometry")?,
            field("material")?,
            field("visible")?,
            field("flash")?,
            field("flashColor")?,
        );
        let l = field("materialLayer")?;
        let s = field("castShadow")?;
        let r = field("receiveShadow")?;
        let ly = field("layers");
        let q = QueryDesc {
            read: vec![layout.schema.clone()],
            ..QueryDesc::default()
        };
        let mut out = Vec::new();
        for a in world.query(&q).ok()? {
            let (g, m, v, f, c) = (
                a.read::<u64>(g).ok()?,
                a.read::<u64>(m).ok()?,
                a.read::<og_core::reflect::Bool8>(v).ok()?,
                a.read::<f32>(f).ok()?,
                a.read::<Vec3>(c).ok()?,
            );
            let l = a.read::<i32>(l).ok()?;
            let s = a.read::<og_core::reflect::Bool8>(s).ok()?;
            let r = a.read::<og_core::reflect::Bool8>(r).ok()?;
            let ly = match ly {
                Some(ly) => Some(a.read::<u32>(ly).ok()?),
                None => None,
            };
            out.reserve(a.len());
            for (i, e) in a.entities().iter().enumerate() {
                out.push(MeshRow {
                    entity: e.to_bits(),
                    geometry: handle_index(g[i]),
                    material: handle_index(m[i]),
                    visible: v[i].get(),
                    flash: f[i],
                    flash_color: c[i],
                    layer: l[i],
                    cast: s[i].get(),
                    receive: r[i].get(),
                    layers: ly.as_ref().map_or(1, |ly| ly[i]),
                });
            }
        }
        Some(out)
    };
    if let Some(rows) = columnar() {
        return rows;
    }
    rows(world, layout)
        .into_iter()
        .filter_map(|(e, v)| {
            let v = crate::host::wire::record::Mesh::from_json(&v).ok()?;
            let index = |h: u64| (h & 0xffff_ffff) as usize;
            let c = &v.flash_color;
            Some(MeshRow {
                entity: e,
                geometry: index(v.geometry),
                material: index(v.material),
                visible: v.visible,
                flash: v.flash,
                flash_color: Vec3::new(c.r, c.g, c.b),
                layer: v.material_layer,
                cast: v.cast_shadow,
                receive: v.receive_shadow,
                layers: v.layers,
            })
        })
        .collect()
}

/// Rows per lane task when decoding a component's rows natively: a row decodes in
/// about 0.6 us, so a task is a few hundred microseconds of work and a
/// component splits from 1,024 rows (500 rows stay serial).
pub(super) const ROW_GRAIN: usize = 512;
