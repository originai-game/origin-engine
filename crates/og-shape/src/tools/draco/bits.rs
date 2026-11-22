//! Draco's bit streams: bytes and varints, raw bits, and rANS symbols with their tables.

use super::*;

pub(crate) struct Reader<'a> {
    pub(crate) d: &'a [u8],
    pub(crate) at: usize,
}

impl<'a> Reader<'a> {
    pub(crate) fn bytes(&mut self, n: usize) -> Res<&'a [u8]> {
        let end = self.at.checked_add(n).filter(|e| *e <= self.d.len());
        let Some(end) = end else { return short() };
        let b = &self.d[self.at..end];
        self.at = end;
        Ok(b)
    }
    pub(crate) fn u8(&mut self) -> Res<u8> {
        Ok(self.bytes(1)?[0])
    }
    pub(crate) fn u16(&mut self) -> Res<u16> {
        let b = self.bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub(crate) fn u32(&mut self) -> Res<u32> {
        let b = self.bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub(crate) fn i32(&mut self) -> Res<i32> {
        Ok(self.u32()? as i32)
    }
    pub(crate) fn f32(&mut self) -> Res<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
    /// LEB128.
    pub(crate) fn var(&mut self) -> Res<u64> {
        let mut v = 0u64;
        for shift in 0..10 {
            let b = self.u8()?;
            v |= u64::from(b & 0x7f) << (7 * shift);
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        bad("a number of more than ten bytes")
    }
    /// A count of things the data left could hold at `per_byte` of
    /// them a byte: a count is believed only as far as the bytes go.
    pub(crate) fn count(&mut self, per_byte: usize) -> Res<usize> {
        let n = self.var()?;
        let room = ((self.d.len() - self.at) as u64 + 1).saturating_mul(per_byte as u64);
        if n > room {
            return bad("a count larger than the data");
        }
        Ok(n as usize)
    }
}

/// Bits of a byte run, the low bit of each byte first.
pub(crate) struct Bits<'a> {
    pub(crate) d: &'a [u8],
    pub(crate) bit: usize,
}

impl Bits<'_> {
    pub(crate) fn read(&mut self, n: u32) -> Res<u32> {
        let mut v = 0u32;
        for i in 0..n {
            let Some(byte) = self.d.get(self.bit / 8) else {
                return short();
            };
            v |= u32::from((byte >> (self.bit % 8)) & 1) << i;
            self.bit += 1;
        }
        Ok(v)
    }
}

pub(crate) const L_RANS_BASE: u32 = 4096;

/// An ANS decoder over a buffer read from its end.
pub(crate) struct Ans<'a> {
    pub(crate) buf: &'a [u8],
    pub(crate) off: usize,
    pub(crate) state: u32,
}

impl<'a> Ans<'a> {
    /// RansInitDecoder.
    pub(crate) fn new(buf: &'a [u8], base: u32) -> Res<Ans<'a>> {
        let n = buf.len();
        let Some(&last) = buf.last() else {
            return short();
        };
        let take = usize::from(last >> 6) + 1;
        if n < take {
            return short();
        }
        let mut state = 0u32;
        for k in 0..take {
            state |= u32::from(buf[n - take + k]) << (8 * k);
        }
        state &= (1u32 << (8 * take as u32 - 2)) - 1;
        Ok(Ans {
            buf,
            off: n - take,
            state: state.wrapping_add(base),
        })
    }

    /// RabsDescRead: one bit that is zero with probability `p0` / 256.
    pub(crate) fn bit(&mut self, p0: u8) -> bool {
        let p = 256 - u32::from(p0);
        if self.state < L_RANS_BASE && self.off > 0 {
            self.off -= 1;
            self.state = self.state * 256 + u32::from(self.buf[self.off]);
        }
        let x = self.state;
        let (quot, rem) = (x / 256, x % 256);
        let xn = quot * p;
        let val = rem < p;
        self.state = if val {
            xn + rem
        } else {
            x.wrapping_sub(xn).wrapping_sub(p)
        };
        val
    }

    /// RansRead: one symbol of a table.
    pub(crate) fn symbol(&mut self, base: u32, precision: u32, table: &Table) -> Res<u32> {
        while self.state < base && self.off > 0 {
            self.off -= 1;
            self.state = self.state * 256 + u32::from(self.buf[self.off]);
        }
        let (quo, rem) = (self.state / precision, self.state % precision);
        let Some(&sym) = table.lut.get(rem as usize) else {
            return bad("a symbol outside its table");
        };
        let (prob, cum) = table.probs[sym as usize];
        self.state = (u64::from(quo) * u64::from(prob) + u64::from(rem) - u64::from(cum)) as u32;
        Ok(sym)
    }
}

/// A symbol table: each symbol's probability and the sum before it, and
/// the symbol of each remainder.
pub(crate) struct Table {
    pub(crate) probs: Vec<(u32, u32)>,
    pub(crate) lut: Vec<u32>,
}

/// BuildSymbolTables.
pub(crate) fn table(r: &mut Reader, symbols: usize, precision: u32) -> Res<Table> {
    let mut probs = vec![(0u32, 0u32); symbols];
    let mut i = 0;
    while i < symbols {
        let data = r.u8()?;
        let token = data & 3;
        if token == 3 {
            i += usize::from(data >> 2) + 1;
        } else {
            let mut prob = u32::from(data >> 2);
            for j in 0..u32::from(token) {
                prob |= u32::from(r.u8()?) << (8 * (j + 1) - 2);
            }
            probs[i].0 = prob;
            i += 1;
        }
    }
    let mut lut = Vec::with_capacity(precision as usize);
    let mut cum = 0u32;
    for (s, p) in probs.iter_mut().enumerate() {
        p.1 = cum;
        cum = cum.saturating_add(p.0);
        if cum > precision {
            return bad("symbol probabilities over their sum");
        }
        lut.resize(cum as usize, s as u32);
    }
    Ok(Table { probs, lut })
}

/// DecodeSymbols: `values` symbols, `components` of them a bit length
/// when tagged.
pub(crate) fn symbols(r: &mut Reader, values: usize, components: usize) -> Res<Vec<u32>> {
    let mut out = Vec::with_capacity(values);
    match r.u8()? {
        0 => {
            // Tagged: a bit length a point, then that many bits a number.
            let n = r.count(64)?;
            let t = table(r, n, 4096)?;
            let size = r.count(1)?;
            let mut ans = Ans::new(r.bytes(size)?, 16384)?;
            let mut bits = Bits {
                d: &r.d[r.at..],
                bit: 0,
            };
            let mut i = 0;
            while i < values {
                let size = ans.symbol(16384, 4096, &t)?;
                if size > 32 {
                    return bad("a number of more than 32 bits");
                }
                for _ in 0..components.max(1) {
                    out.push(bits.read(size)?);
                }
                i += components.max(1);
            }
            r.at += bits.bit.div_ceil(8);
        }
        1 => {
            let max_bits = u32::from(r.u8()?);
            let n = r.count(64)?;
            let precision_bits = (3 * max_bits / 2).clamp(12, 20);
            let precision = 1u32 << precision_bits;
            let t = table(r, n, precision)?;
            let size = r.count(1)?;
            let mut ans = Ans::new(r.bytes(size)?, precision * 4)?;
            for _ in 0..values {
                out.push(ans.symbol(precision * 4, precision, &t)?);
            }
        }
        _ => return bad("a symbol scheme that is neither tagged nor raw"),
    }
    Ok(out)
}
