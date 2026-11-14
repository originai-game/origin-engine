//! Stylise's object pass ([`StyliseSpace::Object`](crate::StyliseSpace)):
//! after the depth-normal prepass, the opaque mesh instances the prepass
//! drew are drawn again against its depth into the `object-space` target
//! (`Rgba32Uint`): each covered pixel's position in its instance's own
//! space (metres along the instance's axes) and its normal there, so the
//! paint and lines passes lay the canvas, the hatching and the halftone on
//! the object and a moving object carries them (`stylise_object.wgsl`).
//!
//! The vertices are placed by the lit shader's `lit_place`, as the prepass
//! places them, and the pass tests `LessEqual` against the prepass depth
//! with a few units of bias towards the camera and writes no depth, so
//! only the surface the prepass kept passes. What it leaves at 0 keeps
//! the world pattern: terrain and scattered copies (not instances), and
//! the moved vertices of a material shader's vertex offset (drawn without
//! it, they miss the prepass depth). Alpha-cut holes are not cut here: a
//! pixel seen through one takes the cut surface's coordinates.
//!
//! The module (the lit shader's source with the built-in hooks and
//! `stylise_object.wgsl`) and its pipelines are made the first frame a
//! look lays a pattern in object space; nothing is built or drawn before.

use crate::post::OBJECT_FORMAT;
use crate::{Faces, skin};
use std::collections::HashMap;

pub(crate) const STYLISE_OBJECT_WGSL: &str = wgsl!("stylise_object.wgsl");

/// The object pass's depth bias towards the camera (units of the depth
/// format's precision at the triangle), so a surface whose vertices land a
/// rounding away from the prepass's still passes `LessEqual`.
const BIAS: wgpu::DepthBiasState = wgpu::DepthBiasState {
    constant: -4,
    slope_scale: -0.5,
    clamp: 0.0,
};

/// The object pass's module source (`shaders.rs` validates it).
pub(crate) fn object_source() -> String {
    format!(
        "{}{STYLISE_OBJECT_WGSL}",
        crate::lit_source_with(&crate::hooks::default_hooks(), false)
    )
}

/// The object pass's module and its pipelines by (side, skinned).
pub(crate) struct ObjectPass {
    module: wgpu::ShaderModule,
    pipelines: HashMap<(Faces, bool), wgpu::RenderPipeline>,
}

impl ObjectPass {
    /// `off`: the lit features the renderer is made without
    /// (pipeline_features.rs), out of its module.
    pub fn new(device: &wgpu::Device, off: crate::pipeline_features::Features) -> ObjectPass {
        ObjectPass {
            module: crate::shader(device, "stylise object", off.leave_out(object_source())),
            pipelines: HashMap::new(),
        }
    }

    /// Makes the pipeline for (side, skinned) unless it is made; `layouts`
    /// are the lit pipelines' (plain, skinned), whose groups the prepass
    /// binds.
    pub fn ensure(
        &mut self,
        device: &wgpu::Device,
        layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
        faces: Faces,
        skinned: bool,
    ) {
        if self.pipelines.contains_key(&(faces, skinned)) {
            return;
        }
        let plain = [Some(crate::VERTEX_LAYOUT), Some(crate::INSTANCE_LAYOUT)];
        let skin = [
            Some(crate::VERTEX_LAYOUT),
            Some(skin::SKINNED_INSTANCE_LAYOUT),
            Some(skin::SKIN_VERTEX_LAYOUT),
        ];
        let p = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("stylise object"),
            layout: Some(if skinned { layouts.1 } else { layouts.0 }),
            vertex: wgpu::VertexState {
                module: &self.module,
                entry_point: Some(if skinned {
                    "vs_object_skinned"
                } else {
                    "vs_object"
                }),
                compilation_options: Default::default(),
                buffers: if skinned { &skin } else { &plain },
            },
            primitive: faces.primitive(faces.cull()),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: BIAS,
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &self.module,
                entry_point: Some("fs_object"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: OBJECT_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: crate::pipeline_cache::of(device).as_ref(),
        });
        self.pipelines.insert((faces, skinned), p);
    }

    /// The pipeline [`ObjectPass::ensure`] made for (side, skinned).
    pub fn pipeline(&self, faces: Faces, skinned: bool) -> &wgpu::RenderPipeline {
        &self.pipelines[&(faces, skinned)]
    }
}
