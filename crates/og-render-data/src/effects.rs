use glam::Vec3;

/// The LUT side every renderer expects.
pub const LUT_SIZE: u32 = 32;

/// A 32³ colour lookup table: `rgba` holds `size³` RGBA8 texels, red
/// fastest, then green, then blue (the `.cube` order), mapping a display
/// (sRGB-encoded) colour to its graded display colour.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorLut {
    pub size: u32,
    pub rgba: Vec<u8>,
}

impl ColorLut {
    /// The identity table: grading with it changes nothing (within the LUT's
    /// 8-bit precision).
    pub fn identity() -> ColorLut {
        ColorLut::from_fn(|c| c)
    }

    /// A table made by `f(display rgb in 0..1) -> display rgb`.
    pub fn from_fn(f: impl Fn(Vec3) -> Vec3) -> ColorLut {
        let n = LUT_SIZE;
        let mut rgba = Vec::with_capacity((n * n * n * 4) as usize);
        let max = (n - 1) as f32;
        for b in 0..n {
            for g in 0..n {
                for r in 0..n {
                    let c = f(Vec3::new(r as f32 / max, g as f32 / max, b as f32 / max));
                    let q = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    rgba.extend_from_slice(&[q(c.x), q(c.y), q(c.z), 255]);
                }
            }
        }
        ColorLut { size: n, rgba }
    }

    /// Parses an Adobe/Resolve `.cube` file with `LUT_3D_SIZE 32` (other
    /// sizes are resampled to 32 with trilinear filtering). `DOMAIN_MIN` and
    /// `DOMAIN_MAX` must be 0 and 1.
    pub fn from_cube(text: &str) -> Result<ColorLut, String> {
        let mut size = 0u32;
        let mut values: Vec<Vec3> = Vec::new();
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with("TITLE") {
                continue;
            }
            let mut parts = line.split_whitespace();
            let first = parts.next().unwrap_or("");
            match first {
                "LUT_3D_SIZE" => {
                    size = parts
                        .next()
                        .and_then(|v| v.parse().ok())
                        .ok_or_else(|| format!("line {}: bad LUT_3D_SIZE", line_no + 1))?;
                }
                "LUT_1D_SIZE" => return Err("a 1D .cube LUT is not a 3D colour table".into()),
                "DOMAIN_MIN" | "DOMAIN_MAX" => {
                    let want = if first == "DOMAIN_MIN" { 0.0 } else { 1.0 };
                    let ok = parts.all(|v| v.parse::<f32>().is_ok_and(|x| x == want));
                    if !ok {
                        return Err(format!("line {}: {first} must be {want}", line_no + 1));
                    }
                }
                _ => {
                    let nums: Result<Vec<f32>, _> =
                        line.split_whitespace().map(str::parse::<f32>).collect();
                    match nums {
                        Ok(v) if v.len() == 3 => values.push(Vec3::new(v[0], v[1], v[2])),
                        _ => return Err(format!("line {}: '{line}'", line_no + 1)),
                    }
                }
            }
        }
        if size < 2 || values.len() != (size * size * size) as usize {
            return Err(format!(
                "LUT_3D_SIZE {size} needs {} entries, found {}",
                size * size * size,
                values.len()
            ));
        }
        let at = |r: u32, g: u32, b: u32| values[(r + g * size + b * size * size) as usize];
        Ok(ColorLut::from_fn(|c| {
            let p = c * (size - 1) as f32;
            let i = p.floor().min(Vec3::splat((size - 2) as f32));
            let f = p - i;
            let (r, g, b) = (i.x as u32, i.y as u32, i.z as u32);
            let lerp = |a: Vec3, b: Vec3, t: f32| a + (b - a) * t;
            let c00 = lerp(at(r, g, b), at(r + 1, g, b), f.x);
            let c10 = lerp(at(r, g + 1, b), at(r + 1, g + 1, b), f.x);
            let c01 = lerp(at(r, g, b + 1), at(r + 1, g, b + 1), f.x);
            let c11 = lerp(at(r, g + 1, b + 1), at(r + 1, g + 1, b + 1), f.x);
            lerp(lerp(c00, c10, f.y), lerp(c01, c11, f.y), f.z)
        }))
    }

    /// Reads a LUT laid out as a horizontal strip of `size` slices (the
    /// Unity/Unreal 1024x32 PNG): slice `b` holds red along x, green along y
    /// (row 0 is green 0). `rgba` is the strip's RGBA8 pixels, rows top to bottom.
    pub fn from_strip(width: u32, height: u32, rgba: &[u8]) -> Result<ColorLut, String> {
        if height == 0 || width != height * height || rgba.len() != (width * height * 4) as usize {
            return Err(format!(
                "a LUT strip is (n*n)xn RGBA8 pixels; got {width}x{height} with {} bytes",
                rgba.len()
            ));
        }
        let n = height;
        let at = |r: u32, g: u32, b: u32| {
            let i = ((g * width + b * n + r) * 4) as usize;
            Vec3::new(rgba[i] as f32, rgba[i + 1] as f32, rgba[i + 2] as f32) / 255.0
        };
        if n == LUT_SIZE {
            let mut out = Vec::with_capacity(rgba.len());
            for b in 0..n {
                for g in 0..n {
                    for r in 0..n {
                        let i = ((g * width + b * n + r) * 4) as usize;
                        out.extend_from_slice(&[rgba[i], rgba[i + 1], rgba[i + 2], 255]);
                    }
                }
            }
            return Ok(ColorLut { size: n, rgba: out });
        }
        Ok(ColorLut::from_fn(|c| {
            let p = (c * (n - 1) as f32).round();
            at(p.x as u32, p.y as u32, p.z as u32)
        }))
    }
}

impl ColorLut {
    pub fn validate(&self) -> Result<(), String> {
        let n = self.size as usize;
        if self.size != LUT_SIZE || self.rgba.len() != n * n * n * 4 {
            return Err(format!(
                "a colour LUT is {LUT_SIZE}³ RGBA8 ({} bytes); got size {} with {} bytes",
                LUT_SIZE.pow(3) * 4,
                self.size,
                self.rgba.len()
            ));
        }
        Ok(())
    }
}
