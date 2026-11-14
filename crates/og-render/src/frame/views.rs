//! Secondary cameras (the S8 depth stream, `docs/design.md` §10.8): a
//! [`View`] draws the scene from its own camera into a render target
//! (`targets.rs`) or into a rect of the frame (split-screen), with a
//! three.js layer mask, an order, a clear colour and a rate; a mirror is a
//! view whose camera is the main one reflected in a quad.
//!
//! A view draws the opaque meshes (static and skinned, every material:
//! hooks, full shaders and physical ones through the standard pipelines, as
//! the water's planar reflection does), the terrain and the sky, lit by the
//! frame's lights (its own light clusters) and shadowed by the frame's
//! shadow maps, into an HDR colour and depth of its size. A resolve pass
//! then writes the target: [`crate::TargetFormat::Rgba8`] exposed and tone
//! mapped with the camera's exposure and the frame's tone curve,
//! [`crate::TargetFormat::Rgba16Float`] as linear radiance, the depth
//! texture as the normalised linear view depth. The depth comes from a
//! prepass of the view's opaque meshes into a linear-depth colour target
//! (as the main prepass's `lindepth`), never from its depth buffer, so a
//! multisampled view reads the same single depth as any other. A viewport view resolves
//! into its rect of the output after the frame's post chain, under the
//! lines and the overlay.
//!
//! Views are drawn before the main pass, lowest [`View::order`] first, so a
//! material of the main camera's (and of a later view's) reads the target
//! as drawn this frame; a view that sees its own target reads the frame
//! before (a monitor in the monitor's room). It then draws what the main
//! pass draws over its opaque scene, on its layers from its eye:
//! transparent meshes, particles, trails, ribbons and beams (facing its
//! eye), lines and points, world sprites and text, its lit sprite layers
//! through its own 2D light map and its 2D fluids from its own fields
//! (`lib.rs`'s view pass), its soft particles against its own depth
//! prepass, its glowing sprites through its own glow chain
//! (`views_glow.wgsl`); the other post effects are the main camera's.
//!
//! Layers: bit `n` of a mask is three.js layer `n`; every object and camera
//! is on layer 0 (mask 1) unless set. A camera draws an object when their
//! masks share a bit (`a & b != 0`), the same test as
//! `og.render.raycast`'s `layers`. Shadow maps take every caster.

use glam::{Mat4, Vec3, Vec4};

use crate::frame::CameraFrame;
use crate::targets::TargetGpu;
use crate::{Camera, RenderTargetHandle};

mod glow;
mod plan;
mod resolve;
mod slots;
mod write;

pub(crate) use glow::glow_source;
pub(crate) use plan::{Dest, ViewPlan, plan};
pub(crate) use resolve::{resolve_color_source, resolve_depth_source};
pub(crate) use slots::DepthPrepass;
pub(crate) use write::camera_gpu;

use glow::{GlowAdd, ViewGlow, glow_add, view_glow};
#[cfg(test)]
use plan::mirror_frame;
use resolve::{ResolveUniform, resolve_entry};
use slots::{ViewSlot, hdr_texture};

/// A layer mask from a per-index list: past its end, layer 0 (mask 1).
pub(crate) fn layer_of(masks: &[u32], i: usize) -> u32 {
    masks.get(i).copied().unwrap_or(1)
}

/// Where a view draws.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewTarget {
    /// Into a render target ([`crate::RenderScene::add_render_target`]):
    /// its colour, and its depth texture when it has one.
    Texture(RenderTargetHandle),
    /// Into a rect of the frame, in fractions of its width and height from
    /// the top left (x, y, width, height): split-screen, picture-in-picture.
    Viewport([f32; 4]),
}

/// A mirror's quad: the plane the view reflects the main camera in, and the
/// rect of it the target covers (its texture coordinates 0..1 across
/// `width` along the transform's +x and `height` along +y, facing +z).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MirrorQuad {
    pub transform: Mat4,
    pub width: f32,
    pub height: f32,
}

/// A secondary camera. See the module documentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    /// The camera (for a mirror, the eye reflected in it: the main camera).
    pub camera: Camera,
    pub target: ViewTarget,
    /// Three.js layer mask: it draws the objects whose masks share a bit.
    pub layers: u32,
    /// Drawn in ascending order (ties in list order), all before the frame.
    pub order: i32,
    /// The colour cleared to (linear rgb); `None`: the scene's background.
    pub clear: Option<Vec3>,
    /// Drawn once every this many ticks (tick % every == 0); 0 and 1 every
    /// frame. A target keeps its last image between.
    pub every: u32,
    /// A mirror: the camera reflected in the quad, the image fitted to it.
    pub mirror: Option<MirrorQuad>,
}

impl Default for View {
    fn default() -> Self {
        View {
            camera: Camera::default(),
            target: ViewTarget::Viewport([0.0, 0.0, 1.0, 1.0]),
            layers: 1,
            order: 0,
            clear: None,
            every: 1,
            mirror: None,
        }
    }
}

/// Every view's slot and the resolve pipelines.
#[derive(Default)]
pub(crate) struct Views {
    pub slots: Vec<Option<ViewSlot>>,
    /// Set by `Renderer::restore_state`: every view draws on the next
    /// frame, so a target drawn every few ticks shows the restored world,
    /// not the image of the run that was cut.
    pub stale: bool,
    /// By (destination format, entry point).
    pipelines: std::collections::HashMap<(wgpu::TextureFormat, &'static str), wgpu::RenderPipeline>,
    glow_add: Option<GlowAdd>,
}

impl Views {
    /// Begins `plan`'s lit pass: its HDR (multisampled when the frame is)
    /// and depth cleared to `clear`.
    pub fn begin<'e>(
        &self,
        encoder: &'e mut wgpu::CommandEncoder,
        plan: &ViewPlan,
        clear: wgpu::Color,
    ) -> wgpu::RenderPass<'e> {
        let slot = self.slots[plan.index].as_ref().expect("made for the plan");
        let (view, resolve_target) = match &slot.msaa {
            Some(m) => (m, Some(&slot.hdr)),
            None => (&slot.hdr, None),
        };
        let clear = match plan.clear {
            Some(c) => wgpu::Color {
                r: c.x as f64,
                g: c.y as f64,
                b: c.z as f64,
                a: 1.0,
            },
            None => clear,
        };
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("view"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: if resolve_target.is_some() {
                        wgpu::StoreOp::Discard
                    } else {
                        wgpu::StoreOp::Store
                    },
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &slot.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        })
    }
}

impl crate::Renderer {
    /// The format a viewport view resolves into (the output, or its sRGB
    /// view where the overlay blends).
    pub(crate) fn viewport_format(&self, target: &crate::RenderTarget) -> wgpu::TextureFormat {
        if !self.output_format.is_srgb() && self.has_linear_view(target) {
            self.output_format.add_srgb_suffix()
        } else {
            self.output_format
        }
    }

    /// Each view's frame group (the frame's, with its camera and light
    /// clusters) and effect group (its decals, no AO).
    pub(crate) fn view_groups(
        &self,
        scene: &crate::RenderScene,
        plans: &[ViewPlan],
    ) -> Vec<(wgpu::BindGroup, wgpu::BindGroup, Option<wgpu::BindGroup>)> {
        let ibl = self.lights.ibl.views();
        let atlas_view = scene.effects.decal_atlas.map_or(&self.default_white, |t| {
            &self.cache.textures[t.index()].view
        });
        plans
            .iter()
            .map(|p| {
                let slot = self.views.slots[p.index].as_ref().expect("written");
                let frame = crate::bind_group(
                    &self.device,
                    "frame (view)",
                    &self.frame_layout,
                    &self.frame_entries(super::FrameGroupParts {
                        uniform: slot.frame_buffer.as_entire_binding(),
                        clusters: [&slot.light_buffers[1], &slot.light_buffers[2]],
                        cubes: [&ibl.irradiance, &ibl.prefiltered],
                        ssr: self.ssr.off_view(),
                    }),
                );
                let fx = crate::fx::fx_group(
                    &self.device,
                    "fx (view)",
                    &self.fx_layout,
                    &[
                        slot.fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(&self.post.white),
                        wgpu::BindingResource::TextureView(atlas_view),
                        wgpu::BindingResource::Sampler(&self.trilinear_sampler),
                        // No planar reflection in a view (the main camera's).
                        wgpu::BindingResource::TextureView(&self.post.white),
                    ][..if self.planar_slot() { 5 } else { 4 }],
                    &self.lights.decal_buffers,
                );
                let lean = self.fx_lean_group(&slot.fx_buffer, &self.post.white, atlas_view);
                (frame, fx, lean)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The mirror's frustum passes exactly through the quad's corners: each
    /// projects to the image corner it maps to (u along +x after the flip).
    #[test]
    fn mirror_frustum_fits_the_quad() {
        let eye = Camera {
            position: Vec3::new(0.7, 1.3, 4.0),
            target: Vec3::new(0.0, 1.0, 0.0),
            ..Camera::default()
        };
        let quad = MirrorQuad {
            transform: Mat4::from_translation(Vec3::new(0.2, 1.0, 0.0)),
            width: 2.0,
            height: 1.0,
        };
        let (f, pe, near) = mirror_frame(&eye, &quad).expect("in front");
        assert!((pe - Vec3::new(0.7, 1.3, -4.0)).length() < 1e-5);
        assert!((near - 4.004).abs() < 1e-4);
        let ndc = |p: Vec3| f.view_proj.project_point3(p);
        // The quad's +x end is the image's left (x = -1) before the flip.
        let a = ndc(Vec3::new(1.2, 0.5, 0.0));
        let c = ndc(Vec3::new(-0.8, 1.5, 0.0));
        assert!((a.x + 1.0).abs() < 1e-4 && (a.y + 1.0).abs() < 1e-4, "{a}");
        assert!((c.x - 1.0).abs() < 1e-4 && (c.y - 1.0).abs() < 1e-4, "{c}");
        // The glass itself is in front of the near plane (clipped); the
        // room in front of the mirror is beyond it.
        assert!(a.z < 0.0, "{a}");
        assert!(ndc(Vec3::new(0.2, 1.0, 2.0)).z > 0.0);
        // Behind the mirror, no reflection.
        let behind = Camera {
            position: Vec3::new(0.0, 1.0, -3.0),
            ..eye
        };
        assert!(mirror_frame(&behind, &quad).is_none());
    }

    #[test]
    fn layers_default_to_layer_zero() {
        assert_eq!(layer_of(&[4], 0), 4);
        assert_eq!(layer_of(&[4], 3), 1);
    }
}
