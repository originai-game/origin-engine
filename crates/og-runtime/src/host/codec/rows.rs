//! Reading a row by key or slot: from JSON, from packed bytes, or from a column.

use super::*;

/// A field of a record named for a [`Rec`] read: by its name (`"width"`,
/// looked up on each read) or by a [`Slot`] resolved once against the
/// layout the read [`Row`]s have.
pub trait Key {
    /// The field's name (what the JSON is indexed by).
    fn name(&self) -> &str;
    /// The field's index and packed offset in `layout`.
    fn index_in(&self, layout: &Layout) -> Option<(usize, usize)>;
}

impl Key for str {
    fn name(&self) -> &str {
        self
    }
    fn index_in(&self, layout: &Layout) -> Option<(usize, usize)> {
        layout.index_of(self)
    }
}

/// A field name resolved once against one layout: its index and packed
/// offset (`None`: the layout has no such field, and every read of it
/// answers `None` as the JSON would). Per-frame extraction resolves the
/// fields it reads once per layout and reads each [`Row`] by offset, where
/// a binary search over the names for every read was about 5 % of a 2D
/// frame (Hollowbell's sprites, the former test sample A slot read
/// from a row of another layout reads that layout's bytes at the offset:
/// keep a layout's slots beside it.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub key: &'static str,
    pub(super) at: Option<(usize, usize)>,
}

impl Slot {
    pub fn of(layout: &Layout, key: &'static str) -> Slot {
        Slot {
            key,
            at: layout.index_of(key),
        }
    }
}

impl Key for Slot {
    fn name(&self) -> &str {
        self.key
    }
    fn index_in(&self, _: &Layout) -> Option<(usize, usize)> {
        self.at
    }
}

/// A record read field by field, as its JSON would answer: implemented by
/// the decoded [`Value`] and by [`Row`], the packed bytes read in place
/// (per-frame extraction reads a few fields of every sprite or row, where
/// decoding each record's whole JSON map was most of the cost).
pub trait Rec {
    /// `value[key].as_f64()`.
    fn num<K: Key + ?Sized>(&self, key: &K) -> Option<f64>;
    /// `value[key].as_i64()`.
    fn int<K: Key + ?Sized>(&self, key: &K) -> Option<i64>;
    /// `value[key].as_u64()`.
    fn uint<K: Key + ?Sized>(&self, key: &K) -> Option<u64>;
    /// `value[key]` as a flag: a bool as itself, a number as `!= 0`, else `None`.
    fn flag<K: Key + ?Sized>(&self, key: &K) -> Option<bool>;
    /// `value[key][lane].as_f64()` (a vector's `x`, a colour's `r`).
    fn lane<K: Key + ?Sized>(&self, key: &K, lane: &str) -> Option<f64>;
    /// `value[key].as_str()`.
    fn text<K: Key + ?Sized>(&self, key: &K) -> Option<std::borrow::Cow<'_, str>>;
}

impl Rec for Value {
    fn num<K: Key + ?Sized>(&self, key: &K) -> Option<f64> {
        self[key.name()].as_f64()
    }
    fn int<K: Key + ?Sized>(&self, key: &K) -> Option<i64> {
        self[key.name()].as_i64()
    }
    fn uint<K: Key + ?Sized>(&self, key: &K) -> Option<u64> {
        self[key.name()].as_u64()
    }
    fn flag<K: Key + ?Sized>(&self, key: &K) -> Option<bool> {
        match &self[key.name()] {
            Value::Bool(b) => Some(*b),
            Value::Number(n) => Some(n.as_f64().is_some_and(|x| x != 0.0)),
            _ => None,
        }
    }
    fn lane<K: Key + ?Sized>(&self, key: &K, lane: &str) -> Option<f64> {
        self[key.name()][lane].as_f64()
    }
    fn text<K: Key + ?Sized>(&self, key: &K) -> Option<std::borrow::Cow<'_, str>> {
        self[key.name()].as_str().map(std::borrow::Cow::Borrowed)
    }
}

/// A record's packed bytes read in place through its layout: each read
/// answers what the same read of [`Layout::decode`]'s JSON would (a
/// non-finite float is JSON's null; an enum is its variant's name).
#[derive(Clone, Copy)]
pub struct Row<'a> {
    pub layout: &'a Layout,
    pub bytes: &'a [u8],
}

/// A finite f64 as a JSON number answers it (`json!` makes NaN and the
/// infinities null).
pub(super) fn finite(x: f64) -> Option<f64> {
    x.is_finite().then_some(x)
}

impl<'a> Row<'a> {
    pub fn new(layout: &'a Layout, bytes: &'a [u8]) -> Row<'a> {
        Row { layout, bytes }
    }

    /// A field as an integer when its JSON is one (an enum index past its
    /// variants is the number itself).
    #[inline]
    pub(super) fn integer(f: &FieldLayout, b: &[u8]) -> Option<i128> {
        let u32_at = || u32::from_le_bytes(b[..4].try_into().expect("4 bytes"));
        match f.kind {
            Kind::U32 => Some(i128::from(u32_at())),
            Kind::I32 => Some(i128::from(u32_at() as i32)),
            Kind::Enum => {
                let i = u32_at() as usize;
                (i >= f.variants.len()).then_some(i as i128)
            }
            Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => Some(
                i128::from(u64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
            ),
            _ => None,
        }
    }

    /// A float field as its JSON number answers it.
    #[inline]
    pub(super) fn float(f: &FieldLayout, b: &[u8]) -> Option<f64> {
        match f.kind {
            Kind::F32 => finite(f64::from(f32_at(b, 0))),
            Kind::F64 => finite(f64::from_le_bytes(b[..8].try_into().expect("8 bytes"))),
            // An integer field holds a u64 or an i32: converted from that
            // width, the same number as from the i128 without its library
            // call (four handle reads of every sprite).
            _ => Row::integer(f, b).map(|i| {
                if i < 0 {
                    i as i64 as f64
                } else {
                    i as u64 as f64
                }
            }),
        }
    }
}

/// A row whose fields are read in place as packed bytes: [`Row`] (one
/// packed record) and [`ColRow`] (a row of an archetype's columns). Their
/// [`Rec`] reads are the same code.
pub trait Packed {
    /// The field `key` names and its bytes; `None` for a list, whose JSON
    /// is an array that none of [`Rec`]'s reads answers.
    fn field<K: Key + ?Sized>(&self, key: &K) -> Option<(&FieldLayout, &[u8])>;
}

impl<'a> Row<'a> {
    /// [`Packed::field`], borrowed from the row's layout and bytes.
    #[inline]
    pub(super) fn cell<K: Key + ?Sized>(self, key: &K) -> Option<(&'a FieldLayout, &'a [u8])> {
        let (i, at) = key.index_in(self.layout)?;
        let f = self.layout.fields.get(i)?;
        if f.ty.is_list() {
            return None;
        }
        self.bytes.get(at..at + f.ty.size()).map(|b| (f, b))
    }
}

impl Packed for Row<'_> {
    #[inline]
    fn field<K: Key + ?Sized>(&self, key: &K) -> Option<(&FieldLayout, &[u8])> {
        self.cell(key)
    }
}

/// One row of a component read in place from its archetype's columns
/// ([`crate::present::extract::column_rows`]): each read answers what the same read
/// of a [`Row`] over the row's packed bytes would, without packing them.
#[derive(Clone, Copy)]
pub struct ColRow<'a> {
    pub(super) layout: &'a Layout,
    pub(super) cells: &'a Cells<'a>,
    pub(super) row: usize,
}

/// One field's column in an archetype, with what every read of a row
/// needs of the field's type (working both out per read was a quarter of
/// reading 20,000 sprites).
pub(crate) struct Col<'a> {
    pub(super) bytes: std::cell::Ref<'a, [u8]>,
    /// The field's bytes a row.
    pub(super) size: usize,
    /// A list: its JSON is an array, which none of [`Rec`]'s reads answers.
    pub(super) list: bool,
}

impl<'a> Col<'a> {
    /// `bytes` as field `f`'s column: `f`'s size a row.
    pub(crate) fn of(bytes: std::cell::Ref<'a, [u8]>, f: &FieldLayout) -> Col<'a> {
        Col {
            bytes,
            size: f.ty.size(),
            list: f.ty.is_list(),
        }
    }
}

/// One archetype's values of a component.
pub(crate) enum Cells<'a> {
    /// Its columns, one per field of the layout in order.
    Columns(Vec<Col<'a>>),
    /// Its rows packed one after the other, `stride` bytes each: columns
    /// that are not the layout's fields (a [`Row`] reads those bytes by the
    /// layout's offsets, and so does this).
    Rows { bytes: Vec<u8>, stride: usize },
}

impl<'a> ColRow<'a> {
    pub(crate) fn new(layout: &'a Layout, cells: &'a Cells<'a>, row: usize) -> ColRow<'a> {
        ColRow { layout, cells, row }
    }

    /// The row's packed bytes ([`Layout::decode`]'s input) appended to `out`.
    pub(crate) fn pack(&self, out: &mut Vec<u8>) {
        match self.cells {
            Cells::Columns(cols) => {
                for c in cols {
                    out.extend_from_slice(&c.bytes[self.row * c.size..(self.row + 1) * c.size]);
                }
            }
            Cells::Rows { bytes, stride } => {
                out.extend_from_slice(&bytes[self.row * stride..(self.row + 1) * stride]);
            }
        }
    }
}

impl Packed for ColRow<'_> {
    // Always: left as a call it was a fifth of reading 20,000 sprites, some
    // thirty fields each.
    #[inline(always)]
    fn field<K: Key + ?Sized>(&self, key: &K) -> Option<(&FieldLayout, &[u8])> {
        match self.cells {
            Cells::Columns(cols) => {
                let (i, _) = key.index_in(self.layout)?;
                let c = cols.get(i)?;
                if c.list {
                    return None;
                }
                let at = self.row * c.size;
                Some((self.layout.fields.get(i)?, c.bytes.get(at..at + c.size)?))
            }
            Cells::Rows { bytes, stride } => {
                let row = bytes.get(self.row * stride..(self.row + 1) * stride)?;
                Row::new(self.layout, row).cell(key)
            }
        }
    }
}

impl<T: Packed> Rec for T {
    #[inline]
    fn num<K: Key + ?Sized>(&self, key: &K) -> Option<f64> {
        let (f, b) = self.field(key)?;
        Row::float(f, b)
    }
    #[inline]
    fn int<K: Key + ?Sized>(&self, key: &K) -> Option<i64> {
        let (f, b) = self.field(key)?;
        Row::integer(f, b).and_then(|i| i64::try_from(i).ok())
    }
    #[inline]
    fn uint<K: Key + ?Sized>(&self, key: &K) -> Option<u64> {
        let (f, b) = self.field(key)?;
        Row::integer(f, b).and_then(|i| u64::try_from(i).ok())
    }
    #[inline]
    fn flag<K: Key + ?Sized>(&self, key: &K) -> Option<bool> {
        let (f, b) = self.field(key)?;
        match f.kind {
            Kind::Bool => Some(b.iter().any(|x| *x != 0)),
            Kind::F32 | Kind::F64 => Row::float(f, b).map(|x| x != 0.0),
            _ => Row::integer(f, b).map(|i| i != 0),
        }
    }
    #[inline]
    fn lane<K: Key + ?Sized>(&self, key: &K, lane: &str) -> Option<f64> {
        let (f, b) = self.field(key)?;
        finite(f64::from(f32_at(b, f.kind.lane_of(lane)?)))
    }
    #[inline]
    fn text<K: Key + ?Sized>(&self, key: &K) -> Option<std::borrow::Cow<'_, str>> {
        let (f, b) = self.field(key)?;
        match f.kind {
            Kind::Str => Some(std::borrow::Cow::Borrowed(og_core::unpack_str(b))),
            Kind::Ease => Some(std::borrow::Cow::Owned(
                og_core::ease::Ease::from_bytes(b)?.to_text(),
            )),
            Kind::Enum => {
                let i = u32::from_le_bytes(b[..4].try_into().expect("4 bytes")) as usize;
                f.variants
                    .get(i)
                    .map(|n| std::borrow::Cow::Borrowed(n.as_str()))
            }
            _ => None,
        }
    }
}
