//! Painted 2D: the look's `Stylise` paint fields read as texel space
//! (docs/art.md §2.3), so the stroke follows the sprite, not the screen.
//! The same steps as the frame's paint pass (stylise.wgsl `paint`): the
//! Kuwahara filter with its sectors turned along the edge by
//! `paintAnisotropy`, colour bleeding from a ring two radii out, pigment
//! pooling where the tone changes, the canvas's grain; radii in the
//! texture's texels, the canvas `paintCanvasScale` canvas texels per
//! texel from each cell's corner. A transparent texel stays transparent
//! and lends no colour: where the frame reads a neighbour off the shape,
//! this reads the texel itself.

use super::colour;

/// The paint fields, at their frame meaning.
#[derive(Clone, Debug, Default)]
pub struct Paint {
    pub radius: f64,
    pub anisotropy: f64,
    pub edge: f64,
    pub bleed: f64,
    /// The canvas's grain (its red channel, linear) and size.
    pub canvas: Option<(Vec<f64>, u32, u32)>,
    pub canvas_strength: f64,
    pub canvas_scale: f64,
}

impl Paint {
    pub fn active(&self) -> bool {
        self.radius > 0.0
            || self.bleed > 0.0
            || self.edge > 0.0
            || (self.canvas.is_some() && self.canvas_strength > 0.0)
    }

    pub fn describe(&self) -> String {
        let mut v = Vec::new();
        if self.radius > 0.0 {
            v.push(format!("Kuwahara radius {} texels", self.radius));
            if self.anisotropy > 0.0 {
                v.push(format!("anisotropy {}", self.anisotropy));
            }
        }
        if self.bleed > 0.0 {
            v.push(format!("bleed {}", self.bleed));
        }
        if self.edge > 0.0 {
            v.push(format!("pigment pooling {}", self.edge));
        }
        if self.canvas.is_some() && self.canvas_strength > 0.0 {
            v.push(format!("canvas grain {}", self.canvas_strength));
        }
        v.join(", ")
    }

    /// Paints one `w` x `h` cell of RGBA texels in place.
    pub fn apply(&self, px: &mut [[u8; 4]], w: u32, h: u32) {
        let lin: Vec<[f64; 3]> = px
            .iter()
            .map(|p| {
                [
                    colour::to_linear(p[0]),
                    colour::to_linear(p[1]),
                    colour::to_linear(p[2]),
                ]
            })
            .collect();
        let opaque = |x: i64, y: i64| -> bool {
            x >= 0
                && y >= 0
                && x < i64::from(w)
                && y < i64::from(h)
                && px[(y as u32 * w + x as u32) as usize][3] > 0
        };
        // `load`: the texel, or the one asked from where the shape is not.
        let load = |x: i64, y: i64, from: (i64, i64)| -> [f64; 3] {
            let (x, y) = if opaque(x, y) { (x, y) } else { from };
            lin[(y as u32 * w + x as u32) as usize]
        };
        let tone = |c: [f64; 3]| {
            let e = c.map(|v| f64::from(colour::to_srgb8(v)) / 255.0);
            0.2126 * e[0] + 0.7152 * e[1] + 0.0722 * e[2]
        };
        let gradient = |x: i64, y: i64, src: &dyn Fn(i64, i64) -> [f64; 3]| -> [f64; 2] {
            let t = |dx: i64, dy: i64| tone(src(x + dx, y + dy));
            [
                (t(1, -1) + 2.0 * t(1, 0) + t(1, 1)) - (t(-1, -1) + 2.0 * t(-1, 0) + t(-1, 1)),
                (t(-1, 1) + 2.0 * t(0, 1) + t(1, 1)) - (t(-1, -1) + 2.0 * t(0, -1) + t(1, -1)),
            ]
        };
        let mut out = lin.clone();
        for y in 0..i64::from(h) {
            for x in 0..i64::from(w) {
                if !opaque(x, y) {
                    continue;
                }
                let at = (x, y);
                let src = |a: i64, b: i64| load(a, b, at);
                let mut c = lin[(y as u32 * w + x as u32) as usize];
                if self.radius > 0.0 {
                    c = self.kuwahara(x, y, &opaque, &lin, w, gradient(x, y, &src));
                }
                if self.bleed > 0.0 {
                    let d = self.radius.max(1.0) as i64 * 2;
                    let ring = [src(x + d, y), src(x - d, y), src(x, y + d), src(x, y - d)];
                    for k in 0..3 {
                        let around = ring.iter().map(|r| r[k]).sum::<f64>() * 0.25;
                        c[k] += (around - c[k]) * self.bleed * 0.5;
                    }
                }
                if self.edge > 0.0 {
                    let g = gradient(x, y, &src);
                    let m = (g[0] * g[0] + g[1] * g[1]).sqrt();
                    let k = 1.0 - self.edge * (m * 1.5).clamp(0.0, 1.0) * 0.6;
                    c = c.map(|v| v * k);
                }
                if let Some((grain, gw, gh)) = &self.canvas
                    && self.canvas_strength > 0.0
                {
                    let g = sample_repeat(
                        grain,
                        *gw,
                        *gh,
                        (x as f64 + 0.5) * self.canvas_scale,
                        (y as f64 + 0.5) * self.canvas_scale,
                    );
                    let k = 1.0 + (g * 0.5 + 0.5 - 1.0) * self.canvas_strength;
                    c = c.map(|v| v * k);
                }
                out[(y as u32 * w + x as u32) as usize] = c;
            }
        }
        for (p, c) in px.iter_mut().zip(out) {
            if p[3] > 0 {
                p[0] = colour::to_srgb8(c[0]);
                p[1] = colour::to_srgb8(c[1]);
                p[2] = colour::to_srgb8(c[2]);
            }
        }
    }

    /// stylise.wgsl's `kuwahara` at one texel; transparent texels are left
    /// out of the sectors.
    fn kuwahara(
        &self,
        x: i64,
        y: i64,
        opaque: &dyn Fn(i64, i64) -> bool,
        lin: &[[f64; 3]],
        w: u32,
        g: [f64; 2],
    ) -> [f64; 3] {
        let r = self.radius.ceil() as i64;
        let mag = (g[0] * g[0] + g[1] * g[1]).sqrt();
        // Strokes run along the edge, perpendicular to the gradient: the
        // turn's cosine and sine straight from it (no angle).
        let (mut ca, mut sa, mut stretch) = (1.0, 0.0, 1.0);
        if self.anisotropy > 0.0 && mag > 1e-3 {
            ca = -g[1] / mag;
            sa = g[0] / mag;
            stretch = 1.0 + self.anisotropy * 2.0;
        }
        let mut best = lin[(y as u32 * w + x as u32) as usize];
        let mut best_var = f64::MAX;
        for s in 0..4 {
            let sx = if s & 1 == 1 { 1 } else { -1 };
            let sy = if s & 2 == 2 { 1 } else { -1 };
            let (mut sum, mut sum2, mut n) = ([0.0f64; 3], [0.0f64; 3], 0.0f64);
            for oy in 0..=r {
                for ox in 0..=r {
                    let o = [(ox * sx) as f64, (oy * sy) as f64];
                    let t = [o[0] * ca - o[1] * sa, o[0] * sa + o[1] * ca];
                    let e = [t[0] * stretch, t[1] / stretch];
                    if (e[0] * e[0] + e[1] * e[1]).sqrt() > self.radius * stretch {
                        continue;
                    }
                    let (qx, qy) = (x + e[0].round() as i64, y + e[1].round() as i64);
                    if !opaque(qx, qy) {
                        continue;
                    }
                    let c = lin[(qy as u32 * w + qx as u32) as usize];
                    for k in 0..3 {
                        sum[k] += c[k];
                        sum2[k] += c[k] * c[k];
                    }
                    n += 1.0;
                }
            }
            if n == 0.0 {
                continue;
            }
            let mean = sum.map(|v| v / n);
            let var: f64 = (0..3).map(|k| sum2[k] / n - mean[k] * mean[k]).sum();
            if var < best_var {
                best_var = var;
                best = mean;
            }
        }
        best
    }
}

/// A bilinear, repeating sample of a grain texture at texel coordinates.
fn sample_repeat(g: &[f64], w: u32, h: u32, x: f64, y: f64) -> f64 {
    let (fx, fy) = (x - 0.5, y - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (tx, ty) = (fx - x0, fy - y0);
    let at = |x: f64, y: f64| {
        let xi = (x as i64).rem_euclid(i64::from(w)) as u32;
        let yi = (y as i64).rem_euclid(i64::from(h)) as u32;
        g[(yi * w + xi) as usize]
    };
    let top = at(x0, y0) * (1.0 - tx) + at(x0 + 1.0, y0) * tx;
    let bottom = at(x0, y0 + 1.0) * (1.0 - tx) + at(x0 + 1.0, y0 + 1.0) * tx;
    top * (1.0 - ty) + bottom * ty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kuwahara_keeps_an_edge_and_flattens_noise_beside_it() {
        // 6 x 1: three dark texels with one speck, then three light.
        let k = [40u8, 40, 40, 255];
        let mut px = vec![
            k,
            [60, 60, 60, 255],
            k,
            [220, 220, 220, 255],
            [220, 220, 220, 255],
            [220, 220, 220, 255],
        ];
        let p = Paint {
            radius: 1.0,
            ..Paint::default()
        };
        p.apply(&mut px, 6, 1);
        // The speck joins its side (the sector without the edge has the
        // least variance); the edge between 2 and 3 stays where it was.
        assert!(px[1][0] < 60 && px[1][0] >= 40, "{:?}", px[1]);
        assert!(px[2][0] < 100, "{:?}", px[2]);
        assert_eq!(px[3], [220, 220, 220, 255]);
    }

    #[test]
    fn pooling_darkens_the_edges_of_a_region_not_its_middle_or_the_clear() {
        let mut px = vec![[200, 200, 200, 255]; 5 * 5];
        px[12] = [20, 20, 20, 255];
        px[0] = [0, 0, 0, 0];
        let p = Paint {
            edge: 1.0,
            ..Paint::default()
        };
        p.apply(&mut px, 5, 5);
        // Next to the dark texel the light ones darken; far from it not;
        // the clear corner stays clear and lends no edge.
        assert!(px[11][0] < 200, "{:?}", px[11]);
        assert_eq!(px[4], [200, 200, 200, 255]);
        assert_eq!(px[0], [0, 0, 0, 0]);
        assert_eq!(px[1], [200, 200, 200, 255]);
    }
}
