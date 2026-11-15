//! Little-endian bytes: the writer and the bounds-checked reader every
//! state block, snapshot and save is written and read with, the one copy.
//!
//! The reader never panics on short or hostile input: a read past the end,
//! a count that claims more items than the bytes left could hold, or text
//! that is not UTF-8 is an [`Error`] of kind [`ErrorKind::Parse`] naming the
//! block (`what`) and the byte it happened at.

use crate::math::{Quat, Vec2, Vec3};
use crate::{Error, ErrorKind, Result};

/// Appends little-endian values to a byte vector.
pub struct ByteWriter<'a> {
    out: &'a mut Vec<u8>,
}

impl<'a> ByteWriter<'a> {
    pub fn new(out: &'a mut Vec<u8>) -> ByteWriter<'a> {
        ByteWriter { out }
    }

    /// The vector written to, for a type that writes itself into one.
    pub fn out(&mut self) -> &mut Vec<u8> {
        self.out
    }
    pub fn bytes(&mut self, b: &[u8]) {
        self.out.extend_from_slice(b);
    }
    pub fn u8(&mut self, v: u8) {
        self.out.push(v);
    }
    /// 1 for true, 0 for false.
    pub fn bool(&mut self, v: bool) {
        self.out.push(u8::from(v));
    }
    pub fn u16(&mut self, v: u16) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn u32(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn i32(&mut self, v: i32) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.bytes(&v.to_le_bytes());
    }
    pub fn i64(&mut self, v: i64) {
        self.bytes(&v.to_le_bytes());
    }
    /// The value's bits: a NaN keeps its payload, -0.0 its sign.
    pub fn f32(&mut self, v: f32) {
        self.bytes(&v.to_bits().to_le_bytes());
    }
    pub fn f64(&mut self, v: f64) {
        self.bytes(&v.to_bits().to_le_bytes());
    }
    pub fn vec2(&mut self, v: Vec2) {
        self.f32(v.x);
        self.f32(v.y);
    }
    pub fn vec3(&mut self, v: Vec3) {
        self.f32(v.x);
        self.f32(v.y);
        self.f32(v.z);
    }
    /// x, y, z, w.
    pub fn quat(&mut self, q: Quat) {
        for v in q.to_array() {
            self.f32(v);
        }
    }
    /// A length or count as u32. Panics past `u32::MAX`, which no block
    /// holds.
    pub fn length(&mut self, n: usize) {
        self.u32(u32::try_from(n).expect("a length fits in u32"));
    }
    /// `b` after its length ([`Self::length`]).
    pub fn sized(&mut self, b: &[u8]) {
        self.length(b.len());
        self.bytes(b);
    }
    /// `s`'s UTF-8 after its length.
    pub fn str(&mut self, s: &str) {
        self.sized(s.as_bytes());
    }
}

/// Reads little-endian values in order, checking every read against the end.
/// A clone is a bookmark: reading on from it leaves the original where it was.
#[derive(Clone)]
pub struct ByteReader<'a> {
    bytes: &'a [u8],
    at: usize,
    what: &'static str,
}

impl<'a> ByteReader<'a> {
    /// A reader of `bytes`; `what` names them in errors ("og.anim state
    /// block").
    pub fn new(bytes: &'a [u8], what: &'static str) -> ByteReader<'a> {
        ByteReader { bytes, at: 0, what }
    }

    /// A parse error about these bytes: `"{what}: {reason}"`.
    pub fn error(&self, reason: impl std::fmt::Display) -> Error {
        Error::new(ErrorKind::Parse, format!("{}: {reason}", self.what))
    }

    /// The bytes read so far.
    pub fn position(&self) -> usize {
        self.at
    }
    /// The bytes not read yet.
    pub fn remaining(&self) -> usize {
        self.bytes.len() - self.at
    }
    /// Every byte was read.
    pub fn is_done(&self) -> bool {
        self.at == self.bytes.len()
    }
    /// An error when bytes are left over.
    pub fn finish(&self) -> Result<()> {
        if self.is_done() {
            Ok(())
        } else {
            Err(self.error(format!("{} trailing bytes", self.remaining())))
        }
    }
    /// The rest, unread.
    pub fn rest(&mut self) -> &'a [u8] {
        let out = &self.bytes[self.at..];
        self.at = self.bytes.len();
        out
    }

    /// What `read` makes of the unread bytes, for a type that reads itself
    /// from a slice it advances: the reader moves past what it consumed.
    pub fn parse<T>(&mut self, read: impl FnOnce(&mut &'a [u8]) -> T) -> T {
        let mut b = &self.bytes[self.at..];
        let before = b.len();
        let out = read(&mut b);
        self.at += before - b.len().min(before);
        out
    }

    /// The next `n` bytes.
    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self
            .at
            .checked_add(n)
            .filter(|&e| e <= self.bytes.len())
            .ok_or_else(|| {
                self.error(format!(
                    "truncated at byte {} ({n} more needed, {} left)",
                    self.at,
                    self.remaining()
                ))
            })?;
        let out = &self.bytes[self.at..end];
        self.at = end;
        Ok(out)
    }
    pub fn array<const N: usize>(&mut self) -> Result<[u8; N]> {
        Ok(self.take(N)?.try_into().expect("N bytes"))
    }
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    /// Any non-zero byte is true.
    pub fn bool(&mut self) -> Result<bool> {
        Ok(self.u8()? != 0)
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.array()?))
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    pub fn i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.array()?))
    }
    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    pub fn i64(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.array()?))
    }
    pub fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
    pub fn f64(&mut self) -> Result<f64> {
        Ok(f64::from_bits(self.u64()?))
    }
    pub fn vec2(&mut self) -> Result<Vec2> {
        Ok(Vec2::new(self.f32()?, self.f32()?))
    }
    pub fn vec3(&mut self) -> Result<Vec3> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    /// x, y, z, w.
    pub fn quat(&mut self) -> Result<Quat> {
        Ok(Quat::from_xyzw(
            self.f32()?,
            self.f32()?,
            self.f32()?,
            self.f32()?,
        ))
    }
    /// A u32 length, as written by [`ByteWriter::length`].
    pub fn length(&mut self) -> Result<usize> {
        Ok(self.u32()? as usize)
    }
    /// A count of items at least `min_item` bytes each, refused when the
    /// bytes left could not hold them (so a hostile count never sizes an
    /// allocation).
    pub fn count(&mut self, min_item: usize) -> Result<usize> {
        let n = self.length()?;
        if n.saturating_mul(min_item) > self.remaining() {
            return Err(self.error(format!(
                "count {n} at byte {} exceeds the data",
                self.at - 4
            )));
        }
        Ok(n)
    }
    /// Bytes after their length, as written by [`ByteWriter::sized`].
    pub fn sized(&mut self) -> Result<&'a [u8]> {
        let n = self.length()?;
        self.take(n)
    }
    /// UTF-8 text after its length, as written by [`ByteWriter::str`].
    pub fn str(&mut self) -> Result<&'a str> {
        let at = self.at;
        std::str::from_utf8(self.sized()?)
            .map_err(|_| self.error(format!("the text at byte {at} is not UTF-8")))
    }
    /// [`Self::str`] owned.
    pub fn string(&mut self) -> Result<String> {
        Ok(self.str()?.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_written_reads_back_and_a_short_or_long_block_is_refused() {
        let mut out = Vec::new();
        let mut w = ByteWriter::new(&mut out);
        w.u8(7);
        w.bool(true);
        w.u16(0xbeef);
        w.u32(0xdead_beef);
        w.i32(-5);
        w.u64(u64::MAX - 1);
        w.i64(-6);
        w.f32(-0.0);
        w.f64(1.5);
        w.vec3(Vec3::new(1.0, 2.0, 3.0));
        w.quat(Quat::from_xyzw(0.0, 0.0, 0.6, 0.8));
        w.str("héllo");
        // Little-endian on the wire.
        assert_eq!(&out[2..4], &[0xef, 0xbe]);
        let mut r = ByteReader::new(&out, "test block");
        assert_eq!(r.u8().unwrap(), 7);
        assert!(r.bool().unwrap());
        assert_eq!(r.u16().unwrap(), 0xbeef);
        assert_eq!(r.u32().unwrap(), 0xdead_beef);
        assert_eq!(r.i32().unwrap(), -5);
        assert_eq!(r.u64().unwrap(), u64::MAX - 1);
        assert_eq!(r.i64().unwrap(), -6);
        assert!(r.f32().unwrap().is_sign_negative());
        assert_eq!(r.f64().unwrap(), 1.5);
        assert_eq!(r.vec3().unwrap(), Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(r.quat().unwrap(), Quat::from_xyzw(0.0, 0.0, 0.6, 0.8));
        assert_eq!(r.str().unwrap(), "héllo");
        r.finish().unwrap();

        let mut short = ByteReader::new(&out[..3], "test block");
        short.u8().unwrap();
        let e = short.u64().unwrap_err();
        assert_eq!(e.kind(), ErrorKind::Parse);
        assert!(
            e.reason().starts_with("test block: truncated at byte 1"),
            "{e}"
        );
        // The failed read consumed nothing.
        assert_eq!(short.position(), 1);

        let mut long = ByteReader::new(&out, "test block");
        long.u8().unwrap();
        assert!(long.finish().is_err());
    }

    #[test]
    fn a_count_or_length_past_the_end_is_refused_before_anything_is_sized() {
        let mut out = Vec::new();
        let mut w = ByteWriter::new(&mut out);
        w.u32(u32::MAX);
        w.u32(1);
        assert!(ByteReader::new(&out, "b").count(1).is_err());
        assert!(ByteReader::new(&out, "b").sized().is_err());
        let mut r = ByteReader::new(&out, "b");
        r.u32().unwrap();
        assert_eq!(r.count(0).unwrap(), 1);
        // Not UTF-8.
        let bad = [2, 0, 0, 0, 0xff, 0xfe];
        assert!(ByteReader::new(&bad, "b").str().is_err());
    }
}
