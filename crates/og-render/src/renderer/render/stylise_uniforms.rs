//! Stylise uniforms stage.

use super::*;

impl Renderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn write_frame_stylise_uniform(
        &self,
        st: &Stylise,
        scene: &RenderScene,
        (camera, cam): (&Camera, &frame::CameraFrame),
        upscale: u32,
        grid_offset: (u32, u32),
        lens: Option<LensDistortion>,
        prepass: bool,
    ) {
        {
            // The chain's frame: the output's under TAAU (its prepass
            // reads map to the scene's pixels, stylise.wgsl `prepass_px`).
            let (w, h) = self.history_size();
            let (w, h) = (w as f32, h as f32);
            let bg = match scene.environment.background {
                Background::Color(c) => c,
                _ => Vec3::ZERO,
            };
            let q = st.quantize;
            let pattern = match q.map(|q| q.dither) {
                Some(DitherPattern::Bayer2) => 2.0,
                Some(DitherPattern::Bayer4) => 4.0,
                Some(DitherPattern::Bayer8) => 8.0,
                Some(DitherPattern::InterleavedGradient) => -1.0,
                Some(DitherPattern::BlueNoise) => -2.0,
                _ => 0.0,
            };
            let palette_count = q
                .and_then(|q| q.palette)
                .and_then(|p| scene.palettes().get(p.index()))
                .map_or(0.0, |p| p.colors.len() as f32);
            let ht = st.halftone;
            let hc = st.hatching;
            let pt = st.paint;
            let ink = st.ink;
            // Surface space: positions from the surface origin (f64 on the
            // CPU, so a camera far from 0 keeps the patterns' precision),
            // and the canvas uv there. The camera's place in the world is
            // the scene's origin plus its own (`origin.rs`).
            let eye = scene.origin.as_dvec3() + camera.position.as_dvec3();
            let origin = post::surface_origin(eye);
            let surface_eye = (eye - origin).as_vec3();
            let canvas_world_scale = pt.map_or(0.0, |p| {
                if p.canvas_world_scale > 0.0 {
                    p.canvas_world_scale.min(4096.0)
                } else {
                    stylise::DEFAULT_CANVAS_WORLD_SCALE
                }
            });
            let canvas_size = pt
                .and_then(|p| p.canvas)
                .and_then(|t| scene.texture_size(t))
                .map_or(glam::DVec2::ONE, |(w, h)| {
                    glam::DVec2::new(w.max(1) as f64, h.max(1) as f64)
                });
            let canvas_origin = {
                // The uv of the origin along x (u), y (v) and z (u on the
                // x plane, v on the y plane), as stylise.wgsl's
                // `canvas_surface` lays the three planes.
                let k = canvas_world_scale as f64;
                let c = |o: f64, size: f64| ((o * k / size).rem_euclid(1.0)) as f32;
                [
                    c(origin.x, canvas_size.x),
                    c(origin.y, canvas_size.y),
                    c(origin.z, canvas_size.x),
                    c(origin.z, canvas_size.y),
                ]
            };
            let u = post::StyliseUniform {
                screen: [w, h, 1.0 / w, 1.0 / h],
                output: [
                    self.out_width as f32,
                    self.out_height as f32,
                    upscale as f32,
                    0.0,
                ],
                offset: [
                    grid_offset.0 as f32,
                    grid_offset.1 as f32,
                    pt.and_then(|p| p.canvas).is_some() as u32 as f32,
                    palette_count,
                ],
                background: [bg.x, bg.y, bg.z, 1.0],
                quantize: [
                    q.map_or(0.0, |q| q.posterize.min(64) as f32),
                    pattern,
                    q.map_or(0.0, |q| q.dither_strength.clamp(0.0, 2.0)),
                    q.map_or(1.0, |q| q.dither_scale.max(1) as f32),
                ],
                halftone: ht.map_or([0.0; 4], |h| {
                    [
                        h.strength.clamp(0.0, 1.0),
                        h.size.max(2.0),
                        h.angle_degrees.to_radians(),
                        0.0,
                    ]
                }),
                halftone_color: ht.map_or([0.0; 4], |h| [h.color[0], h.color[1], h.color[2], 1.0]),
                hatching: hc.map_or([0.0; 4], |h| {
                    [
                        h.strength.clamp(0.0, 1.0),
                        h.scale.max(2.0),
                        h.levels.clamp(1, 6) as f32,
                        h.angle_degrees.to_radians(),
                    ]
                }),
                hatching_color: hc.map_or([0.0; 4], |h| [h.color[0], h.color[1], h.color[2], 1.0]),
                paint: pt.map_or([0.0; 4], |p| {
                    [
                        p.radius.clamp(0.0, 8.0),
                        p.anisotropy.clamp(0.0, 1.0),
                        p.edge_darkening.clamp(0.0, 1.0),
                        p.bleed.clamp(0.0, 1.0),
                    ]
                }),
                canvas: pt.map_or([0.0; 4], |p| {
                    [
                        p.canvas_strength.clamp(0.0, 1.0),
                        p.canvas_scale.max(0.125),
                        0.0,
                        0.0,
                    ]
                }),
                ink: ink.map_or([0.0; 4], |i| {
                    [
                        1.0,
                        i.threshold.clamp(0.0, 1.0),
                        i.width.max(0.5),
                        i.depth.clamp(0.0, 1.0),
                    ]
                }),
                ink_color: ink.map_or([0.0; 4], |i| [i.color[0], i.color[1], i.color[2], 1.0]),
                ink_depth: {
                    let hatching_depth = hc.map_or(0.0, |h| h.depth.clamp(0.0, 1.0));
                    ink.map_or([camera.far, 0.04, 0.4, hatching_depth], |i| {
                        [
                            camera.far,
                            i.depth_threshold.max(0.0),
                            i.normal_threshold.max(0.0),
                            hatching_depth,
                        ]
                    })
                },
                inv_proj: cam.proj.inverse().to_cols_array_2d(),
                view_rotation: glam::Mat4::from_mat3(glam::Mat3::from_mat4(cam.view.inverse()))
                    .to_cols_array_2d(),
                eye: surface_eye.extend(0.0).to_array(),
                // 0 screen, 1 world, 2 object space (StyliseSpace::code).
                surface: [
                    pt.filter(|p| p.canvas_on_surfaces())
                        .map_or(0.0, |p| p.canvas_space.code()),
                    canvas_world_scale,
                    hc.map_or(0.0, |h| h.space.code()),
                    ht.map_or(0.0, |h| h.space.code()),
                ],
                canvas_origin,
                lens: lens.map_or([0.0, 1.0, 0.5, 0.5], |l| {
                    // The farthest corner's squared radius (1 for a centred
                    // lens), which a barrel's scale keeps on the image.
                    let size = glam::Vec2::new(w, h);
                    let far = [0.0, 1.0]
                        .into_iter()
                        .flat_map(|x| [0.0, 1.0].map(|y| glam::Vec2::new(x, y)))
                        .map(|corner| ((corner - l.center) * size).length_squared())
                        .fold(0.0, f32::max)
                        / (size * 0.5).length_squared();
                    let amount = l.amount.clamp(-1.0, 1.0);
                    let fit = 1.0 / (1.0 + amount.max(0.0) * far);
                    [amount, fit, l.center.x, l.center.y]
                }),
                paint2: pt.filter(|p| p.detail_aware()).map_or([0.0; 4], |p| {
                    [
                        p.detail.clamp(0.0, 1.0),
                        p.character_detail.clamp(0.0, 1.0),
                        prepass as u32 as f32,
                        0.0,
                    ]
                }),
            };
            self.post.write_stylise(&self.queue, &u);
        }
    }
}
