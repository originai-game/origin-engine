//! The shadow passes: the sun's cascades, the weather's shelter map in the
//! cascade array's last layer, and the shadowed local lights' faces, each
//! drawing its casters into its layer of the shadow arrays.
//!
//! Their plan steps: the sun's cascades and their static cache plans, the
//! weather's shelter map, and the shadowed local lights' faces, each with
//! its casters' lists ([`Casters`]), registered with the visibility stage
//! in that order (cascades, shelter, local faces) and culled on the CPU
//! where the GPU cull does not take them; their upload steps make the
//! cache textures the plans copy through, at their layer's side on first
//! need.

use super::*;

/// What every shadow pass's plan reads of its casters: the frame's
/// culling inputs, the clustered casters, which batches the static cache
/// keeps apart, and what a layer draws every frame whatever it does.
pub(super) struct Casters<'a> {
    pub seen: CullFrame<'a>,
    pub meshlet_casters: &'a [u32],
    pub shadow_split: &'a shadow_cache::Split,
    dynamic: shadow_cache::DynamicFirsts,
    /// The casters drawn unculled over every layer: skinned, scattered
    /// copies, clustered.
    unculled_overlay: bool,
    grass_casts: bool,
    terrain_casters: bool,
    terrain_draws: &'a [terrain::TerrainDraw],
}

impl<'a> Casters<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        seen: CullFrame<'a>,
        meshlet_casters: &'a [u32],
        shadow_split: &'a shadow_cache::Split,
        skinned_batches: &[Batch],
        scatter_batches: &scatter::ScatterBatches,
        grass_frame: &grass::GrassFrame,
        terrain_casters: bool,
        terrain_draws: &'a [terrain::TerrainDraw],
    ) -> Casters<'a> {
        // The static shadow cache (shadow_cache.rs): what each cascade and
        // local face draws this frame. A layer needs its dynamic casters
        // drawn over the cached static ones when one reaches it, or when it
        // draws casters every frame whatever they do: the skinned ones and
        // scattered copies (unculled), the clustered ones (their level
        // follows the camera), terrain levels in its volume (the clipmap
        // follows the camera) and grass (the first cascade).
        let dynamic = crate::shadow_cache::DynamicFirsts::new(
            seen.batches
                .iter()
                .zip(&shadow_split.dynamic)
                .map(|(b, d)| (b.first, b.count, *d)),
        );
        let unculled_overlay = skinned_batches.iter().any(|b| b.cast && !b.transparent)
            || !scatter_batches.scatters.is_empty()
            || !scatter_batches.impostors.is_empty()
            || !meshlet_casters.is_empty();
        Casters {
            seen,
            meshlet_casters,
            shadow_split,
            dynamic,
            unculled_overlay,
            grass_casts: grass_frame.casts(),
            terrain_casters,
            terrain_draws,
        }
    }

    /// The casters of GPU pass `g` that `keep` keeps; with a meshlet
    /// `view`, less its clustered casters.
    fn caster_list(&self, g: u32, view: u32, keep: &dyn Fn(&Batch) -> bool) -> DrawList {
        let mut list = indirect_list(self.seen.batches, self.seen.cull_n, g, |b| {
            b.cast && keep(b)
        });
        if view > 0 {
            list.ranges
                .retain(|r| self.meshlet_casters.binary_search(&r.0).is_err());
        }
        list
    }

    fn is_dynamic(&self, b: &Batch) -> bool {
        self.dynamic.contains(b.first, b.count)
    }

    /// Whether the layer at shadow slot `slot` draws dynamic casters over
    /// its cached static ones: those in `f`, what the frame culls its own
    /// casters to.
    fn overlay_in(&self, slot: usize, f: &Frustum) -> bool {
        self.unculled_overlay
            || (slot < grass::SHADOW_CASCADES && self.grass_casts)
            || (self.terrain_casters
                && self
                    .terrain_draws
                    .iter()
                    .filter(|d| d.cast_shadows)
                    .flat_map(|d| &d.levels)
                    .any(|l| f.intersects_sphere(l.center, l.radius)))
            || crate::shadow_cache::ShadowCache::moving_in(self.shadow_split, f)
    }

    /// A layer's two lists: its static casters (every caster when it is
    /// drawn in one pass) and the dynamic ones drawn over them.
    fn static_keep(&self, plan: &crate::shadow_cache::Plan, b: &Batch) -> bool {
        plan.draw_static && (plan.single() || !self.is_dynamic(b))
    }

    fn dynamic_keep(&self, plan: &crate::shadow_cache::Plan, b: &Batch) -> bool {
        plan.overlay && !plan.single() && self.is_dynamic(b)
    }

    /// What a layer's GPU pass counts (`CullPass::counted`): what its
    /// two lists draw, every caster when it is drawn in one pass.
    fn counted(&self, plan: &crate::shadow_cache::Plan) -> Option<Vec<u32>> {
        (!plan.single()).then(|| {
            self.seen
                .batches
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    b.cast && (self.static_keep(plan, b) || self.dynamic_keep(plan, b))
                })
                .map(|(i, _)| i as u32)
                .collect()
        })
    }

    /// The two lists of a cascade or local face seeing `frustum` under
    /// its cache `plan`, and its meshlet view (0: none): registered as a
    /// GPU shadow pass under GPU culling, else culled here.
    fn layer_lists(
        &self,
        r: &Renderer,
        cull: &mut FrameCulling,
        frustum: &Frustum,
        plan: &crate::shadow_cache::Plan,
    ) -> ([DrawList; 2], u32) {
        if !plan.draw_static && !plan.overlay {
            return (Default::default(), 0);
        }
        let batches = self.seen.batches;
        if self.seen.gpu_cull {
            let meshlet_gpu = r.visibility.meshlet.as_ref();
            let g = cull.vis.gpu_pass(
                cull_gpu::PassKind::Shadow,
                frustum,
                self.counted(plan),
                None,
            );
            let view = if plan.overlay {
                cull.vis
                    .shadow_view(meshlet_gpu, batches, self.meshlet_casters, frustum)
            } else {
                0
            };
            (
                [
                    self.caster_list(g, view, &|b| self.static_keep(plan, b)),
                    self.caster_list(g, view, &|b| self.dynamic_keep(plan, b)),
                ],
                view,
            )
        } else {
            let mut list = |keep: &dyn Fn(&Batch) -> bool, wanted: bool| {
                if !wanted {
                    return DrawList::default();
                }
                r.cull(
                    batches,
                    self.seen.spheres,
                    &mut cull.transforms,
                    &mut cull.origins,
                    &mut cull.cull_inside,
                    frustum,
                    true,
                    |b| b.cast && keep(b),
                )
                .0
            };
            (
                [
                    list(&|b| self.static_keep(plan, b), plan.draw_static),
                    list(
                        &|b| self.dynamic_keep(plan, b),
                        plan.overlay && !plan.single(),
                    ),
                ],
                0,
            )
        }
    }
}

/// Makes the textures the plans of `kept` keep their layers in
/// (`shadow_keep.rs`), at their layer's side (`(slot, plan, side,
/// format)`), on first need.
fn cache_maps<'p>(
    r: &mut Renderer,
    kept: impl Iterator<
        Item = (
            usize,
            &'p crate::shadow_cache::Plan,
            u32,
            wgpu::TextureFormat,
        ),
    >,
) {
    for (slot, _, side, format) in kept.filter(|(_, p, _, _)| p.keeps()) {
        if r.shadows.shadow_cache_maps.len() <= slot {
            r.shadows.shadow_cache_maps.resize_with(slot + 1, || None);
        }
        if r.shadows.shadow_cache_maps[slot]
            .as_ref()
            .is_none_or(|c| c.size != side)
        {
            let keep = r
                .shadows
                .shadow_keep
                .get_or_insert_with(|| crate::shadow_keep::ShadowKeep::new(&r.device));
            r.shadows.shadow_cache_maps[slot] = Some(keep.layer(&r.device, side, format));
        }
    }
}

/// A frame's cascades: the point they are nested round, then each one's
/// view, casters, cache plan and meshlet view (0: none), by cascade.
#[derive(Default)]
pub(super) struct CascadeFrame {
    pub anchor: Vec3,
    pub cascades: Vec<Cascade>,
    pub lists: Vec<[DrawList; 2]>,
    pub plans: Vec<crate::shadow_cache::Plan>,
    pub views: Vec<u32>,
}

/// What the cascades' plan reads: the frame's culling and casters, the
/// scene, its camera and sun, and what the cascades are fitted to.
pub(super) struct CascadeIn<'a> {
    pub cull: &'a mut FrameCulling,
    pub casters: &'a Casters<'a>,
    pub scene: &'a RenderScene,
    pub camera: &'a Camera,
    pub cam: &'a frame::CameraFrame,
    pub light: &'a crate::DirectionalLight,
    /// Whether anything casts (meshes, terrain, grass, scattered copies).
    pub has_casters: bool,
    /// The sorted instances' box (debris widened).
    pub sphere_box: Option<(Vec3, Vec3)>,
    pub to_light: Vec3,
    pub shadow_size: u32,
    pub aspect: f32,
}

pub(super) struct CascadePass;

impl Pass for CascadePass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Cascade(0),
        name: "cascade",
        needs: Caps::NONE,
    };
    type State = ShadowMap;
    type Frame = CascadeFrame;
    type PlanIn<'a> = CascadeIn<'a>;
    /// The cascades' side.
    type UploadIn<'a> = u32;
    type BindIn<'f> = ();
    type Plan<'f> = &'f CascadeFrame;

    fn state(r: &Renderer) -> &Self::State {
        &r.shadows.cascade_map
    }

    /// The cascades fitted to the view (or the shadow box), each one's
    /// cache plan, then each one's lists.
    fn plan(
        r: &mut Renderer,
        CascadeIn {
            cull,
            casters,
            scene,
            camera,
            cam,
            light,
            has_casters,
            sphere_box,
            to_light,
            shadow_size,
            aspect,
        }: CascadeIn<'_>,
    ) -> CascadeFrame {
        let (anchor, cascades) = if light.cast_shadows && has_casters {
            let bounds = match light.shadow_bounds {
                ShadowBounds::Sphere { center, radius } => (center, radius.max(1e-3)),
                ShadowBounds::Auto => {
                    // The instances' box (kept while they stand still), then
                    // the skinned instances and the terrains.
                    let skinned = scene.objects.skinned_instances.iter().map(|s| {
                        let r = r.cache.meshes[s.mesh.index()]
                            .bounds
                            .map_or(1.0, |(_, e)| e.length() * 2.0);
                        (s.transform.w_axis.truncate(), r)
                    });
                    let all = instances::fold_box(sphere_box, skinned);
                    Renderer::instance_bounds(instances::fold_box(
                        all,
                        if WORLD_3D {
                            r.world.terrain.bounds(scene)
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
                let fixed = shadow_fit::fit_box(
                    to_light,
                    bounds,
                    focus,
                    light.shadow_box,
                    shadow_size,
                    light.shadow_bias,
                    light.shadow_normal_bias,
                    scene.frame.origin,
                );
                (camera.position, fixed)
            } else {
                shadow_fit::fit_cascades(
                    camera,
                    cam,
                    aspect,
                    to_light,
                    bounds,
                    light.max_shadow_distance,
                    shadow_size,
                    r.frame_cascades(scene) as usize,
                    light.cascade_blend,
                    scene.frame.origin,
                )
            }
        } else {
            (camera.position, Vec::new())
        };
        let plans: Vec<crate::shadow_cache::Plan> = cascades
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let key = crate::shadow_cache::view_key(
                    &c.view_proj.to_cols_array_2d(),
                    [0.0; 4],
                    [0.0; 4],
                    scene.frame.origin,
                    shadow_size,
                );
                r.shadows.shadow_cache.plan(
                    i,
                    key,
                    &c.frustum,
                    casters.overlay_in(i, &c.seen),
                    u64::from(shadow_size).pow(2),
                )
            })
            .collect();
        let mut views = Vec::new();
        let lists = cascades
            .iter()
            .zip(&plans)
            .map(|(c, plan)| {
                // A layer drawn to be kept holds everything in its box; a
                // frame's own casters are culled to what the camera sees.
                let kept = plan.draw_static && !plan.single() && casters.shadow_split.any_static;
                let f = if kept { &c.frustum } else { &c.seen };
                let (lists, view) = casters.layer_lists(r, cull, f, plan);
                views.push(view);
                lists
            })
            .collect();
        CascadeFrame {
            anchor,
            cascades,
            lists,
            plans,
            views,
        }
    }

    fn upload(r: &mut Renderer, frame: &mut CascadeFrame, shadow_size: u32) {
        cache_maps(
            r,
            frame
                .plans
                .iter()
                .enumerate()
                .map(|(i, p)| (i, p, shadow_size, crate::shadow_keep::CASCADE_FORMAT)),
        );
    }

    fn bind<'f>(_: &'f Renderer, frame: &'f mut CascadeFrame, _: ()) -> Option<&'f CascadeFrame> {
        Some(frame)
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &&'f CascadeFrame,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let FramePass::Cascade(i) = pass else {
            unreachable!("a cascade pass")
        };
        cx.ctx.shadow_pass(
            cx.encoder,
            &st.layers[i as usize],
            &st.depth,
            (&st.texture, i, &st.depth_texture),
            i as usize,
            &plan.lists[i as usize],
            plan.plans[i as usize],
            &plan.cascades[i as usize].seen,
            plan.views.get(i as usize).copied().unwrap_or(0),
            cx.stats,
            true,
            &[ShadowTarget::Cascade],
            plan.cascades[i as usize].reach,
        );
    }
}

/// A frame's shelter map: its casters when it is drawn again, and its
/// meshlet view (0: none).
#[derive(Default)]
pub(super) struct ShelterFrame {
    pub list: Option<[DrawList; 2]>,
    pub view: u32,
}

/// The shelter map's view, frustum and key, and whether it is drawn
/// again (shadow_inputs.rs).
type Shelter = Option<(Mat4, Frustum, u64, bool)>;

/// What the shelter map's encode reads: its frustum and casters, when
/// redrawn.
pub(super) struct ShelterPlan<'f> {
    pub frustum: &'f Frustum,
    pub list: &'f [DrawList; 2],
    pub view: u32,
}

pub(super) struct ShelterPass;

impl Pass for ShelterPass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::Shelter,
        name: "shelter",
        needs: Caps::NONE,
    };
    type State = ShadowMap;
    type Frame = ShelterFrame;
    /// The frame's culling, its casters and the shelter map.
    type PlanIn<'a> = (&'a mut FrameCulling, &'a Casters<'a>, &'a Shelter);
    type UploadIn<'a> = ();
    type BindIn<'f> = &'f Shelter;
    type Plan<'f> = ShelterPlan<'f>;

    fn state(r: &Renderer) -> &Self::State {
        &r.shadows.cascade_map
    }

    /// The weather occlusion map's casters (not the skinned or the
    /// scattered copies: see `shadow_pass`), when it is drawn again: its
    /// clustered casters through a shadow view of the meshlet dispatch,
    /// as a cascade's (its own planes, no cone or occlusion, the
    /// camera's level per model), and its meshlet view (0: none).
    fn plan(r: &mut Renderer, (cull, casters, shelter): Self::PlanIn<'_>) -> ShelterFrame {
        let mut view = 0;
        let list = shelter.as_ref().filter(|s| s.3).map(|s| {
            let list = if casters.seen.gpu_cull {
                let meshlet_gpu = r.visibility.meshlet.as_ref();
                let batches = casters.seen.batches;
                let g = cull
                    .vis
                    .gpu_pass(cull_gpu::PassKind::Shadow, &s.1, None, None);
                view = cull
                    .vis
                    .shadow_view(meshlet_gpu, batches, casters.meshlet_casters, &s.1);
                casters.caster_list(g, view, &|_| true)
            } else {
                r.cull(
                    casters.seen.batches,
                    casters.seen.spheres,
                    &mut cull.transforms,
                    &mut cull.origins,
                    &mut cull.cull_inside,
                    &s.1,
                    true,
                    |b| b.cast,
                )
                .0
            };
            [list, DrawList::default()]
        });
        ShelterFrame { list, view }
    }

    fn bind<'f>(
        _: &'f Renderer,
        frame: &'f mut ShelterFrame,
        shelter: &'f Shelter,
    ) -> Option<ShelterPlan<'f>> {
        shelter
            .as_ref()
            .zip(frame.list.as_ref())
            .map(|(s, list)| ShelterPlan {
                frustum: &s.1,
                list,
                view: frame.view,
            })
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &ShelterPlan<'f>,
        _: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        cx.ctx.shadow_pass(
            cx.encoder,
            st.layers.last().expect("fit_shelter_layer"),
            &st.depth,
            (&st.texture, st.layers.len() as u32 - 1, &st.depth_texture),
            SHELTER_SLOT,
            plan.list,
            crate::shadow_cache::Plan::ALL,
            plan.frustum,
            plan.view,
            cx.stats,
            false,
            &[ShadowTarget::Cascade],
            f32::INFINITY,
        );
    }
}

/// A frame's shadowed local lights' faces: each layer's casters, cache
/// plan, frustum and meshlet view (0: none), by layer.
pub(super) struct LocalFaceFrame {
    pub lists: Vec<Option<[DrawList; 2]>>,
    pub plans: [crate::shadow_cache::Plan; LOCAL_SHADOW_LAYERS],
    pub frustums: Vec<Option<Frustum>>,
    pub views: [u32; LOCAL_SHADOW_LAYERS],
}

impl Default for LocalFaceFrame {
    fn default() -> LocalFaceFrame {
        LocalFaceFrame {
            lists: (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect(),
            plans: [crate::shadow_cache::Plan::default(); LOCAL_SHADOW_LAYERS],
            frustums: (0..LOCAL_SHADOW_LAYERS).map(|_| None).collect(),
            views: [0; LOCAL_SHADOW_LAYERS],
        }
    }
}

pub(super) struct LocalFacePass;

impl Pass for LocalFacePass {
    const DESC: PassDesc = PassDesc {
        pass: FramePass::LocalFace(0),
        name: "local-shadows",
        needs: Caps::NONE,
    };
    type State = ShadowMap;
    type Frame = LocalFaceFrame;
    /// The frame's culling, its casters, the scene and its shadowed
    /// local lights (lights_stage.rs).
    type PlanIn<'a> = (
        &'a mut FrameCulling,
        &'a Casters<'a>,
        &'a RenderScene,
        &'a [frame::LocalShadow],
    );
    type UploadIn<'a> = ();
    type BindIn<'f> = ();
    type Plan<'f> = &'f LocalFaceFrame;

    fn state(r: &Renderer) -> &Self::State {
        &r.shadows.local_map
    }

    /// Each shadowed light's faces: its cache plan, then its lists.
    fn plan(
        r: &mut Renderer,
        (cull, casters, scene, local_shadows): Self::PlanIn<'_>,
    ) -> LocalFaceFrame {
        let mut frame = LocalFaceFrame::default();
        for ls in local_shadows {
            for (face, (vp, f)) in ls.faces.iter().enumerate() {
                let layer = ls.layer + face;
                frame.frustums[layer] = Some(f.clone());
                let key = crate::shadow_cache::view_key(
                    &vp.to_cols_array_2d(),
                    ls.position.extend(ls.range).to_array(),
                    local_paraboloid(ls.paraboloid, face, r.shadows.local_shadow_size),
                    scene.frame.origin,
                    r.shadows.local_shadow_size,
                );
                let plan = r
                    .shadows
                    .shadow_cache
                    .plan(
                        CASCADES + layer,
                        key,
                        f,
                        casters.overlay_in(CASCADES + layer, f),
                        u64::from(r.shadows.local_shadow_size).pow(2),
                    )
                    .with_own_depth();
                frame.plans[layer] = plan;
                let (lists, view) = casters.layer_lists(r, cull, f, &plan);
                frame.views[layer] = view;
                frame.lists[layer] = Some(lists);
            }
        }
        frame
    }

    fn upload(r: &mut Renderer, frame: &mut LocalFaceFrame, _: ()) {
        let side = r.shadows.local_shadow_size;
        cache_maps(
            r,
            frame
                .plans
                .iter()
                .enumerate()
                .map(|(l, p)| (CASCADES + l, p, side, LOCAL_SHADOW_FORMAT)),
        );
    }

    fn bind<'f>(
        _: &'f Renderer,
        frame: &'f mut LocalFaceFrame,
        _: (),
    ) -> Option<&'f LocalFaceFrame> {
        Some(frame)
    }

    fn encode<'f>(
        st: &Self::State,
        plan: &&'f LocalFaceFrame,
        pass: FramePass,
        cx: &mut EncodeCx<'_, 'f>,
    ) {
        let FramePass::LocalFace(layer) = pass else {
            unreachable!("a local face pass")
        };
        let l = layer as usize;
        cx.ctx.shadow_pass(
            cx.encoder,
            &st.layers[l],
            &st.depth,
            (&st.texture, layer, &st.depth_texture),
            CASCADES + l,
            plan.lists[l].as_ref().expect("a list per shadowed face"),
            plan.plans[l],
            plan.frustums[l]
                .as_ref()
                .expect("a frustum per shadowed face"),
            plan.views[l],
            cx.stats,
            true,
            &ShadowTarget::LOCAL,
            f32::INFINITY,
        );
    }
}
