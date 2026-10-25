//! Little-endian bytes for the state block: a writer, and a reader that
//! refuses short or trailing input.

use og_core::{Error, ErrorKind, Result};

pub(crate) struct Writer<'a> {
    out: &'a mut Vec<u8>,
}

impl<'a> Writer<'a> {
    pub fn new(out: &'a mut Vec<u8>) -> Writer<'a> {
        Writer { out }
    }
    fn put(&mut self, b: &[u8]) {
        self.out.extend_from_slice(b);
    }
    pub fn u8(&mut self, v: u8) {
        self.put(&[v]);
    }
    pub fn u32(&mut self, v: u32) {
        self.put(&v.to_le_bytes());
    }
    pub fn u64(&mut self, v: u64) {
        self.put(&v.to_le_bytes());
    }
    pub fn f32(&mut self, v: f32) {
        self.put(&v.to_bits().to_le_bytes());
    }
}

pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes, at: 0 }
    }
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.at.checked_add(n).filter(|&e| e <= self.bytes.len());
        match end {
            Some(end) => {
                let s = &self.bytes[self.at..end];
                self.at = end;
                Ok(s)
            }
            None => Err(Error::new(
                ErrorKind::Parse,
                format!(
                    "the og.anim state ends at byte {} of {} needed",
                    self.bytes.len(),
                    self.at + n
                ),
            )),
        }
    }
    pub fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    pub fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
    /// A count of items of at least `item` bytes each, checked against what is left.
    pub fn count(&mut self, item: usize) -> Result<usize> {
        let n = self.u32()? as usize;
        if n.saturating_mul(item) > self.bytes.len() - self.at {
            return Err(Error::new(
                ErrorKind::Parse,
                format!("the og.anim state claims {n} items past its end"),
            ));
        }
        Ok(n)
    }
    /// Every byte was read.
    pub fn done(&self) -> bool {
        self.at == self.bytes.len()
    }
    pub fn finish(&self) -> Result<()> {
        if self.at != self.bytes.len() {
            return Err(Error::new(
                ErrorKind::Parse,
                format!(
                    "the og.anim state has {} trailing bytes",
                    self.bytes.len() - self.at
                ),
            ));
        }
        Ok(())
    }
}
