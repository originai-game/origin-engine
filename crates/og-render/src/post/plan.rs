//! The render graph of a frame: its passes ([`FramePass`]), its targets
//! (`plan/targets.rs`) and, family by family, what each pass reads and
//! writes. The families are those that bind and encode the passes
//! (`post/graph/`), with the scene's own beside them.

use super::*;

mod ao;
mod bloom;
mod chain;
mod config;
mod cx;
mod exposure;
mod names;
mod pass;
mod scene;
mod screen;
mod targets;
#[cfg(test)]
mod tests;
pub(crate) use config::*;
use cx::PlanCx;
pub(crate) use names::*;
pub(crate) use pass::*;

/// The render graph of a frame made of `passes` (in recording order).
pub(crate) fn build_graph(passes: &[FramePass], cfg: &GraphConfig) -> RenderGraph {
    let cx = PlanCx::new(passes, cfg);
    let mut g = RenderGraph::new();
    targets::declare(&mut g, &cx);
    for &p in passes {
        let family = match p {
            FramePass::CrowdCull
            | FramePass::CullCounts
            | FramePass::Wind
            | FramePass::WaterFields
            | FramePass::Trails
            | FramePass::Aurora
            | FramePass::CloudShadow
            | FramePass::Visual
            | FramePass::Surfaces
            | FramePass::TerrainCorners
            | FramePass::Grass
            | FramePass::Scatter
            | FramePass::Cascade(_)
            | FramePass::LocalFace(_)
            | FramePass::Shelter
            | FramePass::Prepass
            | FramePass::Particles
            | FramePass::Light2d
            | FramePass::Metaballs
            | FramePass::Fluid3d
            | FramePass::LiveSky
            | FramePass::Clouds
            | FramePass::GiProbes
            | FramePass::Probes
            | FramePass::Reflection(_)
            | FramePass::View(_)
            | FramePass::Volumetric
            | FramePass::Main
            | FramePass::Ssr
            | FramePass::Glow
            | FramePass::ViewportViews
            | FramePass::UiLayers
            | FramePass::Overlay
            | FramePass::HdrOutput => scene::declare,
            FramePass::Ao | FramePass::AoBlur | FramePass::AoUpsample | FramePass::Contact => {
                ao::declare
            }
            FramePass::DepthBands
            | FramePass::FogOfWar
            | FramePass::Outline
            | FramePass::OutlineSeed
            | FramePass::OutlineJump(_)
            | FramePass::OutlineWide
            | FramePass::Taa
            | FramePass::TaaSharpen
            | FramePass::DofCoc
            | FramePass::DofBlur
            | FramePass::DofComposite
            | FramePass::MotionBlur
            | FramePass::ScreenBlur
            | FramePass::CustomPost(_)
            | FramePass::GodRaysMask
            | FramePass::GodRaysBlur => screen::declare,
            FramePass::Meter | FramePass::Reduce(_) | FramePass::Adapt => exposure::declare,
            FramePass::BloomPrefilter
            | FramePass::BloomDown(_)
            | FramePass::BloomUp(_)
            | FramePass::FlareOcclusion
            | FramePass::Flare
            | FramePass::FlareBlurX
            | FramePass::FlareBlurY => bloom::declare,
            FramePass::Tonemap
            | FramePass::Paint
            | FramePass::Watercolor
            | FramePass::Lines
            | FramePass::Lens
            | FramePass::Quantize
            | FramePass::Fxaa
            | FramePass::SmaaEdges
            | FramePass::SmaaWeights
            | FramePass::SmaaBlend
            | FramePass::SmaaResolve
            | FramePass::Upscale => chain::declare,
        };
        family(&mut g, &cx, p);
    }
    g
}
