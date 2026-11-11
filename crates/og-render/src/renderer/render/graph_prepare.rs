//! The frame's render graph prepared for its pass list (post/graph.rs):
//! its targets' sizes and formats from the frame's decisions, and the
//! tables and textures its post passes bind (the grading table, the
//! stylise palette, canvas and conformed colours).

use super::*;

pub(super) struct PrepareGraphInputs<'a> {
    pub scene: &'a RenderScene,
    pub effects: &'a FrameEffects,
    pub passes: Vec<FramePass>,
    pub taa: bool,
    pub lightmap_size: (u32, u32),
    pub lightdir_on: bool,
    pub gtao: bool,
    pub scene_water: &'a [Water],
    pub distortion: bool,
    pub transmission: bool,
    pub fluids: usize,
    pub reflections: &'a [camera::PlanarReflection],
    pub object_space: bool,
    pub oit_mode: Option<oit::Oit>,
    pub adjusted: bool,
    pub lut_index: Option<usize>,
    pub stylise_key: u64,
}

impl Renderer {
    /// Prepares the render graph for the frame's passes.
    #[allow(clippy::too_many_lines)]
    pub(super) fn prepare_graph(
        &mut self,
        PrepareGraphInputs {
            scene,
            effects,
            passes,
            taa,
            lightmap_size,
            lightdir_on,
            gtao,
            scene_water,
            distortion,
            transmission,
            fluids,
            reflections,
            object_space,
            oit_mode,
            adjusted,
            lut_index,
            stylise_key,
        }: PrepareGraphInputs<'_>,
    ) -> Result<(), RenderError> {
        let st = &scene.stylise;
        let FrameEffects {
            ao,
            gi,
            ssr,
            bent,
            fluids_3d,
            ui_depth,
            prepass,
            ..
        } = *effects;
        // The grading table bound: the adjusted one, the scene's, or none.
        let (lut_key, lut_view) = match (self.post.adjusted_lut().filter(|_| adjusted), lut_index) {
            (Some(view), _) => (Some((0, 0)), view),
            (None, Some(i)) => (
                Some((scene.id(), i)),
                &self.cache.luts[i].as_ref().expect("checked above").0,
            ),
            (None, None) => (None, &self.post.identity_lut),
        };
        // The stylise passes' palette and canvas.
        let palette_view = st
            .quantize
            .and_then(|q| q.palette)
            .and_then(|p| self.cache.palettes.get(p.index()))
            .and_then(Option::as_ref)
            .unwrap_or(&self.default_white);
        let canvas_view = st
            .paint
            .and_then(|p| p.canvas)
            .and_then(|t| self.cache.textures.get(t.index()))
            .map_or(&self.default_white, |g| &g.view);
        self.graph
            .prepare(
                &self.device,
                passes,
                &post::GraphConfig {
                    width: self.width,
                    height: self.height,
                    post: self.history_size(),
                    chain_format: self.post.chain_format,
                    samples: self.samples,
                    taa,
                    lightmap: lightmap_size,
                    lightdir: lightdir_on,
                    gtao,
                    gi: gi.is_some(),
                    ssr: ssr.is_some(),
                    bent,
                    water: !scene_water.is_empty() || fluids_3d || distortion,
                    water_ao: !scene_water.is_empty() && ao.is_some_and(|a| a.direct > 0.0),
                    transmission,
                    fluid_fields: fluids as u32,
                    reflection: [
                        reflections.first().map(|r| r.4),
                        reflections.get(1).map(|r| r.4),
                    ],
                    object_space,
                    ui_depth: ui_depth && prepass,
                    oit: oit_mode.map(|m| m == oit::Oit::List),
                },
                &self.post,
                &self.persistent,
                &self.lights.lightmaps.probe_buffer,
                (lut_key, lut_view),
                (palette_view, canvas_view, &self.conformed.1, stylise_key),
            )
            .map_err(|e| RenderError::Graph(e.to_string()))?;
        Ok(())
    }
}
