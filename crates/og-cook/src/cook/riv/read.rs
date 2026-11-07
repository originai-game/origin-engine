//! The `.riv` binary as objects (https://rive.app/docs/runtimes/advanced-topic/format):
//! the header with its table of contents, then objects, each a type key
//! and properties until a zero key. A property's field type comes from the
//! pinned format tables ([`super::defs`]), else from the file's table of
//! contents (a property newer than the tables, skipped by its field type).

use super::{Field, MAJOR, defs};
use std::collections::HashMap;

/// A property's value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Prop {
    Uint(u64),
    Int(i64),
    Float(f32),
    Bool(bool),
    /// 0xAARRGGBB.
    Color(u32),
    String(String),
    Bytes(Vec<u8>),
}

/// One object of the file, in file order.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Object {
    pub ty: u16,
    pub props: Vec<(u16, Prop)>,
}

impl Object {
    pub fn get(&self, key: u16) -> Option<&Prop> {
        self.props.iter().find(|(k, _)| *k == key).map(|(_, v)| v)
    }

    /// A number property (uint, int, float, bool, colour), or the default
    /// the format gives it when the file leaves it out.
    pub fn num(&self, key: u16) -> f64 {
        match self.get(key) {
            Some(Prop::Uint(v)) => *v as f64,
            Some(Prop::Int(v)) => *v as f64,
            Some(Prop::Float(v)) => f64::from(*v),
            Some(Prop::Bool(v)) => f64::from(u8::from(*v)),
            Some(Prop::Color(v)) => f64::from(*v),
            _ => default_of(key),
        }
    }

    pub fn f(&self, key: u16) -> f32 {
        self.num(key) as f32
    }

    pub fn u(&self, key: u16) -> u32 {
        self.num(key) as u32
    }

    pub fn flag(&self, key: u16) -> bool {
        self.num(key) != 0.0
    }

    pub fn string(&self, key: u16) -> Option<&str> {
        match self.get(key)? {
            Prop::String(v) => Some(v),
            _ => None,
        }
    }
}

/// A read file: its version and objects.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RivFile {
    pub major: u64,
    pub minor: u64,
    pub objects: Vec<Object>,
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn eof(&self) -> bool {
        self.at >= self.b.len()
    }

    fn byte(&mut self) -> Result<u8, String> {
        let v = *self
            .b
            .get(self.at)
            .ok_or_else(|| format!("the file ends early, at byte {}", self.at))?;
        self.at += 1;
        Ok(v)
    }

    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self
            .at
            .checked_add(n)
            .filter(|e| *e <= self.b.len())
            .ok_or_else(|| format!("the file ends early, at byte {}", self.at))?;
        let s = &self.b[self.at..end];
        self.at = end;
        Ok(s)
    }

    /// LEB128.
    fn varuint(&mut self) -> Result<u64, String> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.byte()?;
            v |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(format!(
            "a variable-length integer overflows at byte {}",
            self.at
        ))
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().expect("4 bytes"),
        ))
    }

    fn value(&mut self, f: Field) -> Result<Prop, String> {
        Ok(match f {
            Field::Uint => Prop::Uint(self.varuint()?),
            Field::Int => {
                let z = self.varuint()?;
                Prop::Int(((z >> 1) as i64) ^ -((z & 1) as i64))
            }
            Field::Float => Prop::Float(f32::from_bits(self.u32()?)),
            Field::Bool => Prop::Bool(self.byte()? == 1),
            Field::Color => Prop::Color(self.u32()?),
            Field::String => {
                let n = self.varuint()? as usize;
                Prop::String(
                    String::from_utf8(self.take(n)?.to_vec())
                        .map_err(|_| format!("a string before byte {} is not UTF-8", self.at))?,
                )
            }
            Field::Bytes => {
                let n = self.varuint()? as usize;
                Prop::Bytes(self.take(n)?.to_vec())
            }
        })
    }
}

/// The field types of the table of contents' two-bit codes: a varuint (a
/// bool or an int reads as one), a length and its bytes, 4 bytes, 4 bytes.
const TOC_FIELDS: [Field; 4] = [Field::Uint, Field::Bytes, Field::Float, Field::Color];

/// Reads a `.riv`: its header (fingerprint `RIVE`, major [`MAJOR`]) and
/// every object.
pub(crate) fn read(bytes: &[u8]) -> Result<RivFile, String> {
    let mut r = Reader { b: bytes, at: 0 };
    if r.take(4).ok() != Some(b"RIVE".as_slice()) {
        return Err("not a Rive file (no RIVE fingerprint)".into());
    }
    let major = r.varuint()?;
    let minor = r.varuint()?;
    if major != MAJOR {
        return Err(format!(
            "Rive format {major}.{minor}: the import reads format {MAJOR} only; re-export the file from the Rive editor for runtime format {MAJOR}"
        ));
    }
    let _file_id = r.varuint()?;
    let mut keys = Vec::new();
    loop {
        let k = r.varuint()?;
        if k == 0 {
            break;
        }
        keys.push(k as u16);
    }
    let mut toc = HashMap::new();
    let mut word = 0u32;
    for (i, k) in keys.iter().enumerate() {
        // Four keys a word, two bits each, low bits first (rive-runtime's
        // RuntimeHeader::read).
        if i % 4 == 0 {
            word = r.u32()?;
        }
        toc.insert(*k, TOC_FIELDS[((word >> ((i % 4) * 2)) & 3) as usize]);
    }
    let props: HashMap<u16, Field> = defs::PROPERTIES
        .iter()
        .map(|(k, _, _, f, _)| (*k, *f))
        .collect();
    let mut objects = Vec::new();
    while !r.eof() {
        let ty = r.varuint()? as u16;
        let mut o = Object {
            ty,
            props: Vec::new(),
        };
        loop {
            let k = r.varuint()? as u16;
            if k == 0 {
                break;
            }
            let f = props.get(&k).or_else(|| toc.get(&k)).copied().ok_or_else(|| {
                format!(
                    "property {k} of a {} has no field type in the format tables or the file's table of contents",
                    type_name(ty)
                )
            })?;
            o.props.push((k, r.value(f)?));
        }
        objects.push(o);
    }
    Ok(RivFile {
        major,
        minor,
        objects,
    })
}

/// The format's default for property `key` (0 for one it does not know).
pub(crate) fn default_of(key: u16) -> f64 {
    defs::PROPERTIES
        .iter()
        .find(|p| p.0 == key)
        .map_or(0.0, |p| p.4)
}

/// A type key's name in the pinned tables (or the key, when it is newer).
pub(crate) fn type_name(ty: u16) -> String {
    defs::TYPES
        .iter()
        .find(|(k, _, _, _)| *k == ty)
        .map_or_else(
            || format!("object type {ty}"),
            |(_, n, _, _)| (*n).to_string(),
        )
}

/// Whether type `ty` is `ancestor` or derives from it.
pub(crate) fn is_a(ty: u16, ancestor: u16) -> bool {
    let mut t = Some(ty);
    for _ in 0..32 {
        match t {
            Some(k) if k == ancestor => return true,
            Some(k) => {
                t = defs::TYPES
                    .iter()
                    .find(|(x, _, _, _)| *x == k)
                    .and_then(|(_, _, p, _)| *p)
            }
            None => return false,
        }
    }
    false
}
