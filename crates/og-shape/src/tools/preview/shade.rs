//! A part's look: its material's maps decoded, normals by crease, each triangle shaded and drawn.

use super::*;

/// The decoded maps of a scene's parts, each distinct one once: the
/// textures and their chains (or why not), in first-use order.
pub(crate) struct Maps<'a> {
    /// Each texture with whether it was read as data, and its chain.
    pub(crate) seen: Vec<(&'a Texture, bool, Result<Mips, String>)>,
    /// What decodes a texture's bytes.
    pub(crate) decode: fn(&[u8]) -> Result<Image, String>,
}

impl<'a> Maps<'a> {
    pub(crate) fn new(codec: &Codec) -> Maps<'a> {
        Maps {
            seen: Vec::new(),
            decode: codec.decode,
        }
    }

    /// The index of `t`'s chain, decoding it the first time.
    pub(crate) fn get(&mut self, t: &'a Texture) -> Result<usize, String> {
        self.chain(t, false)
    }

    /// As [`Maps::get`], for a map of data (a normal map).
    pub(crate) fn get_data(&mut self, t: &'a Texture) -> Result<usize, String> {
        self.chain(t, true)
    }

    pub(crate) fn chain(&mut self, t: &'a Texture, data: bool) -> Result<usize, String> {
        let k = match self.seen.iter().position(|(s, d, _)| *s == t && *d == data) {
            Some(k) => k,
            None => {
                let m = (self.decode)(&t.bytes).map(if data { Mips::new_data } else { Mips::new });
                self.seen.push((t, data, m));
                self.seen.len() - 1
            }
        };
        match &self.seen[k].2 {
            Ok(_) => Ok(k),
            Err(e) => Err(format!("map {:?}: {e}", t.name)),
        }
    }

    pub(crate) fn mips(&self, k: usize) -> Option<&Mips> {
        self.seen.get(k).and_then(|(_, _, m)| m.as_ref().ok())
    }
}

/// How a part is coloured per pixel, when it is.
pub(crate) struct Paint {
    /// The map's chain in [`Maps`], with the part's UVs.
    pub(crate) map: Option<(usize, Vec<[DVec2; 3]>)>,
    /// The normal map's chain, the part's UVs and the map's scale.
    pub(crate) normal: Option<(usize, Vec<[DVec2; 3]>, f64)>,
    /// Linear RGBA per triangle corner.
    pub(crate) colors: Option<Vec<crate::mesh::CornerColors>>,
    /// The material's colour in linear light.
    pub(crate) base: [f64; 3],
    /// What the surface does with light besides its colour, if anything.
    pub(crate) gloss: Option<Gloss>,
}

/// A material that is a metal, smoother than matte, or has maps for
/// those, for occlusion or for its glow. Each map is its chain in
/// [`Maps`]; they share the part's UVs.
pub(crate) struct Gloss {
    pub(crate) metal: f64,
    pub(crate) rough: f64,
    /// Roughness in green, metalness in blue, times the factors.
    pub(crate) metal_rough: Option<usize>,
    /// Occlusion in red, and how much of it counts.
    pub(crate) occlusion: Option<(usize, f64)>,
    /// The colour the emissive light is multiplied by.
    pub(crate) glow: Option<usize>,
    pub(crate) uvs: Option<Vec<[DVec2; 3]>>,
    /// A clear layer and its roughness; a sheen's colour and spread;
    /// how much passes through; a thin film's share.
    pub(crate) coat: (f64, f64),
    pub(crate) sheen: ([f64; 3], f64),
    pub(crate) transmission: f64,
    pub(crate) iridescence: f64,
}

/// The part's [`Paint`], if it has a map it can draw or vertex colours;
/// what it cannot draw goes to `undrawn`.
pub(crate) fn paint<'a>(
    name: &str,
    s: &Solid,
    material: &'a Material,
    rgb: [f64; 3],
    maps: &mut Maps<'a>,
    undrawn: &mut Vec<(String, String)>,
    lit_round: bool,
) -> Option<Paint> {
    let n = s.triangles.len();
    let map = material.map.as_ref();
    let normal = material
        .normal_map
        .as_ref()
        .map(|t| (t, material.normal_scale.unwrap_or(1.0)));
    // The maps of the surface's gloss, each read with the part's UVs.
    let uvs = s.uvs.as_ref().filter(|uv| uv.len() == n);
    let mut chain = |t: Option<&'a Texture>, color: bool, what: &str| -> Option<usize> {
        let t = t?;
        let why = match uvs {
            None => format!("the part has no UVs to sample its {what} with"),
            Some(_) => match if color { maps.get(t) } else { maps.get_data(t) } {
                Ok(k) => return Some(k),
                Err(e) => e,
            },
        };
        undrawn.push((name.to_string(), why));
        None
    };
    let metal_rough = chain(material.metalness_map.as_ref(), false, "metalness map");
    let occlusion = chain(material.ao_map.as_ref(), false, "occlusion map")
        .map(|k| (k, material.ao_map_intensity.unwrap_or(1.0).clamp(0.0, 1.0)));
    let glow = chain(material.emissive_map.as_ref(), true, "emissive map");
    let metal = material.metalness.clamp(0.0, 1.0);
    let rough = material.roughness.unwrap_or(1.0).clamp(0.0, 1.0);
    let unit = |v: f64| {
        if v.is_finite() {
            v.clamp(0.0, 1.0)
        } else {
            0.0
        }
    };
    let coat = (unit(material.clearcoat), unit(material.clearcoat_roughness));
    let sheen = (material.sheen.map(unit), unit(material.sheen_roughness));
    let (transmission, iridescence) = (unit(material.transmission), unit(material.iridescence));
    // A picture all round lights every surface, matte ones too.
    let glossy = metal > 0.0
        || rough < 1.0
        || coat.0 > 0.0
        || sheen.0 != [0.0; 3]
        || transmission > 0.0
        || iridescence > 0.0
        || lit_round;
    let gloss =
        (glossy || metal_rough.is_some() || occlusion.is_some() || glow.is_some()).then(|| Gloss {
            metal,
            rough,
            metal_rough,
            occlusion,
            glow,
            uvs: uvs.cloned(),
            coat,
            sheen,
            transmission,
            iridescence,
        });
    let normal = normal.and_then(|(t, scale)| {
        let why = match &s.uvs {
            None => "the part has no UVs to sample its normal map with".to_string(),
            Some(uv) if uv.len() != n => "the part's UVs are not one per triangle".to_string(),
            Some(uv) => match maps.get_data(t) {
                Ok(k) => return Some((k, uv.clone(), scale)),
                Err(e) => e,
            },
        };
        undrawn.push((name.to_string(), why));
        None
    });
    let map = map.and_then(|t| {
        let why = match &s.uvs {
            None => "the part has no UVs to sample its map with".to_string(),
            Some(uv) if uv.len() != n => "the part's UVs are not one per triangle".to_string(),
            Some(uv) => match maps.get(t) {
                Ok(k) => return Some((k, uv.clone())),
                Err(e) => e,
            },
        };
        undrawn.push((name.to_string(), why));
        None
    });
    let colors = s.colors.as_ref().and_then(|c| {
        if c.len() == n && c.iter().flatten().flatten().all(|v| v.is_finite()) {
            Some(c.clone())
        } else {
            undrawn.push((
                name.to_string(),
                "the part's vertex colours are not one finite set per triangle".to_string(),
            ));
            None
        }
    });
    if map.is_none() && colors.is_none() && normal.is_none() && gloss.is_none() {
        return None;
    }
    Some(Paint {
        map,
        normal,
        colors,
        base: rgb.map(|c| to_linear(c.clamp(0.0, 1.0))),
        gloss,
    })
}

/// A part as drawn, posed.
pub(crate) struct Drawn {
    /// Each triangle's corners.
    pub(crate) tris: Vec<[DVec3; 3]>,
    /// Each triangle's corner normals ([`corner_normals`]).
    pub(crate) normals: Vec<[DVec3; 3]>,
    /// The real edges.
    pub(crate) edges: Vec<(DVec3, DVec3)>,
    /// sRGB colour.
    pub(crate) rgb: [f64; 3],
    /// Light given off, linear, added to the lit colour.
    pub(crate) emissive: [f64; 3],
    /// In 0..=1; 1 is opaque.
    pub(crate) opacity: f64,
    /// Per-pixel colour, if the part has a map or vertex colours.
    pub(crate) paint: Option<Paint>,
}

/// What one view's triangles are drawn with.
pub(crate) struct Look<'a> {
    pub(crate) cam: &'a Camera,
    pub(crate) light: DVec3,
    /// From the scene to the camera (orthographic); the view's axis.
    pub(crate) toward: DVec3,
    pub(crate) unlit: bool,
    pub(crate) maps: &'a Maps<'a>,
    /// The picture all round, if one was given.
    pub(crate) environment: Option<&'a Environment>,
}

/// Per triangle, each corner's shading normal: the area-weighted sum of
/// the normals of the triangles around its vertex that share the
/// triangle's surface and whose unit normals have a dot product of at
/// least `crease` (the cosine of [`CREASE_DEGREES`]) with its own,
/// normalized. The triangle itself always counts unless it is degenerate;
/// a zero sum gives +Z (only degenerate triangles have one).
pub(crate) fn corner_normals(s: &Solid, crease: f64) -> Vec<[DVec3; 3]> {
    let weighted: Vec<DVec3> = s
        .triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|v| s.positions[v as usize]);
            (b - a).cross(c - a)
        })
        .collect();
    let unit: Vec<DVec3> = s
        .triangles
        .iter()
        .map(|t| {
            let [a, b, c] = t.map(|v| s.positions[v as usize]);
            face_normal(a, b, c)
        })
        .collect();
    let mut around: Vec<Vec<u32>> = vec![Vec::new(); s.positions.len()];
    for (i, t) in s.triangles.iter().enumerate() {
        for &v in t {
            around[v as usize].push(i as u32);
        }
    }
    s.triangles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            t.map(|v| {
                let mut n = DVec3::ZERO;
                for &j in &around[v as usize] {
                    let j = j as usize;
                    if s.surfaces[j] == s.surfaces[i] && unit[j].dot(unit[i]) >= crease {
                        n += weighted[j];
                    }
                }
                let len = n.length();
                if len > 0.0 { n / len } else { DVec3::Z }
            })
        })
        .collect()
}

/// Triangle `k` of part `p`: opaque into the depth buffer, or blended
/// over what is drawn, tested against the depth but not writing it.
pub(crate) fn draw_triangle(t: &mut Target, look: &Look, p: &Drawn, k: usize) {
    let dims = (t.width, t.height);
    let q = p.tris[k];
    // Lit from the side the camera sees.
    let toward = match look.cam.eye {
        None => look.toward,
        Some((eye, _)) => eye - q[0],
    };
    let flip = if face_normal(q[0], q[1], q[2]).dot(toward) < 0.0 {
        -1.0
    } else {
        1.0
    };
    let c = [0, 1, 2].map(|i| {
        let n = p.normals[k][i] * flip;
        let light = if look.unlit {
            1.0
        } else {
            AMBIENT + DIFFUSE * n.dot(look.light).max(0.0)
        };
        look.cam.corner(q[i], light)
    });
    let persp = look.cam.eye.is_some();
    let (rgb, alpha, e) = (p.rgb, p.opacity, p.emissive);
    // The lit sRGB component `c × l`, plus the emissive light (none: the
    // product as it was).
    let glow = |c: f64, l: f64, k: usize| {
        if e[k] == 0.0 {
            c * l
        } else {
            to_srgb((to_linear((c * l).clamp(0.0, 1.0)) + e[k]).min(1.0))
        }
    };
    let (depth, ground, out) = (&mut t.depth, &mut t.ground, &mut t.rgb);
    let Some(paint) = &p.paint else {
        raster(dims, c, persp, false, |f| {
            let (i, z, l) = (f.i, f.z, f.light);
            if z >= depth[i] {
                return;
            }
            if alpha == 1.0 {
                depth[i] = z;
                ground[i] = false;
                out[i] = [0, 1, 2].map(|c| byte(glow(rgb[c], l, c)));
            } else {
                let under = out[i];
                out[i] = [0, 1, 2].map(|c| {
                    byte(glow(rgb[c], l, c) * alpha + f64::from(under[c]) / 255.0 * (1.0 - alpha))
                });
            }
        });
        return;
    };
    let map = paint
        .map
        .as_ref()
        .and_then(|(m, uvs)| Some((look.maps.mips(*m)?, &uvs[k])));
    let colors = paint.colors.as_ref().map(|c| &c[k]);
    // A normal map bends the light per pixel: the triangle's tangent
    // frame from its positions and UVs (u along the tangent, v along the
    // bitangent, the image's green up), round the interpolated normal.
    let bump = paint.normal.as_ref().and_then(|(m, uvs, scale)| {
        if look.unlit {
            return None;
        }
        let uv = &uvs[k];
        let (e1, e2) = (q[1] - q[0], q[2] - q[0]);
        let (d1, d2) = (uv[1] - uv[0], uv[2] - uv[0]);
        let det = d1.x * d2.y - d2.x * d1.y;
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        let tangent = (e1 * d2.y - e2 * d1.y) / det;
        let bitangent = (e2 * d1.x - e1 * d2.x) / det;
        Some((look.maps.mips(*m)?, uv, *scale, tangent, bitangent))
    });
    let corner_normals = p.normals[k].map(|n| n * flip);
    let gloss = paint.gloss.as_ref();
    let view = toward.try_normalize().unwrap_or(DVec3::Z);
    raster(dims, c, persp, map.is_some() || bump.is_some(), |f| {
        let (i, z, mut l) = (f.i, f.z, f.light);
        if z >= depth[i] {
            return;
        }
        // The normal the light meets here: the map's, where it bends it.
        let mut facing = None;
        if let Some((mips, uv, scale, tangent, bitangent)) = bump {
            let at = mix2(f.w, uv);
            let s = mips.sample(at, 0);
            let n = corner_normals[0] * f.w[0]
                + corner_normals[1] * f.w[1]
                + corner_normals[2] * f.w[2];
            if let (Some(n), true) = (n.try_normalize(), s.iter().all(|v| v.is_finite())) {
                let t = tangent - n * n.dot(tangent);
                if let Some(t) = t.try_normalize() {
                    let hand = if n.cross(t).dot(bitangent) < 0.0 {
                        -1.0
                    } else {
                        1.0
                    };
                    let b = n.cross(t) * hand;
                    let (x, y, zz) = (
                        (s[0] * 2.0 - 1.0) * scale,
                        (s[1] * 2.0 - 1.0) * scale,
                        s[2] * 2.0 - 1.0,
                    );
                    if let Some(bent) = (t * x + b * y + n * zz).try_normalize() {
                        l = AMBIENT + DIFFUSE * bent.dot(look.light).max(0.0);
                        facing = Some(bent);
                    }
                }
            }
        }
        let b = paint.base;
        let mut col = [b[0], b[1], b[2], 1.0];
        if let Some((mips, uv)) = map {
            let at = mix2(f.w, uv);
            let (tw, th) = mips.size();
            let footprint = |w: [f64; 3]| {
                let d = mix2(w, uv) - at;
                let (du, dv) = (d.x * tw, d.y * th);
                du * du + dv * dv
            };
            let rho2 = footprint(f.wx).max(footprint(f.wy));
            let s = mips.sample(at, mips.level(rho2));
            for (c, v) in col.iter_mut().zip(s) {
                *c *= v;
            }
        }
        if let Some(vc) = colors {
            let vc = vc.map(crate::mesh::wide);
            for (ch, c) in col.iter_mut().enumerate() {
                *c *= f.w[0] * vc[0][ch] + f.w[1] * vc[1][ch] + f.w[2] * vc[2][ch];
            }
        }
        let mut transmitted = 0.0;
        let lit: [f64; 3] = match gloss {
            None => {
                let srgb = [0, 1, 2].map(|c| to_srgb(col[c].clamp(0.0, 1.0)));
                [0, 1, 2].map(|c| glow(srgb[c], l, c))
            }
            Some(g) => {
                let at = g.uvs.as_ref().map(|uv| mix2(f.w, &uv[k]));
                let read = |m: Option<usize>| {
                    let s = look.maps.mips(m?)?.sample(at?, 0);
                    s.iter().all(|v| v.is_finite()).then_some(s)
                };
                let (mut metal, mut rough) = (g.metal, g.rough);
                if let Some(s) = read(g.metal_rough) {
                    rough *= s[1];
                    metal *= s[2];
                }
                let shade = match g.occlusion.and_then(|(m, by)| Some((read(Some(m))?, by))) {
                    Some((s, by)) => 1.0 + by * (s[0] - 1.0),
                    None => 1.0,
                };
                let tint = read(g.glow).unwrap_or([1.0; 4]);
                let n = facing.or_else(|| {
                    (corner_normals[0] * f.w[0]
                        + corner_normals[1] * f.w[1]
                        + corner_normals[2] * f.w[2])
                        .try_normalize()
                });
                let towards = n.map_or(0.0, |n| n.dot(look.light).max(0.0));
                // The light from all round: even, or the picture's from
                // the side the surface faces; occlusion dims it, not
                // the sun.
                let round = match (look.environment, n) {
                    (Some(env), Some(n)) => env.gathered(n),
                    _ => [1.0; 3],
                };
                let mut lights = [l; 3];
                if !look.unlit && (shade != 1.0 || look.environment.is_some()) {
                    lights = round.map(|r| AMBIENT * shade * r + DIFFUSE * towards);
                }
                // What is reflected: a highlight of the light, and the
                // sky over the ground (or the picture), blurred as the
                // surface roughens.
                let mut mirror = [0.0; 3];
                let extras = g.coat.0 > 0.0 || g.sheen.0 != [0.0; 3] || g.iridescence > 0.0;
                if let (Some(n), false, true) = (
                    n,
                    look.unlit,
                    metal > 0.0 || rough < 1.0 || extras || look.environment.is_some(),
                ) {
                    let smooth = 1.0 - rough;
                    let facing_view = n.dot(view).max(0.0);
                    let back = n * (2.0 * n.dot(view)) - view;
                    // A lobe of roughness r: its highlight, and what it
                    // reflects of all round.
                    let lobe = |r: f64| -> (f64, [f64; 3]) {
                        let alpha2 = (r * r * r * r).max(1e-6);
                        let power = (2.0 / alpha2 - 2.0).clamp(0.0, 2000.0);
                        let half = (look.light + view).try_normalize().unwrap_or(n);
                        let highlight = DIFFUSE
                            * ((power + 2.0) / 8.0 * trig::pow(n.dot(half).max(0.0), power))
                                .min(8.0)
                            * towards
                            * (1.0 - r);
                        let around = match look.environment {
                            Some(env) => env.reflected(back, r).map(|c| c * shade),
                            None => {
                                let sharp = ENV_DOWN + (ENV_UP - ENV_DOWN) * (0.5 + 0.5 * back.z);
                                let flat = (ENV_DOWN + ENV_UP) / 2.0;
                                [(sharp + (flat - sharp) * r) * shade; 3]
                            }
                        };
                        (highlight, around)
                    };
                    let (highlight, around) = lobe(rough);
                    let reach = metal + (1.0 - metal) * smooth;
                    // More is reflected at a grazing angle (Schlick).
                    let graze = {
                        let g = 1.0 - facing_view;
                        g * g * g * g * g
                    };
                    // A thin film tints the reflection by the angle: a
                    // film of 400 nm and index 1.3, at red, green and
                    // blue light.
                    let film = [650.0, 510.0, 475.0].map(|wave: f64| {
                        let inside = (1.0 - (1.0 - facing_view * facing_view) / (1.3 * 1.3)).sqrt();
                        0.5 + 0.5 * trig::cos(trig::TAU * 2.0 * 1.3 * 400.0 * inside / wave)
                    });
                    let coat_share = g.coat.0 * (0.04 + 0.96 * graze);
                    for (c, m) in mirror.iter_mut().enumerate() {
                        let mut f0 = 0.04 * (1.0 - metal) + col[c].clamp(0.0, 1.0) * metal;
                        f0 += (film[c] - f0) * g.iridescence * (0.3 + 0.7 * reach);
                        let near = if look.environment.is_some() || extras {
                            // With a picture to reflect, and under a
                            // coat, the grazing gain shows.
                            f0 + (1.0 - f0) * graze * reach
                        } else {
                            f0
                        };
                        *m = near * (highlight + around[c] * reach) * (1.0 - coat_share);
                    }
                    if g.coat.0 > 0.0 {
                        let (highlight, around) = lobe(g.coat.1);
                        for (c, m) in mirror.iter_mut().enumerate() {
                            *m += coat_share * (highlight + around[c]);
                        }
                    }
                    if g.sheen.0 != [0.0; 3] {
                        // Cloth's soft light towards the rim: wider the
                        // rougher the sheen.
                        let rim = trig::pow(1.0 - facing_view, 1.0 + 4.0 * (1.0 - g.sheen.1));
                        for (c, m) in mirror.iter_mut().enumerate() {
                            *m += g.sheen.0[c] * rim * to_linear(lights[c].clamp(0.0, 1.0));
                        }
                    }
                }
                transmitted = g.transmission * (1.0 - metal);
                [0, 1, 2].map(|c| {
                    let matte = to_srgb((col[c] * (1.0 - metal)).clamp(0.0, 1.0));
                    let linear = to_linear((matte * lights[c]).clamp(0.0, 1.0))
                        * (1.0 - transmitted)
                        + mirror[c]
                        + e[c] * tint[c];
                    to_srgb(linear.clamp(0.0, 1.0))
                })
            }
        };
        if transmitted > 0.0 {
            // What passes through: what is behind, tinted by the
            // surface's colour, under the surface's own light. Nearer
            // things still hide what is behind this.
            let a = alpha * col[3].clamp(0.0, 1.0);
            let under = out[i];
            out[i] = [0, 1, 2].map(|c| {
                let behind = to_linear(f64::from(under[c]) / 255.0) * col[c].clamp(0.0, 1.0);
                let own = to_linear(lit[c]);
                let seen = to_srgb((own + transmitted * behind).clamp(0.0, 1.0));
                byte(seen * a + f64::from(under[c]) / 255.0 * (1.0 - a))
            });
            if alpha == 1.0 {
                depth[i] = z;
                ground[i] = false;
            }
            return;
        }
        if alpha == 1.0 {
            depth[i] = z;
            ground[i] = false;
            out[i] = lit.map(byte);
        } else {
            let a = alpha * col[3].clamp(0.0, 1.0);
            let under = out[i];
            out[i] = [0, 1, 2].map(|c| byte(lit[c] * a + f64::from(under[c]) / 255.0 * (1.0 - a)));
        }
    });
}
