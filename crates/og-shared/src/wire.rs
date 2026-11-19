//! The byte encoding every shared-world message is written in: LEB128
//! varints, zigzag for signed values, little-endian fixed widths, and
//! length-prefixed bytes and UTF-8 strings.

use std::fmt;

/// A message that does not decode: truncated, too long or malformed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WireError(pub &'static str);

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for WireError {}

pub type WireResult<T> = Result<T, WireError>;

/// Writes a message.
#[derive(Default)]
pub struct Writer {
    pub buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Writer {
        Writer::default()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn i16(&mut self, v: i16) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn f32(&mut self, v: f32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn varint(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.buf.push((v as u8) | 0x80);
            v >>= 7;
        }
        self.buf.push(v as u8);
    }

    pub fn zigzag(&mut self, v: i64) {
        self.varint(((v << 1) ^ (v >> 63)) as u64);
    }

    pub fn bytes(&mut self, b: &[u8]) {
        self.varint(b.len() as u64);
        self.buf.extend_from_slice(b);
    }

    pub fn str(&mut self, s: &str) {
        self.bytes(s.as_bytes());
    }
}

/// Reads a message.
pub struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(buf: &'a [u8]) -> Reader<'a> {
        Reader { buf, pos: 0 }
    }

    pub fn done(&self) -> bool {
        self.pos == self.buf.len()
    }

    fn take(&mut self, n: usize) -> WireResult<&'a [u8]> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|&e| e <= self.buf.len())
            .ok_or(WireError("the message ends early"))?;
        let s = &self.buf[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    pub fn u8(&mut self) -> WireResult<u8> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> WireResult<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub fn i16(&mut self) -> WireResult<i16> {
        Ok(i16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub fn u32(&mut self) -> WireResult<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> WireResult<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn f32(&mut self) -> WireResult<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn varint(&mut self) -> WireResult<u64> {
        let mut v = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.u8()?;
            v |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(WireError("a varint is longer than 64 bits"))
    }

    pub fn varint_u32(&mut self) -> WireResult<u32> {
        u32::try_from(self.varint()?).map_err(|_| WireError("a number is past 32 bits"))
    }

    pub fn zigzag(&mut self) -> WireResult<i64> {
        let v = self.varint()?;
        Ok(((v >> 1) as i64) ^ -((v & 1) as i64))
    }

    pub fn bytes(&mut self, max: usize) -> WireResult<&'a [u8]> {
        let n = self.varint()?;
        if n > max as u64 {
            return Err(WireError("a field is longer than its limit"));
        }
        self.take(n as usize)
    }

    pub fn str(&mut self, max: usize) -> WireResult<&'a str> {
        std::str::from_utf8(self.bytes(max)?).map_err(|_| WireError("a string is not UTF-8"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varints_and_zigzag_round_trip_at_their_boundaries() {
        let mut w = Writer::new();
        let unsigned = [
            0u64,
            1,
            127,
            128,
            16_383,
            16_384,
            u64::from(u32::MAX),
            u64::MAX,
        ];
        let signed = [
            0i64,
            -1,
            1,
            -64,
            64,
            i64::from(i32::MIN),
            i64::MAX,
            i64::MIN,
        ];
        for v in unsigned {
            w.varint(v);
        }
        for v in signed {
            w.zigzag(v);
        }
        let mut r = Reader::new(&w.buf);
        for v in unsigned {
            assert_eq!(r.varint().unwrap(), v);
        }
        for v in signed {
            assert_eq!(r.zigzag().unwrap(), v);
        }
        assert!(r.done());
        // 127 is one byte, 128 two; -64 zigzags to 127 (one byte), 64 to 128 (two).
        let size = |f: &dyn Fn(&mut Writer)| {
            let mut w = Writer::new();
            f(&mut w);
            w.len()
        };
        assert_eq!(size(&|w| w.varint(127)), 1);
        assert_eq!(size(&|w| w.varint(128)), 2);
        assert_eq!(size(&|w| w.zigzag(-64)), 1);
        assert_eq!(size(&|w| w.zigzag(64)), 2);
    }

    #[test]
    fn truncated_and_oversized_fields_are_errors_not_panics() {
        assert!(Reader::new(&[0x80]).varint().is_err());
        assert!(Reader::new(&[0xff; 11]).varint().is_err());
        let mut w = Writer::new();
        w.bytes(&[1, 2, 3]);
        assert!(Reader::new(&w.buf).bytes(2).is_err());
        assert!(Reader::new(&w.buf[..3]).bytes(8).is_err());
        assert!(Reader::new(&[1, 0xff]).str(8).is_err());
    }
}
