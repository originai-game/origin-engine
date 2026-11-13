//! How one shadow layer is encoded: a cascade's, the weather shelter
//! map's or a local light's face, each drawing its casters as its cache
//! plan says.

use super::*;

impl DrawCtx<'_> {
    /// One shadow layer: a cascade or a local light's face, as its plan
    /// (shadow_cache.rs) says: its static casters, their copy into the
    /// layer's kept texture (shadow_keep.rs), then that put back with
    /// its dynamic casters over it. `target`: the live array, the
    /// layer's index in it and the depth buffer, which are saved. `all_casters`: false for the weather occlusion
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
        let kept = || {
            let layer = r.shadows.shadow_cache_maps[slot].as_ref();
            r.shadows
                .shadow_keep
                .as_ref()
                .zip(layer)
                .expect("made for its plan")
        };
        // One pass into the cleared layer: the kept static casters put
        // back first (`over_kept`), then `list` and with `rest` the other
        // casters.
        let draw = |encoder: &mut crate::Encoder,
                    list: Option<&DrawList>,
                    rest: bool,
                    over_kept: bool,
                    keep_depth: bool,
                    stats: &mut FrameStats| {
            let mut pass = shadow_target(encoder, color, depth, keep_depth);
            if over_kept {
                let (keep, layer) = kept();
                keep.put_back(&mut pass, layer, targets);
            }
            let Some(list) = list else { return };
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
        if plan.draw_static {
            // Every caster when it is one pass; a local layer's depth kept
            // for the copy when it is saved.
            let keep_depth = plan.save && kept().1.keeps_depth();
            draw(
                encoder,
                Some(&lists[0]),
                plan.single(),
                false,
                keep_depth,
                stats,
            );
        }
        if plan.save {
            kept().1.save(encoder, target);
        }
        // The kept casters come back in the pass that draws over them,
        // or alone into a layer that last held dynamic casters too.
        if plan.over_kept() {
            draw(encoder, Some(&lists[1]), true, true, false, stats);
        } else if plan.restore {
            draw(encoder, None, false, true, false, stats);
        }
    }
}
