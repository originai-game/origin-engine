//! The rows kept from one gather to the next, and reading a component's
//! rows out of an archetype.

use super::*;

/// Every row of component `T` in `a`, decoded from its columns, by ranges
/// of rows on the lanes from two grains of them (`Grains::rows`), the
/// ranges' rows joined in row order.
pub(super) fn read_rows<T: Reflect + Send>(
    a: &ArchetypeView<'_>,
    fields: &[FieldRef],
    layout: &Layout,
) -> Result<Vec<T>> {
    let columns = fields
        .iter()
        .map(|f| a.read_bytes(*f))
        .collect::<Result<Vec<_>>>()?;
    let columns: Vec<&[u8]> = columns.iter().map(|c| &**c).collect();
    let decode = |rows: std::ops::Range<usize>| -> Result<Vec<T>> {
        let mut packed = vec![0u8; layout.size()];
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            for (i, column) in columns.iter().enumerate() {
                let range = layout.range(i);
                let n = range.len();
                packed[range].copy_from_slice(&column[row * n..(row + 1) * n]);
            }
            out.push(layout.decode::<T>(&packed)?);
        }
        Ok(out)
    };
    let mut parts = crate::lanes::map_ranges(a.len(), crate::lanes::now().rows, decode);
    if parts.len() == 1 {
        return parts.pop().unwrap_or_else(|| Ok(Vec::new()));
    }
    let mut out = Vec::with_capacity(a.len());
    for part in parts {
        out.extend(part?);
    }
    Ok(out)
}

/// The rows of one component in one archetype as a gather decoded them,
/// with the write stamp of their columns then ([`ArchetypeView::written`]).
pub(super) struct KeptRows<T> {
    pub(super) written: Option<u64>,
    pub(super) rows: Vec<T>,
}

impl<T> Default for KeptRows<T> {
    fn default() -> Self {
        KeptRows {
            written: None,
            rows: Vec::new(),
        }
    }
}

/// The colliders and bodies a gather decoded, by query and by archetype in
/// the query's order, for the next gather to reuse where nothing wrote the
/// component's columns since: most colliders of a level never change, and
/// decoding them was most of the gather. Only what a row decodes to is
/// kept; the scene is put together from it every time, in the same order,
/// so a scene made with rows kept is the scene made without. Not world
/// state: a restore stamps every column again, so nothing kept outlives it.
///
/// For gathers inside a system only (a write through a view taken outside
/// one is not stamped when it lands, see [`ArchetypeView::written`]); the
/// gather outside the tick starts from an empty one.
#[derive(Default)]
pub(crate) struct Kept {
    pub(super) bodies: Vec<KeptRows<RigidBody>>,
    pub(super) body_colliders: Vec<KeptRows<Collider>>,
    pub(super) static_colliders: Vec<KeptRows<Collider>>,
    /// Q_CHARACTERS' archetypes, then Q_PLATFORMERS'.
    pub(super) controller_colliders: Vec<KeptRows<Collider>>,
}

impl Kept {
    /// Every row of component `id` (`fields`, `layout`) in `a`, archetype
    /// `at` of its query: the rows kept when the columns have the stamp
    /// they had when those were decoded, decoded and kept otherwise.
    pub(super) fn rows<'k, T: Reflect + Send>(
        kept: &'k mut Vec<KeptRows<T>>,
        at: usize,
        a: &ArchetypeView<'_>,
        id: SchemaId,
        fields: &[FieldRef],
        layout: &Layout,
    ) -> Result<&'k [T]> {
        if kept.len() <= at {
            kept.resize_with(at + 1, KeptRows::default);
        }
        let slot = &mut kept[at];
        let written = a.written(id);
        if written.is_none() || slot.written != written || slot.rows.len() != a.len() {
            // Unstamped first: a decode that fails keeps nothing.
            slot.written = None;
            slot.rows = read_rows(a, fields, layout)?;
            slot.written = written;
        }
        Ok(&slot.rows)
    }
}

/// A sparse map by entity index, valid for one tick (live entities have
/// distinct indices).
pub(super) struct ByEntity<T>(Vec<Option<T>>);

impl<T: Copy> ByEntity<T> {
    pub(super) fn new() -> Self {
        ByEntity(Vec::new())
    }
    pub(super) fn insert(&mut self, e: Entity, value: T) {
        let i = e.index() as usize;
        if i >= self.0.len() {
            self.0.resize(i + 1, None);
        }
        self.0[i] = Some(value);
    }
    pub(super) fn get(&self, e: Entity) -> Option<T> {
        self.0.get(e.index() as usize).copied().flatten()
    }
}
