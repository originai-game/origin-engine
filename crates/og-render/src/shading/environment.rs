pub use og_render_data::environment::*;

/// How a frame uses an environment map ([`crate::RenderScene::environment_map`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentMap {
    pub map: crate::EnvironmentMapHandle,
    /// Turns the map about +y, in radians (counter-clockwise seen from
    /// above: what was at +x moves towards -z), as three.js
    /// `scene.environmentRotation.y`.
    pub rotation: f32,
    /// Multiplies the map where it is drawn as the background
    /// ([`crate::Background::Sky`]), as three.js `backgroundIntensity`; the
    /// light's strength is [`crate::Environment::intensity`].
    pub background_intensity: f32,
    /// A second map blended over this one, light and background alike
    /// (indoor to outdoor): `None` is this map alone.
    pub blend: Option<EnvironmentBlend>,
}

/// A blend from [`EnvironmentMap::map`] towards another map.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnvironmentBlend {
    /// The map blended towards (turned and scaled as the first).
    pub to: crate::EnvironmentMapHandle,
    /// 0 is the first map, 1 the second. The caller drives it from the
    /// tick, never from a clock, so a replay blends alike.
    pub weight: f32,
}

/// The steps a blend's weight is drawn at (each step regenerates the
/// image-based light once; finer steps are invisible).
pub const ENVIRONMENT_BLEND_STEPS: f32 = 128.0;

impl EnvironmentBlend {
    /// The weight drawn: within 0..1 (not finite: 0), in
    /// [`ENVIRONMENT_BLEND_STEPS`] steps.
    pub fn drawn_weight(&self) -> f32 {
        let w = if self.weight.is_finite() {
            self.weight.clamp(0.0, 1.0)
        } else {
            0.0
        };
        (w * ENVIRONMENT_BLEND_STEPS).round() / ENVIRONMENT_BLEND_STEPS
    }
}

/// `a`'s harmonics moved `w` of the way to `b`'s: the irradiance of the
/// blend of two radiances is the blend of their irradiances.
pub(crate) fn mix_irradiance(a: &[[f32; 3]; 9], b: &[[f32; 3]; 9], w: f32) -> [[f32; 3]; 9] {
    std::array::from_fn(|k| std::array::from_fn(|c| a[k][c] + (b[k][c] - a[k][c]) * w))
}

impl EnvironmentMap {
    /// `map` unturned, its background as cooked.
    pub fn new(map: crate::EnvironmentMapHandle) -> EnvironmentMap {
        EnvironmentMap {
            map,
            rotation: 0.0,
            background_intensity: 1.0,
            blend: None,
        }
    }
}

/// The IBL passes' uniform block for a map: (cos, sin, scale, the second
/// map's weight in a blend) then the 27 irradiance floats.
pub(crate) fn env_params(
    rotation: f32,
    scale: f32,
    blend: f32,
    irradiance: &[[f32; 3]; 9],
) -> [[f32; 4]; 8] {
    let mut out = [[0.0; 4]; 8];
    out[0] = [rotation.cos(), rotation.sin(), scale, blend];
    for (k, v) in irradiance.iter().flatten().enumerate() {
        out[1 + k / 4][k % 4] = *v;
    }
    out
}

/// Uploads a map as a cube texture of [`ENVIRONMENT_MAP_LEVELS`] levels.
pub(crate) fn upload(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    map: &EnvironmentMapData,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("environment map"),
        size: wgpu::Extent3d {
            width: map.size,
            height: map.size,
            depth_or_array_layers: 6,
        },
        mip_level_count: ENVIRONMENT_MAP_LEVELS,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgb9e5Ufloat,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut at = 0;
    for level in 0..ENVIRONMENT_MAP_LEVELS {
        let n = map.size >> level;
        let count = 6 * (n * n) as usize;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&map.texels[at..at + count]),
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * n),
                rows_per_image: Some(n),
            },
            wgpu::Extent3d {
                width: n,
                height: n,
                depth_or_array_layers: 6,
            },
        );
        at += count;
    }
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some("environment map"),
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    });
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb9e5_round_trips_within_its_precision() {
        for rgb in [
            [0.0, 0.0, 0.0],
            [1.0, 0.5, 0.25],
            [100.0, 3.0, 0.01],
            [65408.0, 1.0, 0.0],
            [0.18, 0.18, 0.18],
            [1e-6, 2e-6, 0.0],
        ] {
            let back = rgb9e5_decode(rgb9e5(rgb));
            let max = rgb.iter().cloned().fold(0.0f32, f32::max);
            for (a, b) in rgb.iter().zip(back) {
                // Below 2^-15 the step is the smallest exponent's, 2^-24.
                assert!((a - b).abs() <= max / 256.0 + 6e-8, "{rgb:?} -> {back:?}");
            }
        }
        assert_eq!(
            rgb9e5_decode(rgb9e5([1e9, -1.0, f32::NAN])),
            [65408.0, 0.0, 0.0]
        );
        // Exactly representable values survive.
        assert_eq!(rgb9e5_decode(rgb9e5([1.0, 0.5, 2.0])), [1.0, 0.5, 2.0]);
    }

    /// The equirect's own irradiance agrees with the cook's bake, which
    /// projects the cube it resampled the image into (a sky with a sun, a
    /// ground and a lit wall: every band carries something).
    #[cfg(feature = "bake")]
    #[test]
    fn equirect_irradiance_matches_the_bake() {
        let (w, h) = (128u32, 64u32);
        let mut rgb = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let c = if y < h / 2 {
                    if x > w / 2 && x < w / 2 + 12 && y > 8 && y < 18 {
                        [30.0, 25.0, 20.0]
                    } else {
                        [0.3, 0.5, 1.0]
                    }
                } else if x < w / 4 {
                    [2.0, 0.4, 0.1]
                } else {
                    [0.2, 0.2, 0.15]
                };
                rgb.extend_from_slice(&c);
            }
        }
        let baked = bake::bake(
            &bake::Equirect {
                width: w,
                height: h,
                rgb: &rgb,
            },
            &bake::BakeDesc {
                size: 64,
                samples: 16,
                ..Default::default()
            },
        )
        .unwrap();
        let mut acc = EquirectIrradiance::new(w, h);
        for y in 0..h {
            acc.add_row(y, &rgb[(y * w * 3) as usize..((y + 1) * w * 3) as usize]);
        }
        let ours = acc.finish();
        let scale = baked.irradiance[0].iter().cloned().fold(0.0f32, f32::max);
        for (k, (o, b)) in ours.iter().zip(&baked.irradiance).enumerate() {
            for (c, (&a, &b)) in o.iter().zip(b).enumerate() {
                assert!(
                    (a - b).abs() < 0.03 * scale,
                    "coefficient {k} channel {c}: {a} against the bake's {b}"
                );
            }
        }
    }

    /// A sky over a ground with a sun of `px`² pixels at radiance `sun`:
    /// the image, the direction towards the sun's centre, and its light
    /// (radiance times solid angle).
    #[cfg(feature = "bake")]
    fn sunny(w: u32, h: u32, px: u32, sun: [f32; 3]) -> (Vec<f32>, glam::Vec3, glam::Vec3) {
        use std::f64::consts::PI;
        let mut rgb = Vec::with_capacity((w * h * 3) as usize);
        for y in 0..h {
            let v = if y < h / 2 { 1.0 } else { 0.2 };
            rgb.extend(std::iter::repeat_n(v, (w * 3) as usize));
        }
        let (x0, y0) = (w / 2 + 37, h / 4 + 11);
        let lat = |y: f64| (0.5 - y / h as f64) * PI;
        let mut solid = 0.0;
        for y in y0..y0 + px {
            for x in x0..x0 + px {
                let i = ((y * w + x) * 3) as usize;
                rgb[i..i + 3].copy_from_slice(&sun);
            }
            solid +=
                px as f64 * (2.0 * PI / w as f64) * (PI / h as f64) * lat(y as f64 + 0.5).cos();
        }
        let lon = ((x0 as f64 + px as f64 / 2.0) / w as f64 - 0.5) * 2.0 * PI;
        let lat = lat(y0 as f64 + px as f64 / 2.0);
        let towards = glam::DVec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        (
            rgb,
            towards.as_vec3(),
            glam::Vec3::from_array(sun) * solid as f32,
        )
    }

    /// A sun of a few pixels in a large image gives the map the same light
    /// wherever it falls among the cube's texels: the bake samples closer
    /// together than the image's pixels. (Four samples a texel gave 0.85
    /// to 1.29 times its light as the map turned by tenths of a degree.)
    #[cfg(feature = "bake")]
    #[test]
    fn a_small_sun_keeps_its_light_as_the_map_turns() {
        let (w, h) = (2048, 1024);
        let (rgb, _, light) = sunny(w, h, 3, [50000.0; 3]);
        let (sky, _, _) = sunny(w, h, 0, [0.0; 3]);
        let total = |rgb: &[f32], turn: f32| {
            let baked = bake::bake(
                &bake::Equirect {
                    width: w,
                    height: h,
                    rgb,
                },
                &bake::BakeDesc {
                    size: 128,
                    samples: 16,
                    rotation_degrees: turn,
                    ..Default::default()
                },
            )
            .unwrap();
            // The first harmonic is the whole sphere's radiance times Y0.
            baked.irradiance[0][0] / 0.282095
        };
        let without = total(&sky, 0.0);
        for k in 0..8 {
            let turn = k as f32 * 0.11;
            let got = (total(&rgb, turn) - without) / light.x;
            assert!(
                (got - 1.0).abs() < 0.02,
                "turned {turn}°: {got} of the sun's light"
            );
        }
    }

    /// `sun` takes the sun out of the map as a light: its direction, its
    /// colour and the irradiance it gave (radiance times solid angle), in
    /// the turned map's frame; what stays is no brighter than sixteen times
    /// the rest of the image, and an image without a sun is left whole.
    #[cfg(feature = "bake")]
    #[test]
    fn the_sun_leaves_the_map_as_a_light() {
        let (w, h) = (2048, 1024);
        let bake = |rgb: &[f32], turn: f32, sun: bool| {
            bake::bake(
                &bake::Equirect {
                    width: w,
                    height: h,
                    rgb,
                },
                &bake::BakeDesc {
                    size: 128,
                    samples: 16,
                    rotation_degrees: turn,
                    sun,
                    ..Default::default()
                },
            )
            .unwrap()
        };
        let (rgb, towards, light) = sunny(w, h, 3, [40000.0, 36000.0, 28000.0]);
        for turn in [0.0f32, 90.0] {
            let map = bake(&rgb, turn, true);
            // Turned about +y: what was at +x moves towards -z.
            let (s, c) = turn.to_radians().sin_cos();
            let want = glam::Vec3::new(
                c * towards.x + s * towards.z,
                towards.y,
                c * towards.z - s * towards.x,
            );
            let got = glam::Vec3::from_array(map.sun.direction);
            assert!(
                got.dot(want) > 0.2f32.to_radians().cos(),
                "turned {turn}°: the sun towards {got}, not {want}"
            );
            let lit = glam::Vec3::from_array(map.sun.color) * map.sun.intensity;
            assert!(
                ((lit - light) / light).abs().max_element() < 0.02,
                "turned {turn}°: the sun's light {lit}, not {light}"
            );
            assert_eq!(map.sun.color[0], 1.0);
            // The rest's mean luminance is 0.6: at most 9.6 stays.
            let count = (6 * 128 * 128) as usize;
            let brightest = map.texels[..count]
                .iter()
                .map(|&t| {
                    glam::Vec3::from_array(rgb9e5_decode(t))
                        .dot(glam::Vec3::new(0.2126, 0.7152, 0.0722))
                })
                .fold(0.0f32, f32::max);
            assert!((9.0..10.0).contains(&brightest), "{brightest} stays");
            // The light that left is the light the sun holds.
            let whole = bake(&rgb, turn, false);
            let left = (whole.irradiance[0][1] - map.irradiance[0][1]) / 0.282095;
            assert!((left / lit.y - 1.0).abs() < 1e-3, "{left} left, {lit}");
        }
        let (sky, _, _) = sunny(w, h, 0, [0.0; 3]);
        let none = bake(&sky, 0.0, true);
        assert_eq!(none.sun, EnvironmentSun::default());
        assert_eq!(none, bake(&sky, 0.0, false));
    }

    /// A blend's weight is clamped and drawn in 128 steps, so a weight
    /// moved a little each tick regenerates the light only when it crosses
    /// a step, and a broken weight draws the first map.
    #[test]
    fn a_blend_weight_is_clamped_and_stepped() {
        let w = |weight: f32| {
            EnvironmentBlend {
                to: crate::EnvironmentMapHandle::from_index(0),
                weight,
            }
            .drawn_weight()
        };
        assert_eq!(w(-0.3), 0.0);
        assert_eq!(w(1.7), 1.0);
        assert_eq!(w(f32::NAN), 0.0);
        assert_eq!(w(f32::INFINITY), 0.0);
        assert_eq!(w(0.5), 0.5);
        // Within half a step: the same drawn weight (no regeneration).
        assert_eq!(w(0.3), w(0.3 + 0.05 / ENVIRONMENT_BLEND_STEPS));
        assert_eq!(w(0.3), 38.0 / 128.0);
        assert_ne!(w(0.3), w(0.3 + 1.0 / ENVIRONMENT_BLEND_STEPS));
    }

    /// The blended harmonics are each coefficient's mix: the ends are the
    /// two maps', and the irradiance at any normal is the mix of theirs.
    #[test]
    fn blended_irradiance_mixes_each_coefficient() {
        let a: [[f32; 3]; 9] = std::array::from_fn(|k| [k as f32, 1.0, -2.0]);
        let b: [[f32; 3]; 9] = std::array::from_fn(|k| [0.5, 3.0 * k as f32, 4.0]);
        assert_eq!(mix_irradiance(&a, &b, 0.0), a);
        assert_eq!(mix_irradiance(&a, &b, 1.0), b);
        let m = mix_irradiance(&a, &b, 0.25);
        assert_eq!(
            m[4],
            [
                4.0 * 0.75 + 0.5 * 0.25,
                0.75 + 12.0 * 0.25,
                -2.0 * 0.75 + 4.0 * 0.25
            ]
        );
        let n = glam::Vec3::new(0.3, -0.8, 0.5).normalize();
        let at = |h: &[[f32; 3]; 9]| -> f32 { sh9(n).iter().zip(h).map(|(y, c)| y * c[1]).sum() };
        assert!((at(&m) - (0.75 * at(&a) + 0.25 * at(&b))).abs() < 1e-4);
    }

    /// The blend's weight rides in the IBL passes' first vector beside the
    /// turn and the scale.
    #[test]
    fn env_params_carry_the_blend_weight() {
        let p = env_params(0.0, 2.0, 0.375, &[[0.0; 3]; 9]);
        assert_eq!(p[0], [1.0, 0.0, 2.0, 0.375]);
    }

    #[test]
    fn face_directions_follow_the_cube_convention() {
        use glam::Vec3;
        assert_eq!(face_direction(0, 0.0, 0.0), Vec3::X);
        assert_eq!(face_direction(1, 0.0, 0.0), -Vec3::X);
        assert_eq!(face_direction(2, 0.0, 0.0), Vec3::Y);
        assert_eq!(face_direction(3, 0.0, 0.0), -Vec3::Y);
        assert_eq!(face_direction(4, 0.0, 0.0), Vec3::Z);
        assert_eq!(face_direction(5, 0.0, 0.0), -Vec3::Z);
        // +x face: right is -z, up is +y.
        assert!(face_direction(0, 1.0, 0.0).z < 0.0);
        assert!(face_direction(0, 0.0, 1.0).y > 0.0);
    }

    /// A cube's own prefiltered chain is taken, not filtered again: five
    /// levels from the map's size are copied texel for texel (the bake's
    /// own levels read back come out as they went in), and a shorter chain
    /// is read by roughness, level m at lod m / 4 * (levels - 1).
    #[cfg(feature = "bake")]
    #[test]
    fn a_prefiltered_chain_is_taken_as_is() {
        use super::bake::{
            BakeDesc, Equirect, PrefilteredCube, bake, from_prefiltered, irradiance,
        };
        let (w, h) = (32u32, 16u32);
        let rgb: Vec<f32> = (0..w * h)
            .flat_map(|i| {
                let (x, y) = ((i % w) as f32 / w as f32, (i / w) as f32 / h as f32);
                [0.2 + 2.0 * x, 1.5 * (1.0 - y), 0.3 + x * y]
            })
            .collect();
        let desc = BakeDesc {
            size: 16,
            samples: 32,
            ..Default::default()
        };
        let baked = bake(
            &Equirect {
                width: w,
                height: h,
                rgb: &rgb,
            },
            &desc,
        )
        .unwrap();
        let mut levels = Vec::new();
        let mut at = 0;
        for m in 0..ENVIRONMENT_MAP_LEVELS {
            let count = 6 * ((16u32 >> m) as usize).pow(2);
            levels.push(
                baked.texels[at..at + count]
                    .iter()
                    .flat_map(|t| rgb9e5_decode(*t))
                    .collect::<Vec<f32>>(),
            );
            at += count;
        }
        let src = PrefilteredCube {
            size: 16,
            levels: &levels,
        };
        let taken = from_prefiltered(&src, &desc).unwrap();
        assert_eq!(taken.texels, baked.texels, "five levels are copied");
        for n in [glam::Vec3::Y, glam::Vec3::X, -glam::Vec3::Z] {
            let (a, b) = (irradiance(&taken, n), irradiance(&baked, n));
            assert!(
                (a - b).abs().max_element() < 0.01 * b.max_element(),
                "{a} {b}"
            );
        }
        // Three levels, each one value: level m reads lod m / 2.
        let flat: Vec<Vec<f32>> = (0..3u32)
            .map(|k| vec![k as f32 + 1.0; 6 * ((16u32 >> k) as usize).pow(2) * 3])
            .collect();
        let src = PrefilteredCube {
            size: 16,
            levels: &flat,
        };
        let turned = BakeDesc {
            rotation_degrees: 30.0,
            intensity: 2.0,
            ..desc
        };
        let map = from_prefiltered(&src, &turned).unwrap();
        let mut at = 0;
        for m in 0..ENVIRONMENT_MAP_LEVELS {
            let count = 6 * ((16u32 >> m) as usize).pow(2);
            let want = 2.0 * (1.0 + m as f32 / 2.0);
            for t in &map.texels[at..at + count] {
                for c in rgb9e5_decode(*t) {
                    assert!((c - want).abs() < want / 200.0, "level {m}: {c} not {want}");
                }
            }
            at += count;
        }
        // One level is not a chain; another size is not this chain's.
        assert!(
            from_prefiltered(
                &PrefilteredCube {
                    size: 16,
                    levels: &flat[..1]
                },
                &desc
            )
            .is_err()
        );
        let other = BakeDesc { size: 32, ..desc };
        assert!(from_prefiltered(&src, &other).unwrap_err().contains("16²"));
    }

    #[cfg(feature = "bake")]
    #[test]
    fn a_uniform_sky_bakes_to_itself() {
        use super::bake::{BakeDesc, Equirect, bake, irradiance};
        let rgb = vec![0.5f32; 16 * 8 * 3];
        let map = bake(
            &Equirect {
                width: 16,
                height: 8,
                rgb: &rgb,
            },
            &BakeDesc {
                size: 16,
                samples: 64,
                ..Default::default()
            },
        )
        .expect("bake");
        assert_eq!(map.texels.len(), EnvironmentMapData::texel_count(16));
        for t in &map.texels {
            for c in rgb9e5_decode(*t) {
                assert!((c - 0.5).abs() < 0.01, "{c}");
            }
        }
        // A white surface under a uniform radiance of 0.5 reflects 0.5.
        for n in [glam::Vec3::Y, glam::Vec3::X, -glam::Vec3::Z] {
            let e = irradiance(&map, n);
            assert!(
                (e - glam::Vec3::splat(0.5)).abs().max_element() < 0.01,
                "{e}"
            );
        }
    }

    #[cfg(feature = "bake")]
    #[test]
    fn the_sky_above_lights_upward_faces_and_rotation_turns_the_map() {
        use super::bake::{BakeDesc, Equirect, bake, irradiance};
        // Bright upper half, dark lower half; a red patch at u = 0.5 (+x).
        let (w, h) = (64u32, 32u32);
        let mut rgb = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let up = y < h / 2;
                let red = (28..36).contains(&x) && (12..20).contains(&y);
                rgb.extend(if red {
                    [8.0, 0.0, 0.0]
                } else if up {
                    [1.0, 1.0, 1.0]
                } else {
                    [0.1, 0.1, 0.1]
                });
            }
        }
        let src = Equirect {
            width: w,
            height: h,
            rgb: &rgb,
        };
        let desc = BakeDesc {
            size: 32,
            samples: 64,
            ..Default::default()
        };
        let map = bake(&src, &desc).expect("bake");
        let up = irradiance(&map, glam::Vec3::Y);
        let down = irradiance(&map, -glam::Vec3::Y);
        assert!(up.y > 4.0 * down.y, "{up} {down}");
        let red = |m: &EnvironmentMapData, face: u32| {
            // The centre texel of `face` at level 0.
            let n = m.size as usize;
            rgb9e5_decode(m.texels[face as usize * n * n + (n / 2) * n + n / 2])[0]
        };
        assert!(red(&map, 0) > 4.0, "the patch faces +x: {}", red(&map, 0));
        // Turned 90 degrees, +x moves towards -z.
        let turned = bake(
            &src,
            &BakeDesc {
                rotation_degrees: 90.0,
                ..desc
            },
        )
        .expect("bake");
        assert!(red(&turned, 5) > 4.0, "turned to -z: {}", red(&turned, 5));
        assert!(red(&turned, 0) < 1.5);
    }

    /// A smooth sky: brighter above, warm, a hot lobe towards +x.
    #[cfg(feature = "bake")]
    fn sky(d: glam::Vec3) -> [f32; 3] {
        let lobe = 4.0 * d.x.max(0.0).powi(8);
        let up = 1.0 + d.y;
        [up + lobe, 0.8 * up, 0.6 * up]
    }

    /// The same sky given as cube faces bakes as it does given as an
    /// equirectangular image: within 2 % in irradiance and a few percent
    /// per texel of every level; a source cube larger than twice the bake
    /// is box-filtered first, so 256² faces bake like 64² ones.
    #[cfg(feature = "bake")]
    #[test]
    fn a_cube_bakes_as_its_equirectangular_twin() {
        use super::bake::{BakeDesc, CubeFaces, Equirect, bake, bake_cube, irradiance};
        use std::f32::consts::PI;
        let (w, h) = (256u32, 128u32);
        let mut equirect = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let lon = ((x as f32 + 0.5) / w as f32 - 0.5) * 2.0 * PI;
                let lat = (0.5 - (y as f32 + 0.5) / h as f32) * PI;
                let d = glam::Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
                equirect.extend(sky(d));
            }
        }
        let cube = |n: u32| {
            let mut rgb = Vec::new();
            for face in 0..6 {
                for j in 0..n {
                    for i in 0..n {
                        let x = 2.0 * (i as f32 + 0.5) / n as f32 - 1.0;
                        let y = 1.0 - 2.0 * (j as f32 + 0.5) / n as f32;
                        rgb.extend(sky(face_direction(face, x, y)));
                    }
                }
            }
            rgb
        };
        let desc = BakeDesc {
            size: 32,
            samples: 64,
            rotation_degrees: 30.0,
            intensity: 1.5,
            sun: false,
        };
        let a = bake(
            &Equirect {
                width: w,
                height: h,
                rgb: &equirect,
            },
            &desc,
        )
        .unwrap();
        let faces64 = cube(64);
        let b = bake_cube(
            &CubeFaces {
                size: 64,
                rgb: &faces64,
            },
            &desc,
        )
        .unwrap();
        for n in [
            glam::Vec3::Y,
            -glam::Vec3::Y,
            glam::Vec3::X,
            glam::Vec3::Z,
            -glam::Vec3::X,
        ] {
            let (ea, eb) = (irradiance(&a, n), irradiance(&b, n));
            assert!(
                ((ea - eb).abs() / ea.max(glam::Vec3::splat(0.05))).max_element() < 0.02,
                "{n}: {ea} {eb}"
            );
        }
        let diff = |x: &EnvironmentMapData, y: &EnvironmentMapData| {
            let mut worst = 0f32;
            let mut sum = 0f32;
            for (p, q) in x.texels.iter().zip(&y.texels) {
                let (p, q) = (rgb9e5_decode(*p), rgb9e5_decode(*q));
                for c in 0..3 {
                    let d = (p[c] - q[c]).abs() / p[c].max(q[c]).max(0.1);
                    worst = worst.max(d);
                    sum += d;
                }
            }
            (sum / (3 * x.texels.len()) as f32, worst)
        };
        let (mean, worst) = diff(&a, &b);
        assert!(mean < 0.01 && worst < 0.1, "mean {mean} worst {worst}");
        // A 256² source is filtered down to the 64² level the bake reads.
        let faces256 = cube(256);
        let c = bake_cube(
            &CubeFaces {
                size: 256,
                rgb: &faces256,
            },
            &desc,
        )
        .unwrap();
        let (mean, worst) = diff(&b, &c);
        assert!(mean < 0.005 && worst < 0.05, "mean {mean} worst {worst}");
        // Faces of the wrong size are refused.
        assert!(
            bake_cube(
                &CubeFaces {
                    size: 63,
                    rgb: &faces64
                },
                &desc
            )
            .is_err()
        );
    }
}
