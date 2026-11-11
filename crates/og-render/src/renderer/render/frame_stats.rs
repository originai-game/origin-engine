//! The frame's statistics (FrameStats) as far as the planning knows them:
//! its batches, instances, lists, lights, sprites, particles, strips and
//! lines, OIT's mode and overflow, GPU culling's lagged counts and the
//! overlay's draws.

use super::*;

pub(super) struct FrameStatsInputs<'a> {
    pub scene: &'a RenderScene,
    pub batches: &'a [Batch],
    pub instance_upload_bytes: u64,
    pub culled: u32,
    pub main_list: &'a DrawList,
    pub blended_list: &'a DrawList,
    pub local_shadows: &'a [frame::LocalShadow],
    pub shadowed_dropped: u32,
    pub sprite_frame: &'a sprites::SpriteFrame,
    pub particle_frame: &'a particles_gpu::ParticleFrame,
    pub ribbon_frame: &'a crate::ribbons::RibbonFrame,
    pub line_frame: &'a crate::lines::LineFrame,
    pub decals: u32,
    pub decals_dropped: u32,
    pub lights_2d: u32,
    pub occluders_2d: u32,
    pub joints_drawn: u32,
    pub taa_blend: bool,
    pub oit_mode: Option<oit::Oit>,
    pub gpu_counts: Option<(Option<cull_gpu::GpuCounts>, u32)>,
    pub overlay_frame: &'a Option<crate::overlay::OverlayFrame>,
    pub skinned_visible: &'a [Batch],
    pub skinned_gpu: bool,
}

pub(super) struct FrameStatsOutputs {
    pub stats: FrameStats,
}

impl Renderer {
    /// Gathers the frame's statistics from its plans.
    #[allow(clippy::too_many_lines)]
    pub(super) fn frame_stats(
        &self,
        FrameStatsInputs {
            scene,
            batches,
            instance_upload_bytes,
            culled,
            main_list,
            blended_list,
            local_shadows,
            shadowed_dropped,
            sprite_frame,
            particle_frame,
            ribbon_frame,
            line_frame,
            decals,
            decals_dropped,
            lights_2d,
            occluders_2d,
            joints_drawn,
            taa_blend,
            oit_mode,
            gpu_counts,
            overlay_frame,
            skinned_visible,
            skinned_gpu,
        }: FrameStatsInputs<'_>,
    ) -> FrameStatsOutputs {
        let cluster_stats = &self.lights.clusters.stats;
        let mut stats = FrameStats {
            instance_upload_bytes,
            terrain_tiles: self.world.terrain.resident,
            terrain_uploads: self.world.terrain.uploads,
            scatter_built: self.world.scatter.built,
            batches: batches.len() as u32,
            instances: scene.objects.instances.len() as u32,
            culled,
            triangles: main_list.triangles + blended_list.triangles,
            in_view_triangles: main_list.in_view_triangles,
            lights: self.lights.clusters.kept.len() as u32,
            shadowed_lights: local_shadows.len() as u32,
            shadowed_lights_dropped: shadowed_dropped,
            lights_dropped: cluster_stats.dropped,
            lights_culled: cluster_stats.culled,
            cluster_lights: cluster_stats.refs,
            cluster_lights_dropped: cluster_stats.refs_dropped,
            texture_bytes: self.cache.textures.iter().map(|t| t.bytes).sum::<u64>()
                + self.asset_bytes(),
            sprites: sprite_frame.sprites,
            sprites_culled: sprite_frame.culled,
            tiles: sprite_frame.tiles,
            tile_chunks: sprite_frame.tile_chunks,
            glyphs: sprite_frame.glyphs,
            sprite_draws: (sprite_frame.runs.len()
                + sprite_frame.billboard_shadows.len()
                + sprites::draw_runs_of(&sprite_frame.billboards).len())
                as u32,
            missing_glyphs: sprite_frame.missing_glyphs,
            transparent: blended_list.instances,
            transparent_draws: blended_list.ranges.len() as u32,
            graph_passes: self.graph.order().len() as u32,
            transient_textures: self.graph.transient_textures() as u32,
            frame: self.frame_index,
            particle_systems: scene.effects.particles.len() as u32,
            particle_capacity: particle_frame.capacity,
            particles_cpu: particle_frame.cpu_alive,
            particle_ticks: particle_frame.ticks,
            particle_collision_skipped: particle_frame.collision_skipped,
            strips: ribbon_frame.strips,
            strip_segments: ribbon_frame.segments,
            line_instances: line_frame.instances,
            decals,
            decals_dropped,
            lights_2d,
            occluders_2d,
            skinned_instances: scene.objects.skinned_instances.len() as u32,
            joints: joints_drawn,
            taa_history: taa_blend,
            ..Default::default()
        };
        // Order-independent transparency (oit.rs): this frame's mode, and
        // the newest overflow count read back from the linked lists.
        stats.transparency_oit = oit_mode.is_some();
        if let Some(arena) = self
            .oit
            .arena
            .as_ref()
            .filter(|_| oit_mode == Some(oit::Oit::List))
        {
            stats.transparency_layers = arena.size.nodes_per_pixel;
            match arena.poll() {
                Some((frame, pixels)) => {
                    stats.transparency_fallback_pixels = pixels;
                    stats.transparency_fallback_lag = (self.frame_index - frame) as u32;
                }
                None => stats.transparency_fallback_lag = u32::MAX,
            }
        }
        if let Some((lagged, occluder_draws)) = gpu_counts {
            let c = lagged.unwrap_or_default();
            stats.gpu_culling = true;
            stats.culled += c.culled;
            stats.occluded = c.occluded;
            stats.occluded_triangles = c.occluded_triangles;
            stats.in_view_triangles = c.triangles + c.occluded_triangles;
            stats.triangles += c.triangles;
            stats.shadow_casters += c.shadow_casters;
            stats.counts_lag = lagged.map_or(u32::MAX, |l| (self.frame_index - l.frame) as u32);
            stats.occluder_draws = occluder_draws;
            stats.draw_calls += occluder_draws;
        }
        if let Some(of) = overlay_frame {
            stats.overlay_sprites = of.sprites;
            stats.overlay_glyphs = of.glyphs;
            stats.overlay_draws = of.runs.len() as u32;
            stats.sprites += of.sprites;
            stats.glyphs += of.glyphs;
            stats.sprite_draws += of.runs.len() as u32;
            stats.missing_glyphs += of.missing_glyphs;
        }
        // Under GPU culling its counts hold the skinned draws too.
        for b in skinned_visible.iter().filter(|_| !skinned_gpu) {
            stats.triangles += (self.cache.meshes[b.mesh].index_count / 3) as u64 * b.count as u64;
        }
        FrameStatsOutputs { stats }
    }
}
