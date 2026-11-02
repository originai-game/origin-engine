//! Archetype storage: one archetype per distinct set of component types;
//! inside it, one column per (component, field), structure of arrays.
//!
//! Each column is a 16-byte-aligned byte buffer of `rows × field size`, so a
//! field can be viewed as `&[f32]`, `&[Vec3]`, `&[Quat]`... (and, on the web,
//! as a `Float32Array` over wasm memory) without copying.

use crate::entity::Entity;
use crate::registry::{Registry, SchemaId};
use std::cell::RefCell;

#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C, align(16))]
struct Block([u8; 16]);

/// One field's values for every row of an archetype.
pub(crate) struct Column {
    size: usize,
    rows: usize,
    blocks: Vec<Block>,
}

impl Column {
    pub fn new(size: usize) -> Column {
        Column {
            size,
            rows: 0,
            blocks: Vec::new(),
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &bytemuck::cast_slice(&self.blocks)[..self.rows * self.size]
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        let len = self.rows * self.size;
        &mut bytemuck::cast_slice_mut(&mut self.blocks)[..len]
    }

    pub fn row(&self, row: usize) -> &[u8] {
        &self.bytes()[row * self.size..(row + 1) * self.size]
    }

    pub fn set_row(&mut self, row: usize, value: &[u8]) {
        let size = self.size;
        self.bytes_mut()[row * size..(row + 1) * size].copy_from_slice(value);
    }

    fn resize_rows(&mut self, rows: usize) {
        self.rows = rows;
        self.blocks
            .resize((rows * self.size).div_ceil(16), Block([0; 16]));
    }

    pub fn push(&mut self, value: &[u8]) {
        debug_assert_eq!(value.len(), self.size);
        let row = self.rows;
        self.resize_rows(row + 1);
        self.set_row(row, value);
    }

    pub fn swap_remove(&mut self, row: usize) {
        let last = self.rows - 1;
        if row != last {
            let size = self.size;
            self.bytes_mut()
                .copy_within(last * size..(last + 1) * size, row * size);
        }
        self.resize_rows(last);
    }

    /// Replaces every row with `bytes` (`rows × size` long).
    pub fn fill_from(&mut self, rows: usize, bytes: &[u8]) {
        self.resize_rows(rows);
        self.bytes_mut().copy_from_slice(bytes);
    }

    /// Appends `rows` rows from `bytes` (`rows × size` long).
    pub fn extend_from(&mut self, rows: usize, bytes: &[u8]) {
        let start = self.rows * self.size;
        self.resize_rows(self.rows + rows);
        self.bytes_mut()[start..].copy_from_slice(bytes);
    }
}

/// Change ticks of one tracked component in one archetype, one entry per
/// row ([`crate::World`]'s change detection): the change tick it was added
/// at, the change tick its value last changed at, and the value as it was
/// when last compared (per field, the column's bytes), which the next
/// comparison reads a write against.
#[derive(Clone, Debug, Default)]
pub(crate) struct Ticks {
    pub added: Vec<u64>,
    pub changed: Vec<u64>,
    pub shadow: Vec<Vec<u8>>,
}

/// A column's field size, whether or not it is borrowed (the size never
/// changes).
fn column_size(c: &RefCell<Column>) -> usize {
    // SAFETY: `size` is written once, at creation; reading it through the
    // pointer races with nothing, borrowed or not.
    unsafe { (*c.as_ptr()).size }
}

/// A row's change ticks as it moves between archetypes: added, changed,
/// and the value as last compared (its fields' bytes in order).
pub(crate) type RowTicks = (u64, u64, Vec<u8>);

pub(crate) struct Archetype {
    /// Component types, sorted by id.
    pub types: Vec<SchemaId>,
    /// For each type, the index of its first column.
    pub first_column: Vec<usize>,
    pub columns: Vec<RefCell<Column>>,
    pub entities: Vec<Entity>,
    /// For each type, its change ticks when the world tracks it.
    pub ticks: Vec<Option<Ticks>>,
}

impl Archetype {
    pub fn new(types: Vec<SchemaId>, registry: &Registry) -> Archetype {
        debug_assert!(types.windows(2).all(|w| w[0] < w[1]));
        let mut first_column = Vec::with_capacity(types.len());
        let mut columns = Vec::new();
        for &id in &types {
            first_column.push(columns.len());
            let layout = &registry.info(id).layout;
            for f in 0..layout.field_count() {
                columns.push(RefCell::new(Column::new(layout.range(f).len())));
            }
        }
        Archetype {
            ticks: vec![None; types.len()],
            types,
            first_column,
            columns,
            entities: Vec::new(),
        }
    }

    /// Starts tracking the type at `position`: every row present counts as
    /// added and changed at tick 0 (before any system ran), its value as
    /// compared.
    pub fn track(&mut self, position: usize) {
        if self.ticks[position].is_some() {
            return;
        }
        let rows = self.len();
        let shadow = self
            .columns_of(position)
            .map(|c| self.columns[c].get_mut().bytes().to_vec())
            .collect();
        self.ticks[position] = Some(Ticks {
            added: vec![0; rows],
            changed: vec![0; rows],
            shadow,
        });
    }

    /// Appends the change ticks of a row just pushed at the end of the
    /// columns of the type at `position` (if tracked): `from` the row's
    /// ticks in the archetype it came from, or new at `stamp` (added and
    /// changed then, its value as compared).
    pub fn push_ticks(&mut self, position: usize, from: Option<RowTicks>, stamp: u64) {
        if self.ticks[position].is_none() {
            return;
        }
        let row = self.len() - 1;
        let range = self.columns_of(position);
        let sizes: Vec<usize> = range
            .clone()
            .map(|c| self.columns[c].get_mut().size)
            .collect();
        let shadow: Vec<u8> = match &from {
            Some((_, _, shadow)) => shadow.clone(),
            None => range
                .flat_map(|c| self.columns[c].get_mut().row(row).to_vec())
                .collect(),
        };
        let t = self.ticks[position].as_mut().expect("tracked");
        let (added, changed) = from.map_or((stamp, stamp), |(a, c, _)| (a, c));
        t.added.push(added);
        t.changed.push(changed);
        let mut at = 0;
        for (s, size) in t.shadow.iter_mut().zip(sizes) {
            s.extend_from_slice(&shadow[at..at + size]);
            at += size;
        }
    }

    /// The change ticks of `row` of the type at `position`, to carry to
    /// another archetype; `None` when it is not tracked.
    pub fn take_row_ticks(&self, position: usize, row: usize) -> Option<RowTicks> {
        let t = self.ticks[position].as_ref()?;
        Some((t.added[row], t.changed[row], self.shadow_row(position, row)))
    }

    /// The (added, changed) ticks of `row` of the type at `position`.
    pub fn row_ticks(&self, position: usize, row: usize) -> Option<(u64, u64)> {
        self.ticks[position]
            .as_ref()
            .map(|t| (t.added[row], t.changed[row]))
    }

    /// Compares every row of the tracked type at `position` with its last
    /// compared value: a row whose value differs changed at `stamp`.
    /// Returns how many rows changed.
    pub fn diff(&mut self, position: usize, stamp: u64) -> usize {
        let range = self.columns_of(position);
        let Some(t) = &mut self.ticks[position] else {
            return 0;
        };
        let mut changed = 0;
        for (k, c) in range.enumerate() {
            let col = self.columns[c].get_mut();
            let size = col.size;
            if size == 0 {
                continue;
            }
            let now = col.bytes();
            let shadow = &mut t.shadow[k];
            if now == shadow.as_slice() {
                continue;
            }
            for (row, (a, b)) in now
                .chunks_exact(size)
                .zip(shadow.chunks_exact_mut(size))
                .enumerate()
            {
                if a != b {
                    b.copy_from_slice(a);
                    if t.changed[row] != stamp {
                        t.changed[row] = stamp;
                        changed += 1;
                    }
                }
            }
        }
        changed
    }

    /// [`Self::diff`] of one row.
    pub fn diff_row(&mut self, position: usize, row: usize, stamp: u64) {
        let range = self.columns_of(position);
        let Some(t) = &mut self.ticks[position] else {
            return;
        };
        for (k, c) in range.enumerate() {
            let col = self.columns[c].get_mut();
            let size = col.size;
            let now = col.row(row);
            let was = &mut t.shadow[k][row * size..(row + 1) * size];
            if now != was {
                was.copy_from_slice(now);
                t.changed[row] = stamp;
            }
        }
    }

    /// Whether `row` of the tracked type at `position` differs from its last
    /// compared value (a write no comparison has seen yet). `unguarded`: a
    /// column the running system holds is read in place, as the snapshot
    /// writer reads it (its reasoning holds here).
    pub fn dirty(&self, position: usize, row: usize, unguarded: bool) -> bool {
        let Some(t) = &self.ticks[position] else {
            return false;
        };
        self.columns_of(position).enumerate().any(|(k, c)| {
            let size = column_size(&self.columns[c]);
            match self.columns[c].try_borrow() {
                Ok(col) => col.row(row) != &t.shadow[k][row * size..(row + 1) * size],
                // SAFETY: see `World::write_columns`: only the running
                // system holds a column mutably, blocked in this call.
                Err(_) if unguarded => {
                    unsafe { &*self.columns[c].as_ptr() }.row(row)
                        != &t.shadow[k][row * size..(row + 1) * size]
                }
                Err(_) => panic!("a column is borrowed outside a system"),
            }
        })
    }

    /// The shadow bytes (the value as last compared) of `row` of the
    /// tracked type at `position`, its fields in order.
    pub fn shadow_row(&self, position: usize, row: usize) -> Vec<u8> {
        let mut out = Vec::new();
        if let Some(t) = &self.ticks[position] {
            for (k, c) in self.columns_of(position).enumerate() {
                let size = column_size(&self.columns[c]);
                out.extend_from_slice(&t.shadow[k][row * size..(row + 1) * size]);
            }
        }
        out
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn position(&self, id: SchemaId) -> Option<usize> {
        self.types.binary_search(&id).ok()
    }

    pub fn has(&self, id: SchemaId) -> bool {
        self.position(id).is_some()
    }

    /// Columns of the type at `position` in [`Self::types`].
    pub fn columns_of(&self, position: usize) -> std::ops::Range<usize> {
        let start = self.first_column[position];
        let end = self
            .first_column
            .get(position + 1)
            .copied()
            .unwrap_or(self.columns.len());
        start..end
    }

    pub fn column_index(&self, id: SchemaId, field: u32) -> Option<usize> {
        let range = self.columns_of(self.position(id)?);
        let index = range.start + field as usize;
        (index < range.end).then_some(index)
    }

    /// Packed bytes of component `id` at `row`.
    pub fn read_packed(&self, position: usize, row: usize, out: &mut Vec<u8>) -> bool {
        for c in self.columns_of(position) {
            match self.columns[c].try_borrow() {
                Ok(col) => out.extend_from_slice(col.row(row)),
                Err(_) => return false,
            }
        }
        true
    }

    /// Removes `row`; returns the entity that moved into it, if any.
    pub fn swap_remove(&mut self, row: usize) -> Option<Entity> {
        for c in &mut self.columns {
            c.get_mut().swap_remove(row);
        }
        for (position, t) in self.ticks.iter_mut().enumerate() {
            let Some(t) = t else { continue };
            let last = t.added.len() - 1;
            t.added.swap_remove(row);
            t.changed.swap_remove(row);
            let start = self.first_column[position];
            for (k, s) in t.shadow.iter_mut().enumerate() {
                let size = self.columns[start + k].get_mut().size;
                if row != last {
                    s.copy_within(last * size..(last + 1) * size, row * size);
                }
                s.truncate(last * size);
            }
        }
        self.entities.swap_remove(row);
        self.entities.get(row).copied()
    }
}
