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

pub(crate) struct Archetype {
    /// Component types, sorted by id.
    pub types: Vec<SchemaId>,
    /// For each type, the index of its first column.
    pub first_column: Vec<usize>,
    pub columns: Vec<RefCell<Column>>,
    pub entities: Vec<Entity>,
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
            types,
            first_column,
            columns,
            entities: Vec::new(),
        }
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
        self.entities.swap_remove(row);
        self.entities.get(row).copied()
    }
}
