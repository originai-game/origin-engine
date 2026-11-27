//! Native sheets shared by every producer and both runtime hosts.
use super::*;

impl FlipbookSheet {
    /// Validate the native sheet independently of its authoring frontend.
    pub fn validate_native(&self) -> Result<()> {
        let invalid = |s: &str| Error::new(ErrorKind::Invalid, s);
        if self.texture.is_empty() || self.size.contains(&0) {
            return Err(invalid("a sheet needs a texture and positive dimensions"));
        }
        let mut names = std::collections::BTreeSet::new();
        for f in &self.frames {
            let [x, y, w, h] = f.rect;
            if !names.insert(&f.name)
                || w == 0
                || h == 0
                || x.checked_add(w).is_none_or(|n| n > self.size[0])
                || y.checked_add(h).is_none_or(|n| n > self.size[1])
                || f.pivot.iter().any(|n| !n.is_finite())
            {
                return Err(invalid("duplicate frame name, invalid rectangle or pivot"));
            }
        }
        names.clear();
        for c in &self.clips {
            if !names.insert(&c.name)
                || c.frames.is_empty()
                || c.frames.len() != c.durations.len()
                || c.frames.iter().any(|&i| i as usize >= self.frames.len())
                || c.durations.iter().any(|&d| !d.is_finite() || d <= 0.0)
                || !c.durations.iter().sum::<f32>().is_finite()
                || c.events
                    .iter()
                    .any(|(i, n)| *i as usize >= c.frames.len() || n.len() > 63)
            {
                return Err(invalid("invalid clip frames, durations, name or events"));
            }
        }
        Ok(())
    }

    /// Encode little-endian fields; callers validate author inputs first.
    pub fn encode(&self, out: &mut Vec<u8>) {
        let mut w = ByteWriter::new(out);
        w.bytes(b"OGFB\x02\0\0\0");
        w.str(&self.texture);
        w.u32(self.size[0]);
        w.u32(self.size[1]);
        w.u8(match self.filter {
            SheetFilter::Nearest => 0,
            SheetFilter::Linear => 1,
        });
        w.bool(self.array);
        w.length(self.frames.len());
        for f in &self.frames {
            w.str(&f.name);
            for n in f.rect {
                w.u32(n);
            }
            for n in f.pivot {
                w.f32(n);
            }
        }
        w.length(self.clips.len());
        for c in &self.clips {
            w.str(&c.name);
            w.u8(match c.mode {
                FlipMode::Loop => 0,
                FlipMode::Once => 1,
                FlipMode::PingPong => 2,
            });
            w.length(c.frames.len());
            for (&i, &d) in c.frames.iter().zip(&c.durations) {
                w.u32(i);
                w.f32(d);
            }
            w.length(c.events.len());
            for (i, n) in &c.events {
                w.u32(*i);
                w.str(n);
            }
        }
    }

    /// Decode bounded native fields, rejecting malformed or old text payloads.
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        let mut r = ByteReader::new(bytes, "a native flipbook");
        if r.take(8)? != b"OGFB\x02\0\0\0" {
            return Err(r.error("unknown sheet format"));
        }
        let mut sheet = Self {
            texture: r.string()?,
            size: [r.u32()?, r.u32()?],
            filter: match r.u8()? {
                0 => SheetFilter::Nearest,
                1 => SheetFilter::Linear,
                _ => return Err(r.error("unknown filter")),
            },
            array: match r.u8()? {
                0 => false,
                1 => true,
                _ => return Err(r.error("invalid array flag")),
            },
            frames: Vec::new(),
            clips: Vec::new(),
        };
        for _ in 0..r.count(28)? {
            sheet.frames.push(SheetFrame {
                name: r.string()?,
                rect: [r.u32()?, r.u32()?, r.u32()?, r.u32()?],
                pivot: [r.f32()?, r.f32()?],
            });
        }
        for _ in 0..r.count(13)? {
            let name = r.string()?;
            let mode = match r.u8()? {
                0 => FlipMode::Loop,
                1 => FlipMode::Once,
                2 => FlipMode::PingPong,
                _ => return Err(r.error("unknown clip mode")),
            };
            let mut frames = Vec::new();
            let mut durations = Vec::new();
            for _ in 0..r.count(8)? {
                frames.push(r.u32()?);
                durations.push(r.f32()?);
            }
            let mut events = Vec::new();
            for _ in 0..r.count(8)? {
                events.push((r.u32()?, r.string()?));
            }
            sheet.clips.push(FlipClip {
                name,
                mode,
                frames,
                durations,
                events,
            });
        }
        r.finish()?;
        sheet.validate_native()?;
        Ok(sheet)
    }
}
