//! Scalar/velocity advection and Jacobi projection adapted to 3D from
//! AndrewBlackwell/fluid-sim (MIT); see third_party/stable-fluids/LICENSE.
use super::{Config, Fbm, NoiseKind};

struct Volume {
    n: usize,
    density: Vec<f32>,
    heat: Vec<f32>,
    velocity: Vec<[f32; 3]>,
    forcing: Vec<[f32; 3]>,
    source: Vec<f32>,
}
fn index(n: usize, x: usize, y: usize, z: usize) -> usize {
    (z * n + y) * n + x
}
fn sample<const C: usize>(n: usize, p: [f32; 3], at: impl Fn(usize) -> [f32; C]) -> [f32; C] {
    if p.iter().any(|v| *v < 0.0 || *v > (n - 1) as f32) {
        return [0.0; C];
    }
    let a = p.map(|v| v.floor() as usize);
    let f = std::array::from_fn::<_, 3, _>(|i| p[i] - a[i] as f32);
    let mut out = [0.0; C];
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let d = [x, y, z];
                let w: f32 = (0..3)
                    .map(|i| if d[i] == 0 { 1.0 - f[i] } else { f[i] })
                    .product();
                let v = at(index(
                    n,
                    (a[0] + x).min(n - 1),
                    (a[1] + y).min(n - 1),
                    (a[2] + z).min(n - 1),
                ));
                for i in 0..C {
                    out[i] += w * v[i];
                }
            }
        }
    }
    out
}
impl Volume {
    fn new(c: &Config, noise: &Fbm) -> Self {
        let n = c.grid;
        let mut out = Self {
            n,
            density: vec![0.0; n * n * n],
            heat: vec![0.0; n * n * n],
            velocity: vec![[0.0; 3]; n * n * n],
            forcing: vec![[0.0; 3]; n * n * n],
            source: vec![0.0; n * n * n],
        };
        for z in 1..n - 1 {
            for y in 1..n - 1 {
                for x in 1..n - 1 {
                    let i = index(n, x, y, z);
                    let p = [x, y, z].map(|v| v as f32 / (n - 1) as f32);
                    let delta = std::array::from_fn::<_, 3, _>(|j| p[j] - c.source[j]);
                    let r = delta.iter().map(|v| v * v).sum::<f32>().sqrt();
                    let detail = (1.0 + c.detail * noise.at(p[0], p[1], p[2], false)).max(0.0);
                    let shape = (1.0 - r / c.radius / detail.max(f32::EPSILON)).clamp(0.0, 1.0);
                    out.source[i] = shape * shape * (3.0 - 2.0 * shape) * detail;
                    out.forcing[i] = noise.curl3(p[0], p[1], p[2]);
                    if c.mode == "explosion" {
                        out.density[i] = out.source[i] * c.density;
                        out.heat[i] = out.source[i] * c.heat;
                        if r > 0.0 {
                            out.velocity[i] = delta.map(|v| v / r * c.expansion * out.source[i]);
                        }
                    }
                }
            }
        }
        out
    }
    fn step(&mut self, c: &Config, time: f32) {
        let n = self.n;
        let len = n * n * n;
        // Fedkiw et al.'s confinement: h * epsilon * (N cross curl(u)).
        // Cell-size scaling keeps the authored strength independent of grid.
        if c.vorticity > 0.0 {
            let mut curl = vec![[0.0; 3]; len];
            let mut magnitude = vec![0.0; len];
            let scale = (n - 1) as f32 * 0.5;
            for z in 1..n - 1 {
                for y in 1..n - 1 {
                    for x in 1..n - 1 {
                        let i = index(n, x, y, z);
                        let v = &self.velocity;
                        curl[i] = [
                            v[i + n][2] - v[i - n][2] - v[i + n * n][1] + v[i - n * n][1],
                            v[i + n * n][0] - v[i - n * n][0] - v[i + 1][2] + v[i - 1][2],
                            v[i + 1][1] - v[i - 1][1] - v[i + n][0] + v[i - n][0],
                        ]
                        .map(|v| v * scale);
                        magnitude[i] = curl[i].iter().map(|v| v * v).sum::<f32>().sqrt();
                    }
                }
            }
            for z in 1..n - 1 {
                for y in 1..n - 1 {
                    for x in 1..n - 1 {
                        let i = index(n, x, y, z);
                        let g = [1, n, n * n].map(|d| magnitude[i + d] - magnitude[i - d]);
                        let length = g.iter().map(|v| v * v).sum::<f32>().sqrt();
                        if length > 0.0 {
                            let a = g.map(|v| v / length);
                            let b = curl[i];
                            let force = [
                                a[1] * b[2] - a[2] * b[1],
                                a[2] * b[0] - a[0] * b[2],
                                a[0] * b[1] - a[1] * b[0],
                            ];
                            for (j, f) in force.iter().enumerate() {
                                self.velocity[i][j] += c.dt * c.vorticity * f / (n - 1) as f32;
                            }
                        }
                    }
                }
            }
        }
        for z in 1..n - 1 {
            for y in 1..n - 1 {
                for x in 1..n - 1 {
                    let i = index(n, x, y, z);
                    if c.mode != "explosion" && time < c.inject {
                        self.density[i] += c.density * self.source[i] * c.dt;
                        self.heat[i] += c.heat * self.source[i] * c.dt;
                        let delta = std::array::from_fn::<_, 3, _>(|j| {
                            [x, y, z][j] as f32 / (n - 1) as f32 - c.source[j]
                        });
                        let r = delta.iter().map(|v| v * v).sum::<f32>().sqrt();
                        if r > 0.0 {
                            for (j, d) in delta.iter().enumerate() {
                                self.velocity[i][j] += d / r * c.expansion * self.source[i] * c.dt;
                            }
                        }
                    }
                    for j in 0..3 {
                        self.velocity[i][j] += c.dt * c.turbulence * self.forcing[i][j];
                    }
                    self.velocity[i][1] += c.dt * c.buoyancy * self.heat[i];
                }
            }
        }
        // Zero inflow at the open domain boundary, not a wrapping smoke box.
        let mut velocity = vec![[0.0; 3]; len];
        for z in 1..n - 1 {
            for y in 1..n - 1 {
                for x in 1..n - 1 {
                    let i = index(n, x, y, z);
                    let p = std::array::from_fn(|j| {
                        [x, y, z][j] as f32 - self.velocity[i][j] * c.dt * (n - 1) as f32
                    });
                    velocity[i] = sample(n, p, |k| self.velocity[k]);
                }
            }
        }
        let mut div = vec![0.0; len];
        let mut pressure = vec![0.0; len];
        let mut next = pressure.clone();
        for z in 1..n - 1 {
            for y in 1..n - 1 {
                for x in 1..n - 1 {
                    let i = index(n, x, y, z);
                    div[i] = 0.5
                        * (velocity[i + 1][0] - velocity[i - 1][0] + velocity[i + n][1]
                            - velocity[i - n][1]
                            + velocity[i + n * n][2]
                            - velocity[i - n * n][2]);
                }
            }
        }
        for _ in 0..c.pressure {
            for z in 1..n - 1 {
                for y in 1..n - 1 {
                    for x in 1..n - 1 {
                        let i = index(n, x, y, z);
                        next[i] = (pressure[i - 1]
                            + pressure[i + 1]
                            + pressure[i - n]
                            + pressure[i + n]
                            + pressure[i - n * n]
                            + pressure[i + n * n]
                            - div[i])
                            / 6.0;
                    }
                }
            }
            std::mem::swap(&mut pressure, &mut next);
        }
        let mut density = vec![0.0; len];
        let mut heat = vec![0.0; len];
        for z in 1..n - 1 {
            for y in 1..n - 1 {
                for x in 1..n - 1 {
                    let i = index(n, x, y, z);
                    for (j, offset) in [1, n, n * n].into_iter().enumerate() {
                        velocity[i][j] -= 0.5 * (pressure[i + offset] - pressure[i - offset]);
                    }
                    let p = std::array::from_fn(|j| {
                        [x, y, z][j] as f32 - velocity[i][j] * c.dt * (n - 1) as f32
                    });
                    let values = sample(n, p, |k| [self.density[k], self.heat[k]]);
                    density[i] = values[0] * libm::expf(-c.decay * c.dt);
                    heat[i] = values[1] * libm::expf(-c.cooling * c.dt);
                }
            }
        }
        self.velocity = velocity;
        self.density = density;
        self.heat = heat;
    }

    /// Directional Beer-Lambert sweep along the light's dominant axis. Every
    /// interpolation reads the already-computed next plane, never itself.
    fn illumination(&self, c: &Config) -> Vec<f32> {
        let n = self.n;
        let length = c.light.iter().map(|v| v * v).sum::<f32>().sqrt();
        let direction = c.light.map(|v| v / length);
        let axis = (0..3)
            .max_by(|&a, &b| direction[a].abs().total_cmp(&direction[b].abs()))
            .unwrap();
        let step = direction.map(|v| v / direction[axis].abs());
        let distance = 1.0 / direction[axis].abs() / (n - 1) as f32;
        let mut trans = vec![1.0; n * n * n];
        for plane in 0..n {
            let major = if direction[axis] > 0.0 {
                n - 1 - plane
            } else {
                plane
            };
            for b in 0..n {
                for a in 0..n {
                    let mut at = [0; 3];
                    at[axis] = major;
                    at[(axis + 1) % 3] = a;
                    at[(axis + 2) % 3] = b;
                    let i = index(n, at[0], at[1], at[2]);
                    let next = std::array::from_fn(|j| at[j] as f32 + step[j]);
                    let outside = next.iter().any(|v| *v < 0.0 || *v > (n - 1) as f32);
                    let incoming = if outside {
                        1.0
                    } else {
                        sample(n, next, |k| [trans[k]])[0]
                    };
                    trans[i] = incoming * libm::expf(-self.density[i] * c.absorption * distance);
                }
            }
        }
        trans
            .into_iter()
            .map(|v| c.ambient + c.direct * v)
            .collect()
    }
}
fn srgb(v: f32) -> u8 {
    (255.0
        * if v <= 0.0031308 {
            12.92 * v
        } else {
            1.055 * libm::powf(v.max(0.0), 1.0 / 2.4) - 0.055
        })
    .clamp(0.0, 255.0)
    .round() as u8
}

/// Interleaved scattering RGBA and emission RGBA, straight alpha, sRGB.
pub(super) fn render(c: &Config) -> Vec<u8> {
    let n = c.grid;
    let noise = Fbm {
        seed: c.seed,
        frequency: c.frequency,
        kind: NoiseKind::Simplex,
        octaves: 3,
        lacunarity: 2.0,
        gain: 0.5,
    };
    let scalar = matches!(c.mode.as_str(), "noise" | "worley" | "curl");
    let mut volume = (!scalar).then(|| Volume::new(c, &noise));
    let mut out = Vec::with_capacity((c.size * c.size * c.frames * 8) as usize);
    for frame in 0..c.frames {
        if let Some(volume) = &mut volume {
            for step in 0..c.steps {
                volume.step(c, (frame * c.steps + step) as f32 * c.dt);
            }
        }
        let illumination = volume.as_ref().map(|v| v.illumination(c));
        for y in 0..c.size {
            for x in 0..c.size {
                let px = x as f32 / (c.size - 1) as f32;
                let py = 1.0 - y as f32 / (c.size - 1) as f32;
                if scalar {
                    let t = (frame + 1) as f32 * c.steps as f32 * c.dt * c.turbulence;
                    let rgb = match c.mode.as_str() {
                        "curl" => noise.curl3(px, py, t).map(|v| 0.5 + 0.5 * v / c.range),
                        "worley" => {
                            [og_core::noise::worley3(
                                c.seed,
                                px * c.frequency,
                                py * c.frequency,
                                t * c.frequency,
                            )
                            .f1 / c.range; 3]
                        }
                        _ => [0.5 + 0.5 * noise.at(px, py, t, false) / c.range; 3],
                    };
                    let bytes = rgb.map(|v| (v.clamp(0.0, 1.0) * 255.0).round() as u8);
                    out.extend_from_slice(&[bytes[0], bytes[1], bytes[2], 255, 0, 0, 0, 255]);
                    continue;
                }
                let volume = volume.as_ref().expect("volume mode");
                let illumination = illumination.as_ref().expect("volume illumination");
                let mut trans = 1.0;
                let mut color = [0.0; 3];
                let mut emission = [0.0; 3];
                for z in 0..n * 2 {
                    let p = [px * (n - 1) as f32, py * (n - 1) as f32, z as f32 / 2.0];
                    let [density, heat, shade] = sample(n, p, |i| {
                        [volume.density[i], volume.heat[i], illumination[i]]
                    });
                    let opacity = 1.0 - libm::expf(-density * c.absorption / ((n - 1) * 2) as f32);
                    for j in 0..3 {
                        color[j] += trans * opacity * c.color[j] * shade;
                        emission[j] += trans * opacity * c.emission[j] * heat;
                    }
                    trans *= 1.0 - opacity;
                }
                let alpha = 1.0 - trans;
                let color = color.map(|v| srgb(if alpha > 0.0 { v / alpha } else { 0.0 }));
                let emission = emission.map(|v| srgb(if alpha > 0.0 { v / alpha } else { 0.0 }));
                let a = (alpha * 255.0).round() as u8;
                out.extend_from_slice(&[
                    color[0],
                    color[1],
                    color[2],
                    a,
                    emission[0],
                    emission[1],
                    emission[2],
                    a,
                ]);
            }
        }
    }
    out
}
