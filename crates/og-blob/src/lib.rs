//! `.ogb`, the one cooked blob container (`docs/design.md` §2.5).
//!
//! A blob is a 64-byte header, a section table, a name pool and the section
//! payloads. The header names what the blob holds (`type_hash`,
//! `schema_version`) and what it was cooked from (`cook_stamp`: a hash of the
//! sources, the cook options and the cooker version). Each section is a named
//! byte range aligned to a power of two, so a loader that places the blob at
//! [`Blob::max_align`] can view a payload in place (`&[T]` for a plain-data
//! `T`) instead of copying it. The exact byte layout is in `README.md`.
//!
//! - [`BlobWriter`] builds a blob. The same calls give the same bytes.
//! - [`Blob::parse`] validates the structure (magic, version, sizes, bounds,
//!   alignment, names) and borrows the bytes; [`Blob::open`] also checks the
//!   type, schema version and cook stamp a loader expects and refuses with the
//!   path and both values on a mismatch (design §6).
//!
//! All integers are little-endian. The crate has no knowledge of what the
//! sections mean; `og-cook` defines the asset blobs on top of it.

use std::borrow::Cow;
use std::fmt;

/// The first eight bytes of every blob: `\x89OGB\r\n\x1a\n` (PNG-style, so a
/// text-mode transfer or a truncated download is detected at once).
pub const MAGIC: [u8; 8] = [0x89, b'O', b'G', b'B', 0x0d, 0x0a, 0x1a, 0x0a];
/// The container layout this crate reads and writes.
pub const CONTAINER_VERSION: u16 = 1;
/// Bytes in the header.
pub const HEADER_SIZE: usize = 64;
/// Bytes in one section table entry.
pub const SECTION_ENTRY_SIZE: usize = 48;
/// The alignment [`BlobWriter::raw`] uses unless told otherwise.
pub const DEFAULT_ALIGN: u32 = 16;
/// The largest section alignment the container accepts.
pub const MAX_ALIGN: u32 = 4096;

/// Why a blob was refused. Every variant that comes from [`Blob::open`]
/// carries the path the loader passed, so the message names the file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlobError {
    /// The bytes are not a well-formed blob (bad magic, truncated, a section
    /// out of bounds, ...).
    Malformed { path: String, reason: String },
    /// A container version this reader does not know.
    ContainerVersion { path: String, found: u16 },
    /// The blob holds another type than the loader asked for.
    TypeMismatch {
        path: String,
        expected: u64,
        found: u64,
    },
    /// The blob was written with another schema version of the type.
    SchemaVersion {
        path: String,
        expected: u32,
        found: u32,
    },
    /// The blob was cooked from other sources, options or cooker than the
    /// manifest records: it is stale (or the manifest is).
    StampMismatch {
        path: String,
        expected: u64,
        found: u64,
    },
    /// A section the loader needs is not there.
    MissingSection { path: String, name: String },
    /// A section exists but does not hold what the loader asked for.
    SectionType {
        path: String,
        name: String,
        reason: String,
    },
}

impl fmt::Display for BlobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlobError::Malformed { path, reason } => {
                write!(f, "{path}: not a valid .ogb blob: {reason}")
            }
            BlobError::ContainerVersion { path, found } => write!(
                f,
                "{path}: .ogb container version {found} is not supported (this reader knows {CONTAINER_VERSION}); re-run og cook"
            ),
            BlobError::TypeMismatch {
                path,
                expected,
                found,
            } => write!(
                f,
                "{path}: holds type {found:016x}, expected {expected:016x}; re-run og cook"
            ),
            BlobError::SchemaVersion {
                path,
                expected,
                found,
            } => write!(
                f,
                "{path}: schema version {found}, expected {expected}; re-run og cook"
            ),
            BlobError::StampMismatch {
                path,
                expected,
                found,
            } => write!(
                f,
                "{path}: cook stamp mismatch: the blob has {found:016x}, the manifest expects {expected:016x}; re-run og cook"
            ),
            BlobError::MissingSection { path, name } => write!(f, "{path}: no section '{name}'"),
            BlobError::SectionType { path, name, reason } => {
                write!(f, "{path}: section '{name}': {reason}")
            }
        }
    }
}

impl std::error::Error for BlobError {}

/// What a loader expects of a blob. `stamp: None` skips the stamp check (for
/// tools that inspect blobs; runtime loaders always pass the manifest's stamp).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Expect {
    pub type_hash: u64,
    pub schema_version: u32,
    pub stamp: Option<u64>,
}

/// The header fields of a blob.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub container_version: u16,
    pub type_hash: u64,
    pub schema_version: u32,
    pub flags: u32,
    pub cook_stamp: u64,
    pub file_size: u64,
    /// The largest section alignment: place the bytes at this alignment for
    /// every in-place view to succeed.
    pub max_align: u32,
}

/// One entry of the section table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section<'a> {
    pub name: &'a str,
    /// Byte offset from the start of the blob.
    pub offset: u64,
    pub size: u64,
    /// The layout hash of the row type for a table, 0 for raw bytes.
    pub type_hash: u64,
    /// Bytes per row for a table, 0 for raw bytes.
    pub stride: u32,
    pub align: u32,
}

fn rd_u16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}
fn rd_u32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().expect("4 bytes"))
}
fn rd_u64(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().expect("8 bytes"))
}

fn align_up(v: u64, align: u64) -> u64 {
    v.div_ceil(align) * align
}

/// A validated blob borrowing its bytes.
#[derive(Clone, Debug)]
pub struct Blob<'a> {
    bytes: &'a [u8],
    path: String,
    header: Header,
    sections: Vec<Section<'a>>,
}

impl<'a> Blob<'a> {
    /// Validates the structure of `bytes`. `path` only labels errors.
    pub fn parse(bytes: &'a [u8], path: &str) -> Result<Blob<'a>, BlobError> {
        let bad = |reason: String| BlobError::Malformed {
            path: path.to_string(),
            reason,
        };
        if bytes.len() < HEADER_SIZE {
            return Err(bad(format!(
                "{} bytes, shorter than the {HEADER_SIZE}-byte header",
                bytes.len()
            )));
        }
        if bytes[..8] != MAGIC {
            return Err(bad("wrong magic (not an .ogb file)".into()));
        }
        let container_version = rd_u16(bytes, 8);
        if container_version != CONTAINER_VERSION {
            return Err(BlobError::ContainerVersion {
                path: path.to_string(),
                found: container_version,
            });
        }
        if rd_u16(bytes, 10) as usize != HEADER_SIZE {
            return Err(bad(format!(
                "header size {} != {HEADER_SIZE}",
                rd_u16(bytes, 10)
            )));
        }
        let count = rd_u32(bytes, 12) as usize;
        let header = Header {
            container_version,
            type_hash: rd_u64(bytes, 16),
            schema_version: rd_u32(bytes, 24),
            flags: rd_u32(bytes, 28),
            cook_stamp: rd_u64(bytes, 32),
            file_size: rd_u64(bytes, 40),
            max_align: rd_u32(bytes, 52),
        };
        let names_size = rd_u32(bytes, 48) as usize;
        if header.file_size != bytes.len() as u64 {
            return Err(bad(format!(
                "header says {} bytes, got {} (truncated or padded)",
                header.file_size,
                bytes.len()
            )));
        }
        if rd_u64(bytes, 56) != 0 || header.flags != 0 {
            return Err(bad("reserved header fields are not zero".into()));
        }
        let table_end = count
            .checked_mul(SECTION_ENTRY_SIZE)
            .and_then(|t| t.checked_add(HEADER_SIZE))
            .ok_or_else(|| bad("section count overflows".into()))?;
        let names_end = table_end
            .checked_add(names_size)
            .filter(|&e| e <= bytes.len())
            .ok_or_else(|| {
                bad(format!(
                    "{count} sections and {names_size} name bytes overrun the file"
                ))
            })?;
        let names = &bytes[table_end..names_end];
        let mut sections = Vec::with_capacity(count);
        let mut cursor = names_end as u64;
        let mut max_align = 1u32;
        for i in 0..count {
            let e = HEADER_SIZE + i * SECTION_ENTRY_SIZE;
            let (no, nl) = (rd_u32(bytes, e) as usize, rd_u32(bytes, e + 4) as usize);
            let name = names
                .get(no..no.saturating_add(nl))
                .and_then(|n| std::str::from_utf8(n).ok())
                .filter(|n| !n.is_empty())
                .ok_or_else(|| bad(format!("section {i} has an invalid name")))?;
            let s = Section {
                name,
                offset: rd_u64(bytes, e + 8),
                size: rd_u64(bytes, e + 16),
                type_hash: rd_u64(bytes, e + 24),
                stride: rd_u32(bytes, e + 32),
                align: rd_u32(bytes, e + 36),
            };
            if rd_u64(bytes, e + 40) != 0 {
                return Err(bad(format!("section '{name}': reserved field is not zero")));
            }
            if !s.align.is_power_of_two() || s.align > MAX_ALIGN {
                return Err(bad(format!(
                    "section '{name}': alignment {} is not a power of two up to {MAX_ALIGN}",
                    s.align
                )));
            }
            if !s.offset.is_multiple_of(s.align as u64) {
                return Err(bad(format!(
                    "section '{name}': offset {} is not aligned to {}",
                    s.offset, s.align
                )));
            }
            if s.offset < cursor {
                return Err(bad(format!("section '{name}' overlaps the previous one")));
            }
            let end = s
                .offset
                .checked_add(s.size)
                .filter(|&end| end <= bytes.len() as u64)
                .ok_or_else(|| {
                    bad(format!(
                        "section '{name}' ({} bytes at {}) runs past the end",
                        s.size, s.offset
                    ))
                })?;
            if s.stride != 0 && !s.size.is_multiple_of(s.stride as u64) {
                return Err(bad(format!(
                    "section '{name}': {} bytes is not a whole number of {}-byte rows",
                    s.size, s.stride
                )));
            }
            if sections.iter().any(|o: &Section| o.name == name) {
                return Err(bad(format!("section '{name}' appears twice")));
            }
            max_align = max_align.max(s.align);
            cursor = end;
            sections.push(s);
        }
        if header.max_align != max_align {
            return Err(bad(format!(
                "header max_align {} != the sections' {}",
                header.max_align, max_align
            )));
        }
        Ok(Blob {
            bytes,
            path: path.to_string(),
            header,
            sections,
        })
    }

    /// [`Blob::parse`] plus the loader's expectations: refuses another type,
    /// another schema version or (when given) another cook stamp, naming
    /// `path` and both values.
    pub fn open(bytes: &'a [u8], path: &str, expect: &Expect) -> Result<Blob<'a>, BlobError> {
        let blob = Blob::parse(bytes, path)?;
        blob.check(expect)?;
        Ok(blob)
    }

    /// The checks of [`Blob::open`] on an already parsed blob.
    pub fn check(&self, expect: &Expect) -> Result<(), BlobError> {
        let path = self.path.clone();
        if self.header.type_hash != expect.type_hash {
            return Err(BlobError::TypeMismatch {
                path,
                expected: expect.type_hash,
                found: self.header.type_hash,
            });
        }
        if self.header.schema_version != expect.schema_version {
            return Err(BlobError::SchemaVersion {
                path,
                expected: expect.schema_version,
                found: self.header.schema_version,
            });
        }
        if let Some(stamp) = expect.stamp
            && self.header.cook_stamp != stamp
        {
            return Err(BlobError::StampMismatch {
                path,
                expected: stamp,
                found: self.header.cook_stamp,
            });
        }
        Ok(())
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Every section in file order.
    pub fn sections(&self) -> &[Section<'a>] {
        &self.sections
    }

    pub fn section(&self, name: &str) -> Option<&Section<'a>> {
        self.sections.iter().find(|s| s.name == name)
    }

    /// The bytes of a section, in place.
    pub fn raw(&self, name: &str) -> Result<&'a [u8], BlobError> {
        let s = self
            .section(name)
            .ok_or_else(|| BlobError::MissingSection {
                path: self.path.clone(),
                name: name.to_string(),
            })?;
        Ok(&self.bytes[s.offset as usize..(s.offset + s.size) as usize])
    }

    /// Like [`Blob::raw`], `None` when the section is absent.
    pub fn raw_opt(&self, name: &str) -> Option<&'a [u8]> {
        self.raw(name).ok()
    }

    fn table_bytes<T: bytemuck::Pod>(
        &self,
        name: &str,
        row_type: u64,
    ) -> Result<&'a [u8], BlobError> {
        let s = *self
            .section(name)
            .ok_or_else(|| BlobError::MissingSection {
                path: self.path.clone(),
                name: name.to_string(),
            })?;
        let wrong = |reason: String| BlobError::SectionType {
            path: self.path.clone(),
            name: name.to_string(),
            reason,
        };
        if s.type_hash != row_type {
            return Err(wrong(format!(
                "rows of type {:016x}, expected {row_type:016x}",
                s.type_hash
            )));
        }
        if s.stride as usize != std::mem::size_of::<T>() {
            return Err(wrong(format!(
                "{}-byte rows, expected {}",
                s.stride,
                std::mem::size_of::<T>()
            )));
        }
        Ok(&self.bytes[s.offset as usize..(s.offset + s.size) as usize])
    }

    /// The rows of a table section as `&[T]` in place when the bytes are
    /// aligned for `T`, otherwise copied. Refused when the section's row type
    /// hash or stride differs from `row_type` and `size_of::<T>()`.
    pub fn table<T: bytemuck::Pod>(
        &self,
        name: &str,
        row_type: u64,
    ) -> Result<Cow<'a, [T]>, BlobError> {
        let bytes = self.table_bytes::<T>(name, row_type)?;
        Ok(match bytemuck::try_cast_slice(bytes) {
            Ok(rows) => Cow::Borrowed(rows),
            Err(_) => {
                let mut rows = vec![T::zeroed(); bytes.len() / std::mem::size_of::<T>()];
                bytemuck::cast_slice_mut::<T, u8>(&mut rows).copy_from_slice(bytes);
                Cow::Owned(rows)
            }
        })
    }

    /// A table whose row layout is only known at run time (scene component
    /// tables): its bytes in place and its stride. Refused when the section's
    /// row type hash differs from `row_type` or it is not a table.
    pub fn rows(&self, name: &str, row_type: u64) -> Result<(&'a [u8], usize), BlobError> {
        let s = *self
            .section(name)
            .ok_or_else(|| BlobError::MissingSection {
                path: self.path.clone(),
                name: name.to_string(),
            })?;
        if s.stride == 0 || s.type_hash != row_type {
            return Err(BlobError::SectionType {
                path: self.path.clone(),
                name: name.to_string(),
                reason: format!(
                    "rows of type {:016x} ({}-byte stride), expected {row_type:016x}",
                    s.type_hash, s.stride
                ),
            });
        }
        Ok((
            &self.bytes[s.offset as usize..(s.offset + s.size) as usize],
            s.stride as usize,
        ))
    }

    /// The rows of a table section in place, or `None` when the bytes are not
    /// aligned for `T` (the blob was not placed at [`Header::max_align`]).
    pub fn table_view<T: bytemuck::Pod>(
        &self,
        name: &str,
        row_type: u64,
    ) -> Result<Option<&'a [T]>, BlobError> {
        let bytes = self.table_bytes::<T>(name, row_type)?;
        Ok(bytemuck::try_cast_slice(bytes).ok())
    }
}

struct Pending {
    name: String,
    bytes: Vec<u8>,
    type_hash: u64,
    stride: u32,
    align: u32,
}

/// Builds a blob. Sections are written in the order they were added, each at
/// the next offset aligned to its alignment, with zero padding: the same
/// calls give byte-identical output.
pub struct BlobWriter {
    type_hash: u64,
    schema_version: u32,
    cook_stamp: u64,
    sections: Vec<Pending>,
}

impl BlobWriter {
    pub fn new(type_hash: u64, schema_version: u32, cook_stamp: u64) -> BlobWriter {
        BlobWriter {
            type_hash,
            schema_version,
            cook_stamp,
            sections: Vec::new(),
        }
    }

    fn push(&mut self, name: &str, bytes: Vec<u8>, type_hash: u64, stride: u32, align: u32) {
        assert!(!name.is_empty(), "a section needs a name");
        assert!(
            align.is_power_of_two() && align <= MAX_ALIGN,
            "section '{name}': alignment {align} is not a power of two up to {MAX_ALIGN}"
        );
        assert!(
            self.sections.iter().all(|s| s.name != name),
            "section '{name}' added twice"
        );
        self.sections.push(Pending {
            name: name.to_string(),
            bytes,
            type_hash,
            stride,
            align,
        });
    }

    /// Adds raw bytes aligned to `align` (a power of two up to [`MAX_ALIGN`]).
    pub fn raw(&mut self, name: &str, bytes: impl Into<Vec<u8>>, align: u32) -> &mut Self {
        self.push(name, bytes.into(), 0, 0, align);
        self
    }

    /// Adds a table of plain-data rows tagged with their row type hash,
    /// aligned to `max(align_of::<T>(), 16)`.
    pub fn table<T: bytemuck::Pod>(&mut self, name: &str, rows: &[T], row_type: u64) -> &mut Self {
        let stride = std::mem::size_of::<T>();
        assert!(
            stride > 0 && stride <= u32::MAX as usize,
            "row size {stride} is not storable"
        );
        let align = (std::mem::align_of::<T>() as u32).max(DEFAULT_ALIGN);
        self.push(
            name,
            bytemuck::cast_slice(rows).to_vec(),
            row_type,
            stride as u32,
            align,
        );
        self
    }

    /// Adds a table of `stride`-byte rows whose layout is only known at run
    /// time (`bytes.len()` must be a multiple of `stride`), aligned to 16.
    pub fn rows(
        &mut self,
        name: &str,
        bytes: impl Into<Vec<u8>>,
        stride: u32,
        row_type: u64,
    ) -> &mut Self {
        let bytes = bytes.into();
        assert!(
            stride > 0 && bytes.len() % stride as usize == 0,
            "section '{name}': {} bytes is not a whole number of {stride}-byte rows",
            bytes.len()
        );
        self.push(name, bytes, row_type, stride, DEFAULT_ALIGN);
        self
    }

    /// The blob's bytes.
    pub fn finish(&self) -> Vec<u8> {
        let count = self.sections.len();
        let mut names = Vec::new();
        let mut name_spans = Vec::with_capacity(count);
        for s in &self.sections {
            name_spans.push((names.len() as u32, s.name.len() as u32));
            names.extend_from_slice(s.name.as_bytes());
        }
        let names_end = (HEADER_SIZE + count * SECTION_ENTRY_SIZE + names.len()) as u64;
        let mut offsets = Vec::with_capacity(count);
        let mut cursor = names_end;
        let mut max_align = 1u32;
        for s in &self.sections {
            let at = align_up(cursor, s.align as u64);
            offsets.push(at);
            cursor = at + s.bytes.len() as u64;
            max_align = max_align.max(s.align);
        }
        let file_size = cursor;
        let mut out = Vec::with_capacity(file_size as usize);
        out.extend_from_slice(&MAGIC);
        out.extend_from_slice(&CONTAINER_VERSION.to_le_bytes());
        out.extend_from_slice(&(HEADER_SIZE as u16).to_le_bytes());
        out.extend_from_slice(&(count as u32).to_le_bytes());
        out.extend_from_slice(&self.type_hash.to_le_bytes());
        out.extend_from_slice(&self.schema_version.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&self.cook_stamp.to_le_bytes());
        out.extend_from_slice(&file_size.to_le_bytes());
        out.extend_from_slice(&(names.len() as u32).to_le_bytes());
        out.extend_from_slice(&max_align.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
        debug_assert_eq!(out.len(), HEADER_SIZE);
        for (i, s) in self.sections.iter().enumerate() {
            out.extend_from_slice(&name_spans[i].0.to_le_bytes());
            out.extend_from_slice(&name_spans[i].1.to_le_bytes());
            out.extend_from_slice(&offsets[i].to_le_bytes());
            out.extend_from_slice(&(s.bytes.len() as u64).to_le_bytes());
            out.extend_from_slice(&s.type_hash.to_le_bytes());
            out.extend_from_slice(&s.stride.to_le_bytes());
            out.extend_from_slice(&s.align.to_le_bytes());
            out.extend_from_slice(&0u64.to_le_bytes());
        }
        out.extend_from_slice(&names);
        for (i, s) in self.sections.iter().enumerate() {
            out.resize(offsets[i] as usize, 0);
            out.extend_from_slice(&s.bytes);
        }
        debug_assert_eq!(out.len() as u64, file_size);
        out
    }
}

/// A byte buffer whose start is aligned to 16 (the default section
/// alignment), so in-place views of a blob read from a file succeed. Hosts
/// that fetch blobs into their own memory should align it the same way.
#[derive(Clone, Default)]
pub struct AlignedBytes {
    words: Vec<Chunk>,
    len: usize,
}

#[repr(C, align(16))]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Chunk([u8; 16]);

impl AlignedBytes {
    pub fn from_slice(bytes: &[u8]) -> AlignedBytes {
        let mut words = vec![Chunk::default(); bytes.len().div_ceil(16)];
        bytemuck::cast_slice_mut::<Chunk, u8>(&mut words)[..bytes.len()].copy_from_slice(bytes);
        AlignedBytes {
            words,
            len: bytes.len(),
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &bytemuck::cast_slice::<Chunk, u8>(&self.words)[..self.len]
    }
}

impl std::ops::Deref for AlignedBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl fmt::Debug for AlignedBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AlignedBytes({} bytes)", self.len)
    }
}
