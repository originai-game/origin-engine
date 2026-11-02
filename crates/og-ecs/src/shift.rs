//! Moving the world's zero (R12, og.WorldOrigin, `docs/design.md` §2.4):
//! every coordinate that is a place in the world is moved by the same
//! amount, so a world far from its zero is brought back near it with
//! nothing in it seeing a change but the numbers.
//!
//! What moves is declared once, in the reflected schema
//! ([`og_core::Worldspace`], the IDL's `worldspace` attribute): the caller
//! resolves the fields ([`ShiftField`]) and [`World::shift`] moves them in
//! every component column (with the change-detection shadow, so a shift is
//! not a change), resource and queued event, then asks every state block
//! to move what it keeps ([`crate::StateBlock::shift`]).

use crate::World;
use crate::registry::SchemaId;
use crate::world::ResourceValue;
use og_core::glam::DVec3;
use og_core::{FieldType, Result, Worldspace};

/// One field a [`World::shift`] moves: a field of a registered schema and
/// how its lanes lie in the world.
#[derive(Clone, Copy, Debug)]
pub struct ShiftField {
    pub schema: SchemaId,
    /// The field's index in the schema.
    pub field: usize,
    pub space: Worldspace,
}

/// One shift as a state block sees it ([`crate::StateBlock::shift`]).
pub struct Shift {
    /// How far every place in the world moves.
    pub d: DVec3,
    /// The entities with og.Parent, by bits, sorted.
    children: Vec<u64>,
    /// The moved fields by schema index and packed byte range, sorted: what
    /// a block that writes into components by offset (og.tween) reads.
    fields: Vec<(u32, u32, u32, Worldspace, FieldType)>,
}

impl Shift {
    /// A shift by `d` of a world whose children (by bits) are `children`.
    pub fn new(d: DVec3, mut children: Vec<u64>) -> Shift {
        children.sort_unstable();
        Shift {
            d,
            children,
            fields: Vec::new(),
        }
    }

    /// How far the lane at packed byte `byte` of component `schema` (its
    /// id's index) on entity `bits` moves: along its field's axis for that
    /// lane when the field moves on that entity, else 0.
    pub fn lane(&self, bits: u64, schema: u32, byte: u32) -> f64 {
        let i = self.fields.partition_point(|f| (f.0, f.2) <= (schema, byte));
        let Some(&(s, start, end, space, ty)) = self.fields.get(i) else {
            return 0.0;
        };
        if s != schema || byte < start || byte >= end || !space.moves(self.is_child(bits)) {
            return 0.0;
        }
        let width = if ty == FieldType::F64 { 8 } else { 4 };
        let lane = ((byte - start) / width) as usize;
        if lane >= space.lanes as usize {
            return 0.0;
        }
        self.d.to_array()[space.axes[lane] as usize]
    }

    /// Whether entity `bits` has a parent.
    pub fn is_child(&self, bits: u64) -> bool {
        self.children.binary_search(&bits).is_ok()
    }

    /// How far a value in entity `bits`'s Transform frame moves: `d` on a
    /// root, nothing on a child (its parent's frame moved with the parent).
    pub fn of(&self, bits: u64) -> DVec3 {
        if self.is_child(bits) { DVec3::ZERO } else { self.d }
    }

    /// `d` as f32 (exact: a shift is a multiple of a power of two).
    pub fn d32(&self) -> og_core::glam::Vec3 {
        self.d.as_vec3()
    }

    /// `p` moved by `d`, added in f64 and rounded once.
    pub fn moved(&self, p: og_core::glam::Vec3) -> og_core::glam::Vec3 {
        (p.as_dvec3() + self.d).as_vec3()
    }

    /// `p` moved by [`Self::of`] `bits`.
    pub fn moved_of(&self, bits: u64, p: og_core::glam::Vec3) -> og_core::glam::Vec3 {
        (p.as_dvec3() + self.of(bits)).as_vec3()
    }
}

/// Adds `d` along each lane's axis to one packed value of type `ty`.
fn shift_value(bytes: &mut [u8], ty: FieldType, space: &Worldspace, d: DVec3) {
    let d = d.to_array();
    for lane in 0..space.lanes as usize {
        let by = d[space.axes[lane] as usize];
        if by == 0.0 {
            continue;
        }
        match ty {
            FieldType::F64 => {
                let at = &mut bytes[lane * 8..lane * 8 + 8];
                let v = f64::from_le_bytes(at.try_into().expect("8 bytes")) + by;
                at.copy_from_slice(&v.to_le_bytes());
            }
            _ => {
                let at = &mut bytes[lane * 4..lane * 4 + 4];
                let v = f32::from_le_bytes(at.try_into().expect("4 bytes"));
                let v = (f64::from(v) + by) as f32;
                at.copy_from_slice(&v.to_le_bytes());
            }
        }
    }
}

/// [`shift_value`] over every value of a column's bytes.
fn shift_column(bytes: &mut [u8], size: usize, ty: FieldType, space: &Worldspace, d: DVec3) {
    for value in bytes.chunks_exact_mut(size) {
        shift_value(value, ty, space, d);
    }
}

impl World {
    /// Moves every coordinate of `fields` by `d`, then every state block's
    /// own ([`crate::StateBlock::shift`]): a component field on each entity
    /// its frame moves ([`Worldspace::moves`]; an entity is a child when it
    /// has `parent`, og.Parent), a resource's and a queued event's always
    /// for `world` and `root` (neither has a parent). A field's `f32` lanes
    /// are added in `f64` and rounded once, so a shift by a multiple of a
    /// power of two near the values is exact. Not a change: change
    /// detection compares against shadows moved the same way. Refused
    /// (with nothing moved) while a system runs.
    pub fn shift(&mut self, fields: &[ShiftField], d: DVec3, parent: Option<SchemaId>) -> Result<()> {
        for f in fields {
            self.registry.check(f.schema)?;
        }
        if d == DVec3::ZERO {
            return Ok(());
        }
        let mut children = Vec::new();
        for arch in &mut self.archetypes {
            let child = parent.is_some_and(|p| arch.has(p));
            if child {
                children.extend(arch.entities.iter().map(|e| e.to_bits()));
            }
            for f in fields {
                let Some(position) = arch.position(f.schema) else {
                    continue;
                };
                if !f.space.moves(child) {
                    continue;
                }
                let Some(c) = arch.column_index(f.schema, f.field as u32) else {
                    continue;
                };
                let ty = self.registry.info(f.schema).layout.field_type(f.field);
                let size = ty.size();
                shift_column(arch.columns[c].get_mut().bytes_mut(), size, ty, &f.space, d);
                if let Some(t) = &mut arch.ticks[position] {
                    shift_column(&mut t.shadow[f.field], size, ty, &f.space, d);
                }
            }
        }
        for f in fields {
            if !f.space.moves(false) {
                continue;
            }
            let info = self.registry.info(f.schema);
            let range = info.layout.range(f.field);
            let ty = info.layout.field_type(f.field);
            if let Some(cell) = self.resources[f.schema.0 as usize].as_mut() {
                let value = cell.get_mut();
                let mut bytes = crate::world::resource_to_bytes(&self.registry, f.schema, value)?;
                shift_value(&mut bytes[range.clone()], ty, &f.space, d);
                *value = match &info.rust {
                    Some(rust) => ResourceValue::Typed((rust.decode)(&info.layout, &bytes)?),
                    None => ResourceValue::Raw(bytes),
                };
            }
            let queue = self.events[f.schema.0 as usize].get_mut();
            for e in queue.current.iter_mut().chain(queue.pending.iter_mut()) {
                shift_value(&mut e[range.clone()], ty, &f.space, d);
            }
        }
        let mut shift = Shift::new(d, children);
        for f in fields {
            let layout = &self.registry.info(f.schema).layout;
            let r = layout.range(f.field);
            let ty = layout.field_type(f.field);
            shift
                .fields
                .push((f.schema.0, r.start as u32, r.end as u32, f.space, ty));
        }
        shift.fields.sort_by_key(|f| (f.0, f.1));
        for b in &mut self.blocks {
            b.block.shift(&shift);
        }
        Ok(())
    }
}
