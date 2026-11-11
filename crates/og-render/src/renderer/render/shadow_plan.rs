//! The shadow passes' plan: the sun's cascades and their static cache
//! plans, the weather's shelter map, and the shadowed local lights' faces,
//! each with its casters' lists, registered with the visibility stage in
//! that order (cascades, shelter, local faces) and culled on the CPU where
//! the GPU cull does not take them; then the cache textures the plans
//! copy through, made at their layer's side on first need.

use super::*;

pub(super) struct PlanShadowsInputs<'a> {
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub batches: &'a [Batch],
    pub light: &'a crate::DirectionalLight,
    pub has_casters: bool,
    pub sphere_box: Option<(Vec3, Vec3)>,
    pub to_light: Vec3,
    pub shadow_size: u32,
    pub aspect: f32,
    pub cull_n: u32,
    pub meshlet_casters: &'a [u32],
    pub shadow_split: &'a shadow_cache::Split,
    pub skinned_batches: &'a [Batch],
    pub scatter_batches: &'a scatter::ScatterBatches,
    pub grass_frame: &'a grass::GrassFrame,
    pub terrain_casters: bool,
    pub terrain_draws: &'a [terrain::TerrainDraw],
    pub gpu_cull: bool,
    pub vis: &'a mut FrameVisibility,
    pub spheres: &'a [(Vec3, f32)],
    pub transforms: &'a mut Vec<instances::GpuTransform>,
    pub origins: &'a mut Vec<u32>,
    pub cull_inside: CullScratch,
    pub shelter: &'a Option<(Mat4, Frustum, u64, bool)>,
    pub local_shadows: &'a [frame::LocalShadow],
}

pub(super) struct PlanShadowsOutputs {
    pub cascades: Vec<Cascade>,
    pub cascade_views: Vec<u32>,
    pub local_views: [u32; LOCAL_SHADOW_LAYERS],
    pub cascade_plans: Vec<crate::shadow_cache::Plan>,
    pub cascade_lists: Vec<[DrawList; 2]>,
    pub shelter_view: u32,
    pub shelter_list: Option<[DrawList; 2]>,
    pub local_lists: Vec<Option<[DrawList; 2]>>,
    pub local_frustums: Vec<Option<Frustum>>,
    pub local_plans: [crate::shadow_cache::Plan; LOCAL_SHADOW_LAYERS],
}

impl Renderer {
    /// Plans the frame's shadow passes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_shadows(
        &mut self,
        PlanShadowsInputs {
            scene,
            camera,
            cam,
            batches,
            light,
            has_casters,
            sphere_box,
            to_light,
            shadow_size,
            aspect,
            cull_n,
            meshlet_casters,
            shadow_split,
            skinned_batches,
            scatter_batches,
            grass_frame,
            terrain_casters,
            terrain_draws,
            gpu_cull,
            vis,
            spheres,
            transforms,
            origins,
            mut cull_inside,
            shelter,
            local_shadows,
        }: PlanShadowsInputs<'_>,
    ) -> PlanShadowsOutputs {
        let meshlet_gpu = self.visibility.meshlet.as_ref();
        let cascades: Vec<Cascade> = if light.cast_shadows && has_casters {
            let bounds = match light.shadow_bounds {
                ShadowBounds::Sphere { center, radius } => (center, radius.max(1e-3)),
                ShadowBounds::Auto => {
                    // The instances' box (kept while they stand still), then
                    // the skinned instances and the terrains.
                    let skinned = scene.objects.skinned_instances.iter().map(|s| {
                        let r = self.cache.meshes[s.mesh.index()]
                            .bounds
                            .map_or(1.0, |(_, e)| e.length() * 2.0);
                        (s.transform.w_axis.truncate(), r)
                    });
                    let all = instances::fold_box(sphere_box, skinned);
                    Self::instance_bounds(instances::fold_box(
                        all,
                        if WORLD_3D {
                            self.world.terrain.bounds(scene)
                        } else {
                            Vec::new()
                        },
                    ))
                }
            };
            if light.shadow_box > 0.0 {
                let level = Vec3::new(cam.forward.x, 0.0, cam.forward.z).normalize_or(Vec3::NEG_Z);
                let focus = camera.position + level * light.shadow_focus
                    - Vec3::Y * light.shadow_focus_drop;
                frame::fit_box(
                    to_light,
                    bounds,
                    focus,
                    light.shadow_box,
                    shadow_size,
                    light.shadow_bias,
                    light.shadow_normal_bias,
                    scene.origin,
                )
            } else {
                frame::fit_cascades(
                    camera,
                    cam,
                    aspect,
                    to_light,
                    bounds,
                    light.max_shadow_distance,
                    shadow_size,
                    self.frame_cascades(scene) as usize,
                    light.cascade_blend,
                    scene.origin,
                )
            }
        } else {
            Vec::new()
        };
        // Each shadow pass's meshlet view (0: none), by cascade and by
        // local layer; its clustered casters leave the pass's list.
        let mut cascade_views: Vec<u32> = Vec::new();
        let mut local_views = [0u32; LOCAL_SHADOW_LAYERS];
        let caster_list = |g: u32, view: u32, keep: &dyn Fn(&Batch) -> bool| {
            let mut list = indirect_list(batches, cull_n, g, |b| b.cast && keep(b));
            if view > 0 {
                list.ranges
                    .retain(|r| meshlet_casters.binary_search(&r.0).is_err());
            }
            list
        };
        // The static shadow cache (shadow_cache.rs): what each cascade and
        // local face draws this frame. A layer needs its dynamic casters
        // drawn over the cached static ones when one reaches it, or when it
        // draws casters every frame whatever they do: the skinned ones and
        // scattered copies (unculled), the clustered ones (their level
        // follows the camera), terrain levels in its volume (the clipmap
        // follows the camera) and grass (the first cascade).
        let dynamic = crate::shadow_cache::DynamicFirsts::new(
            batches
                .iter()
                .zip(&shadow_split.dynamic)
                .map(|(b, d)| (b.first, b.count, *d)),
        );
        let is_dynamic = |b: &Batch| dynamic.contains(b.first, b.count);
        let unculled_overlay = skinned_batches.iter().any(|b| b.cast && !b.transparent)
            || !scatter_batches.scatters.is_empty()
            || !scatter_batches.impostors.is_empty()
            || !meshlet_casters.is_empty();
        let grass_casts = grass_frame.casts();
        let overlay_in = |slot: usize, f: &Frustum| {
            unculled_overlay
                || (slot < grass::SHADOW_CASCADES && grass_casts)
                || (terrain_casters
                    && terrain_draws
                        .iter()
                        .filter(|d| d.cast_shadows)
                        .flat_map(|d| &d.levels)
                        .any(|l| f.intersects_sphere(l.center, l.radius)))
                || crate::shadow_cache::ShadowCache::moving_in(shadow_split, f)
        };
        let cascade_plans: Vec<crate::shadow_cache::Plan> = cascades
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let key = crate::shadow_cache::view_key(
                    &c.view_proj.to_cols_array_2d(),
                    [0.0; 4],
                    [0.0; 4],
                    scene.origin,
                    shadow_size,
                );
                self.shadows.shadow_cache.plan(
                    i,
                    key,
                    &c.frustum,
                    overlay_in(i, &c.frustum),
                    shadow_split.any_static,
                )
            })
            .collect();
        // A layer's two lists: its static casters (every caster when it is
        // drawn in one pass) and the dynamic ones drawn over them.
        let static_keep = |plan: &crate::shadow_cache::Plan, b: &Batch| {
            plan.draw_static && (plan.single() || !is_dynamic(b))
        };
        let dynamic_keep = |plan: &crate::shadow_cache::Plan, b: &Batch| {
            plan.overlay && !plan.single() && is_dynamic(b)
        };
        // What a layer's GPU pass counts (`CullPass::counted`): what its
        // two lists draw, every caster when it is drawn in one pass.
        let counted = |plan: &crate::shadow_cache::Plan| -> Option<Vec<u32>> {
            (!plan.single()).then(|| {
                batches
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.cast && (static_keep(plan, b) || dynamic_keep(plan, b)))
                    .map(|(i, _)| i as u32)
                    .collect()
            })
        };
        let cascade_lists: Vec<[DrawList; 2]> = cascades
            .iter()
            .zip(&cascade_plans)
            .map(|(c, plan)| {
                if !plan.draw_static && !plan.overlay {
                    cascade_views.push(0);
                    return Default::default();
                }
                if gpu_cull {
                    let g =
                        vis.gpu_pass(cull_gpu::PassKind::Shadow, &c.frustum, counted(plan), None);
                    let view = if plan.overlay {
                        vis.shadow_view(meshlet_gpu, batches, meshlet_casters, &c.frustum)
                    } else {
                        0
                    };
                    cascade_views.push(view);
                    [
                        caster_list(g, view, &|b| static_keep(plan, b)),
                        caster_list(g, view, &|b| dynamic_keep(plan, b)),
                    ]
                } else {
                    let mut cull = |keep: &dyn Fn(&Batch) -> bool, wanted: bool| {
                        if !wanted {
                            return DrawList::default();
                        }
                        self.cull(
                            batches,
                            spheres,
                            transforms,
                            origins,
                            &mut cull_inside,
                            &c.frustum,
                            true,
                            |b| b.cast && keep(b),
                        )
                        .0
                    };
                    [
                        cull(&|b| static_keep(plan, b), plan.draw_static),
                        cull(&|b| dynamic_keep(plan, b), plan.overlay && !plan.single()),
                    ]
                }
            })
            .collect();
        // The weather occlusion map's casters (not the skinned or the
        // scattered copies: see `shadow_pass`), when it is drawn again: its
        // clustered casters through a shadow view of the meshlet dispatch,
        // as a cascade's (its own planes, no cone or occlusion, the
        // camera's level per model), and its meshlet view (0: none).
        let mut shelter_view = 0;
        let shelter_list: Option<[DrawList; 2]> = shelter.as_ref().filter(|s| s.3).map(|s| {
            let list = if gpu_cull {
                let g = vis.gpu_pass(cull_gpu::PassKind::Shadow, &s.1, None, None);
                shelter_view = vis.shadow_view(meshlet_gpu, batches, meshlet_casters, &s.1);
                caster_list(g, shelter_view, &|_| true)
            } else {
                self.cull(
                    batches,
                    spheres,
                    transforms,
                    origins,
                    &mut cull_inside,
                    &s.1,
                    true,
                    |b| b.cast,
                )
                .0
            };
            [list, DrawList::default()]
        });

        let mut local_lists: Vec<Option<[DrawList; 2]>> =
            (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect();
        let mut local_frustums: Vec<Option<Frustum>> =
            (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect();
        let mut local_plans = [crate::shadow_cache::Plan::default(); LOCAL_SHADOW_LAYERS];
        for ls in local_shadows {
            for (face, (vp, f)) in ls.faces.iter().enumerate() {
                let layer = ls.layer + face;
                local_frustums[layer] = Some(f.clone());
                let key = crate::shadow_cache::view_key(
                    &vp.to_cols_array_2d(),
                    ls.position.extend(ls.range).to_array(),
                    local_paraboloid(ls.paraboloid, face, self.shadows.local_shadow_size),
                    scene.origin,
                    self.shadows.local_shadow_size,
                );
                let plan = self.shadows.shadow_cache.plan(
                    CASCADES + layer,
                    key,
                    f,
                    overlay_in(CASCADES + layer, f),
                    shadow_split.any_static,
                );
                local_plans[layer] = plan;
                let list = if !plan.draw_static && !plan.overlay {
                    Default::default()
                } else if gpu_cull {
                    let g = vis.gpu_pass(cull_gpu::PassKind::Shadow, f, counted(&plan), None);
                    let view = if plan.overlay {
                        vis.shadow_view(meshlet_gpu, batches, meshlet_casters, f)
                    } else {
                        0
                    };
                    local_views[layer] = view;
                    [
                        caster_list(g, view, &|b| static_keep(&plan, b)),
                        caster_list(g, view, &|b| dynamic_keep(&plan, b)),
                    ]
                } else {
                    let mut cull = |keep: &dyn Fn(&Batch) -> bool, wanted: bool| {
                        if !wanted {
                            return DrawList::default();
                        }
                        self.cull(
                            batches,
                            spheres,
                            transforms,
                            origins,
                            &mut cull_inside,
                            f,
                            true,
                            |b| b.cast && keep(b),
                        )
                        .0
                    };
                    [
                        cull(&|b| static_keep(&plan, b), plan.draw_static),
                        cull(&|b| dynamic_keep(&plan, b), plan.overlay && !plan.single()),
                    ]
                };
                local_lists[layer] = Some(list);
            }
        }
        self.visibility.cull_inside = cull_inside;
        // The cache textures this frame's plans copy through, made at their
        // layer's side on first need.
        let local_side = self.shadows.local_shadow_size;
        let copied = cascade_plans
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p, shadow_size, wgpu::TextureFormat::R32Float))
            .chain(
                local_plans
                    .iter()
                    .enumerate()
                    .map(|(l, p)| (CASCADES + l, p, local_side, LOCAL_SHADOW_FORMAT)),
            )
            .filter(|(_, p, _, _)| p.save || p.restore);
        for (slot, _, side, format) in copied {
            if self.shadows.shadow_cache_maps.len() <= slot {
                self.shadows
                    .shadow_cache_maps
                    .resize_with(slot + 1, || None);
            }
            if self.shadows.shadow_cache_maps[slot]
                .as_ref()
                .is_none_or(|c| c.size != side)
            {
                self.shadows.shadow_cache_maps[slot] =
                    Some(CachedShadow::new(&self.device, side, format));
            }
        }
        PlanShadowsOutputs {
            cascades,
            cascade_views,
            local_views,
            cascade_plans,
            cascade_lists,
            shelter_view,
            shelter_list,
            local_lists,
            local_frustums,
            local_plans,
        }
    }
}
