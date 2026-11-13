//! How one shadow layer is encoded: a cascade's, the weather shelter
//! map's or a local light's face, each drawing its casters as its cache
//! plan says.

use super::*;

impl DrawCtx<'_> {
    /// One shadow layer: a cascade or a local light's face, as its plan
    /// (shadow_cache.rs) says: its static casters, the copy into or out
    /// of its cache texture, its dynamic casters over them. `target`: the
    /// live array, the layer's index in it and the depth buffer, which
    /// the cache copies. `all_casters`: false for the weather occlusion
    /// map, which takes only what stands still for its key
    /// (weather::shelter_key): no skinned casters, no scattered copies
    /// (grass must not keep the ground under it dry). `targets`: the
    /// draws of every caster, in order, each into its target
    /// (shadow_target.rs): a cascade's, or a local layer's far sides
    /// then near sides.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn shadow_pass(
        &self,
        encoder: &mut crate::Encoder,
        color: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        target: (&wgpu::Texture, u32, &wgpu::Texture),
        slot: usize,
        lists: &[DrawList; 2],
        plan: crate::shadow_cache::Plan,
        frustum: &Frustum,
        view: u32,
        stats: &mut FrameStats,
        all_casters: bool,
        targets: &[ShadowTarget],
        // How far from the camera the pass's receivers lie (a cascade's
        // `reach`): grass farther than its shadow reaches stays out.
        reach: f32,
    ) {
        let &DrawCtx {
            r,
            batches,
            culled_gpu,
            skinned_gpu,
            terrain_indices,
            terrain_draws,
            scatter_batches,
            grass_frame,
            shadow_skinned,
            skinned_batches,
            terrain_casters,
            ..
        } = self;
        stats.shadow_passes += 1;
        stats.shadow_cached += plan.cached() as u32;
        // What every pass of every caster draws besides the instances.
        let draw_rest = |pass: &mut wgpu::RenderPass,
                         shadow: ShadowTarget,
                         stats: &mut FrameStats| {
            // The clustered casters (meshlet_gpu.rs), each one draw of
            // the clusters the pass's view kept.
            if view > 0
                && let Some(m) = r.visibility.meshlet.as_ref()
            {
                stats.draw_calls += m.draw_shadow(pass, shadow, batches, view, |b| {
                    &r.cache.materials[b.material].bind_group
                });
            }
            // Terrain levels that reach into the pass's volume.
            if let Some(indices) = terrain_indices
                && terrain_casters
            {
                let (n, _) = terrain::draw_levels(
                    pass,
                    terrain_draws,
                    indices,
                    terrain::TerrainPass::Shadow(shadow),
                    frustum,
                    None,
                );
                stats.draw_calls += n;
                stats.terrain_shadow_draws += n;
            }
            if !all_casters {
                return;
            }
            // Skinned casters: those GPU culling's pass for the layer
            // kept (its counts hold them), unculled on the CPU path.
            let g = lists[0].indirect.or(lists[1].indirect);
            let pipelines: Vec<&wgpu::RenderPipeline> =
                shadow_skinned.iter().map(|s| s.get(shadow)).collect();
            let n = self.draw_skinned(
                pass,
                skinned_batches,
                false,
                1,
                2,
                &pipelines,
                None,
                None,
                g,
            );
            stats.draw_calls += n;
            if !(skinned_gpu && g.is_some()) {
                stats.shadow_casters += skinned_batches
                    .iter()
                    .filter(|b| !b.transparent && b.cast)
                    .map(|b| b.count)
                    .sum::<u32>();
            }
            let (n, _) = r.world.scatter.draw(
                pass,
                scatter::ScatterPass::Shadow,
                scatter_batches,
                |m| self.scatter_material(m),
                |m| self.scatter_mesh(m),
                |c, r| frustum.intersects_sphere(c, r),
                None,
                None,
                false,
                self.card_source(g),
                shadow,
            );
            stats.draw_calls += n; // Grass casts into the first cascades only (grass.rs).
            if slot < grass::SHADOW_CASCADES {
                stats.draw_calls +=
                    r.world
                        .grass
                        .draw(pass, grass::GrassPass::Shadow, grass_frame, None, reach);
            }
        };
        let draw = |encoder: &mut crate::Encoder,
                    list: &DrawList,
                    rest: bool,
                    load: bool,
                    keep_depth: bool,
                    stats: &mut FrameStats| {
            let mut pass = shadow_target(encoder, color, depth, load, keep_depth);
            // No pipeline first: the draw list sets each batch's own, as
            // do the meshlet, skinned, terrain and scatter casters.
            pass.set_bind_group(
                0,
                &r.shadows.shadow_bind_group,
                &[(slot as u64 * SHADOW_SLOT_STRIDE) as u32],
            );
            for &shadow in targets {
                r.frame_draw_list(DrawListInputs {
                    pass: &mut pass,
                    list,
                    lit: false,
                    shadow,
                    current: &mut None,
                    physical_fx: None,
                    probes: None,
                    fx: None,
                    batches,
                    culled_gpu,
                    fx_lean_of: &|fx: &wgpu::BindGroup| self.fx_lean_of(fx),
                    oit: None,
                });
                stats.draw_calls += list.ranges.len() as u32;
                if rest {
                    draw_rest(&mut pass, shadow, stats);
                }
            }
            stats.shadow_casters += list.instances;
        };
        // The layer and its cache texture (and both depths), one way.
        let copy = |encoder: &mut crate::Encoder, into_cache: bool| {
            let cache = r.shadows.shadow_cache_maps[slot]
                .as_ref()
                .expect("made for its plan");
            let (array, layer, depth_texture) = target;
            let at = |texture, z| wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z },
                aspect: wgpu::TextureAspect::All,
            };
            let size = wgpu::Extent3d {
                width: cache.size,
                height: cache.size,
                depth_or_array_layers: 1,
            };
            let pairs = [
                (at(array, layer), at(&cache.color, 0)),
                (at(depth_texture, 0), at(&cache.depth, 0)),
            ];
            for (live, kept) in pairs {
                if into_cache {
                    encoder.copy_texture_to_texture(live, kept, size);
                } else {
                    encoder.copy_texture_to_texture(kept, live, size);
                }
            }
        };
        if plan.draw_static {
            // Every caster when it is one pass; the depth kept for the
            // dynamic casters when they follow.
            draw(encoder, &lists[0], plan.single(), false, plan.save, stats);
        }
        if plan.save {
            copy(encoder, true);
        }
        if plan.restore {
            copy(encoder, false);
        }
        if plan.overlay && !plan.single() {
            draw(encoder, &lists[1], true, true, false, stats);
        }
    }
}
