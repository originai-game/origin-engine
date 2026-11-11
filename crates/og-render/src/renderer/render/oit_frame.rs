//! The frame's order-independent transparency: what the main camera's OIT
//! pass may draw (blended meshes, billboards, alpha particles and strips),
//! whether the frame runs it (three targets: never without MRT), and its
//! pipelines for every blended surface of the frame.

use super::*;

pub(super) struct PlanOitFrameInputs<'a> {
    pub scene: &'a RenderScene,
    pub batches: &'a [Batch],
    pub skinned_batches: &'a [Batch],
    pub sprite_frame: &'a sprites::SpriteFrame,
    pub ribbon_frame: &'a crate::ribbons::RibbonFrame,
    pub particle_frame: &'a particles_gpu::ParticleFrame,
}

pub(super) struct PlanOitFrameOutputs {
    pub oit_candidates: bool,
    pub oit_frame: Option<oit::Oit>,
}

impl Renderer {
    /// Decides the frame's OIT mode and makes its pipelines.
    #[allow(clippy::too_many_lines)]
    pub(super) fn plan_oit_frame(
        &mut self,
        PlanOitFrameInputs {
            scene,
            batches,
            skinned_batches,
            sprite_frame,
            ribbon_frame,
            particle_frame,
        }: PlanOitFrameInputs<'_>,
    ) -> PlanOitFrameOutputs {
        // Order-independent transparency (oit.rs): the main camera's
        // blended meshes, billboards, alpha particles and alpha strips go
        // to its OIT pass, whose pipelines are made here for every blended
        // surface of the frame (a debug view keeps the sorted blending:
        // its channels are not colours to weight).
        let oit_billboards: Vec<oit::Source> = sprite_frame
            .billboard_shadows
            .iter()
            .chain(sprite_frame.billboards.iter().map(|(_, r)| r))
            .filter_map(|r| self.oit_billboard(r, scene))
            .collect();
        let oit_strips = ribbon_frame
            .runs
            .iter()
            .filter(|r| r.layer.is_none() && r.blend == ParticleBlend::Alpha)
            .fold((false, false), |(v, p), r| (v || !r.points, p || r.points));
        let oit_particles = particle_frame.has_oit(scene.camera_layers);
        // What needs no culling: billboards, particles and strips.
        let oit_candidates =
            !oit_billboards.is_empty() || oit_particles || oit_strips.0 || oit_strips.1;
        // Its targets are three (sorted blending without MRT).
        let oit_frame = (scene.debug == DebugView::None
            && self.info.caps.contains(Caps::MRT)
            && (oit_candidates
                || batches.iter().any(|b| b.transparent && b.blend)
                || skinned_batches.iter().any(|b| b.transparent && b.blend)))
        .then(|| {
            self.oit
                .frame_mode(&self.device, &self.queue, self.width, self.height)
        });
        if let Some(mode) = oit_frame {
            // Fork s60: drawn by the main pass and the resolve, as the
            // materials' lit pipelines are.
            frame_pipelines(|| {
                self.ensure_oit_pipelines(
                    mode,
                    batches,
                    skinned_batches,
                    &oit_billboards,
                    oit_particles,
                    oit_strips,
                )
            });
        }
        PlanOitFrameOutputs {
            oit_candidates,
            oit_frame,
        }
    }
}
