//! The frame's sprite and strip arenas: the strips (trails, ribbons,
//! beams), the light map's slots, the 2D fluids, and the sprites, 2D
//! particles and text of the main camera, each view and each due probe
//! face, appended in that order to the frame's strip and sprite buffers,
//! so the passes that draw them read their ranges.

use super::*;

pub(super) struct SpriteArenaInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub aspect: f32,
    pub view_frustum: &'a Frustum,
    pub view_plans: &'a [views::ViewPlan],
    pub particle_frame: &'a particles_gpu::ParticleFrame,
    pub probe_frame: &'a probes::ProbeFrame,
    pub probe_faces: &'a [[frame::CameraFrame; 6]],
    pub active_probes: &'a [ReflectionProbe],
}

pub(super) struct SpriteArenas {
    pub ribbon_frame: ribbons::RibbonFrame,
    pub light_slots: light2d::LightSlots,
    pub lit_range: Option<(i32, i32)>,
    pub view_ribbons: Vec<Vec<ribbons::RibbonRun>>,
    pub fluids: usize,
    pub gpu_sprites: sprites::GpuSprites,
    pub sprite_frame: sprites::SpriteFrame,
    pub view_sprites: Vec<Vec<sprites::SpriteRun>>,
    pub view_glow_runs: Vec<Vec<sprites::SpriteRun>>,
    pub view_boards: Vec<sprites::Billboards>,
    pub face_sprites: Vec<Vec<sprites::SpriteRun>>,
    pub face_boards: Vec<sprites::Billboards>,
}

impl Renderer {
    pub(super) fn build_sprite_arenas(
        &mut self,
        SpriteArenaInputs {
            scene,
            camera,
            cam,
            aspect,
            view_frustum,
            view_plans,
            particle_frame,
            probe_frame,
            probe_faces,
            active_probes,
        }: SpriteArenaInputs<'_>,
    ) -> Result<SpriteArenas, RenderError> {
        let mut ribbon_frame = match &mut self.strips.gpu_trails {
            Some(g) => g.build(
                &self.device,
                &self.queue,
                scene.id(),
                scene.trail_styles(),
                &scene.effects.trails,
                &scene.effects.trail_layers,
                &scene.effects.ribbons,
                &scene.effects.beams,
                scene.tick,
                scene.tick_rate,
            ),
            None => self.strips.trails.build(
                scene.id(),
                scene.trail_styles(),
                &scene.effects.trails,
                &scene.effects.trail_layers,
                &scene.effects.ribbons,
                &scene.effects.beams,
                scene.tick,
                scene.tick_rate,
                camera.position,
                cam.forward,
                cam.orthographic,
            ),
        }
        .map_err(RenderError::InvalidHandle)?;
        for run in &ribbon_frame.runs {
            if let Some(t) = run.texture
                && t.index() >= self.cache.textures.len()
            {
                return Err(RenderError::InvalidHandle(format!(
                    "strip texture {}",
                    t.index()
                )));
            }
            self.ribbon_group(run.texture);
        }
        let lit_range = scene.lighting_2d.map(|l| (l.min_layer, l.max_layer));
        // The light map's strips, one per light mask of the lit receivers.
        let light_slots = light2d::LightSlots::of(scene);
        let lit = |layer: i32, mask: u32| light_slots.lit(lit_range, layer, mask);
        let view = match camera.projection {
            Projection::Orthographic { height } => {
                let half = Vec2::new(height * 0.5 * aspect, height * 0.5);
                let c = camera.position.truncate();
                Some(sprites::ViewRect {
                    min: c - half,
                    max: c + half,
                })
            }
            Projection::Perspective { .. } => None,
        };
        water::water_2d_strips(&scene.water_2d, &mut ribbon_frame);
        // Each view's strips facing its own eye (views.rs): the same strip
        // points and 2D water (the vertex shader faces the view's camera),
        // or, built on the CPU, its own after the main camera's in the
        // strip buffer.
        let gpu_strips = self.strips.gpu_trails.is_some();
        let view_ribbons: Vec<Vec<ribbons::RibbonRun>> = view_plans
            .iter()
            .map(|p| {
                if gpu_strips {
                    return ribbon_frame.runs.clone();
                }
                let mut f =
                    self.strips
                        .trails
                        .emit(p.position, p.frame.forward, p.frame.orthographic);
                water::water_2d_strips(&scene.water_2d, &mut f);
                let base = ribbon_frame.vertices.len() as u32;
                ribbon_frame.vertices.extend_from_slice(&f.vertices);
                f.runs
                    .into_iter()
                    .map(|mut r| {
                        r.first += base;
                        r
                    })
                    .collect()
            })
            .collect();
        let mut layered: Vec<sprites::LayeredStrip> = ribbon_frame
            .runs
            .iter()
            .filter_map(|r| {
                Some(sprites::LayeredStrip {
                    layer: r.layer?,
                    source: RunSource::Strip(r.blend as u8, r.texture, r.points),
                    first: r.first,
                    count: r.count,
                })
            })
            .collect();
        let fluids = self.world.metaballs.prepare(
            &self.device,
            &self.queue,
            &scene.metaballs,
            (self.width, self.height),
        );
        layered.extend(
            metaballs::layers(&scene.metaballs)
                .into_iter()
                .map(|(i, layer)| sprites::LayeredStrip {
                    layer,
                    source: RunSource::Metaballs(i as u8),
                    first: 0,
                    count: 1,
                }),
        );
        let fonts = &self.cache.fonts;
        let arrays = &self.cache.texture_arrays;
        let font = |i: usize| fonts.get(i).map(|f| &f.layout);
        let array_layers = |a: TextureArrayHandle| {
            scene
                .contains_texture_array(a)
                .then(|| arrays.get(a.index()).and_then(Option::as_ref))
                .flatten()
                .map(|g| g.info.layers)
        };
        // The ranges the builds below take for the 2D particles simulated
        // in compute (`expand_sprites` writes them in the particles pass).
        let mut gpu_sprites = sprites::GpuSprites {
            tiles: self.sprite.tiles_gpu_supported.then(Vec::new),
            ..Default::default()
        };
        let mut sprite_frame = sprites::build(
            scene,
            font,
            self.cache.textures.len(),
            array_layers,
            &particle_frame.sprites,
            (&particle_frame.sprite_systems, &mut gpu_sprites),
            lit,
            view,
            &layered,
            scene.camera_layers,
            &sprites::SpriteEye {
                position: camera.position,
                forward: Some(cam.forward),
                frustum: Some(view_frustum),
            },
        )?;
        // Each view's world sprites, 2D particles and text on its layers
        // (views.rs), their instances after the main camera's in the
        // sprite buffer, lit by the view's own 2D light map, its 2D fluids
        // composited from its own fields, its glowing sprites into its own
        // glow buffer.
        let mut view_sprites: Vec<Vec<sprites::SpriteRun>> = Vec::new();
        let mut view_glow_runs: Vec<Vec<sprites::SpriteRun>> = Vec::new();
        // Each view's billboards among its transparent meshes and their
        // drop shadows (sprites.rs `SpriteFrame::billboards`).
        let mut view_boards: Vec<sprites::Billboards> = Vec::new();
        for (p, runs) in view_plans.iter().zip(&view_ribbons) {
            let view_strips: Vec<sprites::LayeredStrip> = runs
                .iter()
                .filter_map(|r| {
                    Some(sprites::LayeredStrip {
                        layer: r.layer?,
                        source: RunSource::Strip(r.blend as u8, r.texture, r.points),
                        first: r.first,
                        count: r.count,
                    })
                })
                .chain(
                    layered
                        .iter()
                        .filter(|s| matches!(s.source, RunSource::Metaballs(_)))
                        .map(|s| sprites::LayeredStrip {
                            layer: s.layer,
                            source: s.source,
                            first: s.first,
                            count: s.count,
                        }),
                )
                .collect();
            let f = sprites::build(
                scene,
                font,
                self.cache.textures.len(),
                array_layers,
                &particle_frame.sprites,
                (&particle_frame.sprite_systems, &mut gpu_sprites),
                lit,
                None,
                &view_strips,
                p.layers,
                &sprites::SpriteEye {
                    position: p.position,
                    forward: Some(p.frame.forward),
                    frustum: Some(&Frustum::from_view_proj(p.frame.view_proj)),
                },
            )?;
            let in_buffer = |r: &sprites::SpriteRun| {
                !matches!(
                    r.source,
                    RunSource::Strip(..)
                        | RunSource::Mesh(_)
                        | RunSource::Tiles(_)
                        | RunSource::Metaballs(_)
                        | RunSource::Particles(_)
                )
            };
            // Its glow instances follow every run's (its glow pass reads them).
            let base = sprite_frame.instances.len() as u32;
            sprite_frame.instances.extend_from_slice(&f.instances);
            view_boards.push(sprites::billboards_at(&f, base));
            view_glow_runs.push(
                f.glow_runs
                    .into_iter()
                    .map(|mut r| {
                        r.first += base;
                        r
                    })
                    .collect(),
            );
            view_sprites.push(
                f.runs
                    .into_iter()
                    .map(|mut r| {
                        if in_buffer(&r) {
                            r.first += base;
                        }
                        r
                    })
                    .collect(),
            );
        }
        // The due probe faces' world sprites, 2D particles and text (every
        // layer, unlit, without glow: probes.rs), after the views' in the
        // sprite buffer; built for each face, its billboards sorted by the
        // face camera's depth from the capture point among the face's
        // transparent meshes and culled to the face.
        let mut face_sprites: Vec<Vec<sprites::SpriteRun>> = Vec::new();
        let mut face_boards: Vec<sprites::Billboards> = Vec::new();
        for (d, &(_, i)) in probe_frame.due.iter().enumerate() {
            for cam in &probe_faces[d] {
                let frustum = Frustum::from_view_proj(cam.view_proj);
                let f = sprites::build(
                    scene,
                    font,
                    self.cache.textures.len(),
                    array_layers,
                    &particle_frame.sprites,
                    (&particle_frame.sprite_systems, &mut gpu_sprites),
                    |_, _| 0.0,
                    None,
                    &[],
                    u32::MAX,
                    &sprites::SpriteEye {
                        position: active_probes[i].position,
                        forward: Some(cam.forward),
                        frustum: Some(&frustum),
                    },
                )?;
                let base = sprite_frame.instances.len() as u32;
                sprite_frame.instances.extend_from_slice(&f.instances);
                face_boards.push(sprites::billboards_at(&f, base));
                face_sprites.push(
                    f.runs
                        .into_iter()
                        .map(|mut r| {
                            if !matches!(
                                r.source,
                                RunSource::Strip(..)
                                    | RunSource::Mesh(_)
                                    | RunSource::Tiles(_)
                                    | RunSource::Metaballs(_)
                                    | RunSource::Particles(_)
                            ) {
                                r.first += base;
                            }
                            r
                        })
                        .collect(),
                );
            }
        }
        Ok(SpriteArenas {
            ribbon_frame,
            light_slots,
            lit_range,
            view_ribbons,
            fluids,
            gpu_sprites,
            sprite_frame,
            view_sprites,
            view_glow_runs,
            view_boards,
            face_sprites,
            face_boards,
        })
    }
}
