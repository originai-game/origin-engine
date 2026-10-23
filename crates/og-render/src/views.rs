//! Secondary cameras (the S8 depth stream, `docs/depth.md` §8): a
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
//! texture as the normalised linear view depth. A viewport view resolves
//! into its rect of the output after the frame's post chain, under the
//! lines and the overlay.
//!
//! Views are drawn before the main pass, lowest [`View::order`] first, so a
//! material of the main camera's (and of a later view's) reads the target
//! as drawn this frame; a view that sees its own target reads the frame
//! before (a monitor in the monitor's room). Transparent meshes, sprites,
//! text, particles, lines and the post effects are the main camera's.
//!
//! Layers: bit `n` of a mask is three.js layer `n`; every object and camera
//! is on layer 0 (mask 1) unless set. A camera draws an object when their
//! masks share a bit (`a & b != 0`), the same test as
//! `og.render.raycast`'s `layers`. Shadow maps take every caster.

use glam::{Mat4, Vec3, Vec4};

use crate::frame::CameraFrame;
use crate::{Camera, RenderTargetHandle};

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

/// What a frame draws of one view.
pub(crate) struct ViewPlan {
    /// The view's index in [`crate::RenderScene::views`] (its GPU slot).
    pub index: usize,
    pub frame: CameraFrame,
    pub position: Vec3,
    pub near: f32,
    pub far: f32,
    /// Pixels drawn (the target's, or the viewport rect's).
    pub size: (u32, u32),
    /// The target's colour and depth texture indices, or the rect in
    /// output pixels (x, y, w, h).
    pub dest: Dest,
    pub layers: u32,
    pub clear: Option<Vec3>,
    /// The mirror's image is written mirrored across x (its uv runs as the
    /// quad's).
    pub flip: bool,
    /// The camera's manual exposure (EV); auto exposure draws at 0.
    pub ev: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Dest {
    Texture {
        color: usize,
        format: crate::TargetFormat,
        depth: Option<usize>,
    },
    Viewport([u32; 4]),
}

/// The frame's views to draw: those due this tick with a live target, in
/// order. `main`: the main camera (a mirror's eye); `out`: the output size.
pub(crate) fn plan(scene: &crate::RenderScene, main: &Camera, out: (u32, u32)) -> Vec<ViewPlan> {
    let mut order: Vec<usize> = (0..scene.views.len()).collect();
    order.sort_by_key(|&i| scene.views[i].order);
    let mut plans = Vec::new();
    for i in order {
        let v = &scene.views[i];
        if v.every > 1 && !scene.tick.is_multiple_of(v.every as u64) {
            continue;
        }
        let (dest, size) = match v.target {
            ViewTarget::Texture(t) => {
                let Some(c) = scene.target_texture(t.color) else {
                    continue;
                };
                let crate::TargetKind::Color(format) = c.kind else {
                    continue;
                };
                let depth = t
                    .depth
                    .filter(|d| scene.target_texture(*d).is_some_and(|d| (d.width, d.height) == (c.width, c.height)))
                    .map(|d| d.index());
                (
                    Dest::Texture {
                        color: t.color.index(),
                        format,
                        depth,
                    },
                    (c.width, c.height),
                )
            }
            ViewTarget::Viewport([x, y, w, h]) => {
                let px = |f: f32, n: u32| ((f.clamp(0.0, 1.0) * n as f32).round() as u32).min(n);
                let (x0, y0) = (px(x, out.0), px(y, out.1));
                let (x1, y1) = (px(x + w, out.0), px(y + h, out.1));
                if x1 <= x0 || y1 <= y0 {
                    continue;
                }
                (
                    Dest::Viewport([x0, y0, x1 - x0, y1 - y0]),
                    (x1 - x0, y1 - y0),
                )
            }
        };
        let aspect = size.0 as f32 / size.1 as f32;
        let ev = match v.camera.exposure {
            crate::Exposure::Manual { ev } => ev,
            crate::Exposure::Auto { .. } => 0.0,
        };
        let (frame, position, near, far, flip) = match v.mirror {
            Some(m) => match mirror_frame(main, &m) {
                Some((f, eye, near)) => (f, eye, near, main.far, true),
                None => continue,
            },
            None => (
                crate::frame::camera_frame(&v.camera, aspect, glam::Vec2::ZERO),
                v.camera.position,
                v.camera.near,
                v.camera.far,
                false,
            ),
        };
        plans.push(ViewPlan {
            index: i,
            frame,
            position,
            near,
            far,
            size,
            dest,
            layers: v.layers,
            clear: v.clear,
            flip,
            ev,
        });
    }
    plans
}

/// The camera a mirror draws with: the eye reflected in the quad's plane,
/// looking through the quad with an off-axis frustum fitted to its corners
/// (Kooima's generalised perspective projection), its near plane on the
/// mirror (nothing behind the glass is drawn). Seen from behind, the quad's
/// +x runs to the image's left: the resolve writes it mirrored, so the
/// target's u runs along +x. `None` when the eye is behind the mirror.
pub(crate) fn mirror_frame(eye: &Camera, m: &MirrorQuad) -> Option<(CameraFrame, Vec3, f32)> {
    let centre = m.transform.w_axis.truncate();
    let right = m.transform.x_axis.truncate();
    let up = m.transform.y_axis.truncate();
    let (sx, sy) = (right.length(), up.length());
    if sx < 1e-6 || sy < 1e-6 {
        return None;
    }
    let (right, up) = (right / sx, up / sy);
    let normal = right.cross(up).normalize_or_zero();
    let d = (eye.position - centre).dot(normal);
    if d <= 1e-4 {
        return None;
    }
    let pe = eye.position - normal * (2.0 * d);
    let hw = m.width * sx * 0.5;
    let hh = m.height * sy * 0.5;
    // From behind: screen right is the quad's -x, the eye looks along +normal.
    let vr = -right;
    let vu = up;
    let vn = -normal;
    let pa = centre + right * hw - up * hh; // lower left as seen from pe
    let pb = centre - right * hw - up * hh; // lower right
    let pc = centre + right * hw + up * hh; // upper left
    let (va, vb, vc) = (pa - pe, pb - pe, pc - pe);
    let dist = -va.dot(vn);
    if dist <= 1e-4 {
        return None;
    }
    let near = dist.max(1e-3);
    let far = eye.far.max(near * 2.0);
    let s = near / dist;
    let l = vr.dot(va) * s;
    let r = vr.dot(vb) * s;
    let b = vu.dot(va) * s;
    let t = vu.dot(vc) * s;
    let proj = off_axis(l, r, b, t, near, far);
    let rot = Mat4::from_cols(
        Vec4::new(vr.x, vu.x, vn.x, 0.0),
        Vec4::new(vr.y, vu.y, vn.y, 0.0),
        Vec4::new(vr.z, vu.z, vn.z, 0.0),
        Vec4::W,
    );
    let view = rot * Mat4::from_translation(-pe);
    let view_proj = proj * view;
    Some((
        CameraFrame {
            view,
            proj,
            view_proj,
            jittered_view_proj: view_proj,
            forward: -vn,
            orthographic: false,
        },
        pe,
        near,
    ))
}

/// A right-handed off-centre perspective projection with depth 0..1 (as
/// glam's `perspective_rh`, for an asymmetric frustum).
pub(crate) fn off_axis(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(2.0 * n / (r - l), 0.0, 0.0, 0.0),
        Vec4::new(0.0, 2.0 * n / (t - b), 0.0, 0.0),
        Vec4::new((r + l) / (r - l), (t + b) / (t - b), f / (n - f), -1.0),
        Vec4::new(0.0, 0.0, n * f / (n - f), 0.0),
    )
}

const VIEWS_WGSL: &str = include_str!("../shaders/views.wgsl");
const VIEWS_COLOR_WGSL: &str = include_str!("../shaders/views_color.wgsl");
const VIEWS_DEPTH_WGSL: &str = include_str!("../shaders/views_depth.wgsl");

/// The colour resolve's source (`fs_color`, `fs_linear`).
pub(crate) fn resolve_color_source() -> String {
    format!("{VIEWS_WGSL}{VIEWS_COLOR_WGSL}")
}

/// The depth resolve's source (`fs_depth`; `ms`: the depth is multisampled).
pub(crate) fn resolve_depth_source(ms: bool) -> String {
    let depth = if ms {
        "texture_depth_multisampled_2d"
    } else {
        "texture_depth_2d"
    };
    format!("alias ViewDepth = {depth};\n{VIEWS_WGSL}{VIEWS_DEPTH_WGSL}")
}

/// `Resolve` in views.wgsl.
#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct ResolveUniform {
    /// x: exposure multiplier, y: tone curve (0 ACES, 1 none, 2 AgX), z: 1
    /// mirrored across x, w: 1 encode sRGB in the shader.
    pub params: [f32; 4],
    /// x, y: the rect's origin in destination pixels, z, w: source size
    /// over rect size.
    pub rect: [f32; 4],
    /// x: near, y: far, z: 1 orthographic, w: source width.
    pub depth: [f32; 4],
}

/// The resolve's entry point for a destination.
pub(crate) fn resolve_entry(format: wgpu::TextureFormat, depth: bool) -> &'static str {
    match (depth, format) {
        (true, _) => "fs_depth",
        (false, wgpu::TextureFormat::Rgba16Float) => "fs_linear",
        _ => "fs_color",
    }
}

/// One view's GPU state, kept across frames while the view exists.
pub(crate) struct ViewSlot {
    pub frame_buffer: wgpu::Buffer,
    pub fx_buffer: wgpu::Buffer,
    /// The view's light clusters: the table and the references.
    pub light_buffers: [wgpu::Buffer; 3],
    pub clusters: crate::clusters::ClusterBuilder,
    pub resolve_buffer: wgpu::Buffer,
    /// (size, samples, depth readable): the attachments below.
    key: ((u32, u32), u32, bool),
    pub hdr: wgpu::TextureView,
    pub msaa: Option<wgpu::TextureView>,
    pub depth: wgpu::TextureView,
}

/// Every view's slot and the resolve pipelines.
#[derive(Default)]
pub(crate) struct Views {
    pub slots: Vec<Option<ViewSlot>>,
    /// By (destination format, entry point, multisampled depth).
    pipelines: std::collections::HashMap<(wgpu::TextureFormat, &'static str, bool), wgpu::RenderPipeline>,
}

impl Views {
    /// The slot of view `index`, its attachments of `size` (made anew when
    /// that changes).
    #[allow(clippy::too_many_arguments)]
    pub fn slot(
        &mut self,
        device: &wgpu::Device,
        tables: crate::clusters::LightTables,
        index: usize,
        size: (u32, u32),
        samples: u32,
        depth_read: bool,
        frame_bytes: u64,
        fx_bytes: u64,
    ) -> &mut ViewSlot {
        if self.slots.len() <= index {
            self.slots.resize_with(index + 1, || None);
        }
        let key = (size, samples, depth_read);
        if self.slots[index].as_ref().is_some_and(|s| s.key != key) {
            let old = self.slots[index].take().expect("checked");
            self.slots[index] = Some(ViewSlot {
                key,
                ..attachments(device, old, key)
            });
        }
        self.slots[index].get_or_insert_with(|| {
            let buffer = |label, size| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(label),
                    size,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            };
            let (hdr, msaa, depth) = textures(device, key);
            ViewSlot {
                frame_buffer: buffer("view frame", frame_bytes),
                fx_buffer: buffer("view fx", fx_bytes),
                light_buffers: tables.buffers(device),
                clusters: Default::default(),
                resolve_buffer: buffer(
                    "view resolve",
                    std::mem::size_of::<ResolveUniform>() as u64,
                ),
                key,
                hdr,
                msaa,
                depth,
            }
        })
    }

    /// Lets the slots of views the scene no longer has go.
    pub fn retain(&mut self, views: usize) {
        self.slots.truncate(views);
    }

    /// Makes the resolve pipeline for a destination `format` and entry.
    pub fn pipeline(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        entry: &'static str,
        ms: bool,
    ) {
        self.pipelines.entry((format, entry, ms)).or_insert_with(|| {
            let module = crate::shader(
                device,
                "view resolve",
                if entry == "fs_depth" {
                    resolve_depth_source(ms)
                } else {
                    resolve_color_source()
                },
            );
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("view resolve"),
                layout: None,
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        });
    }

    /// Writes the views' frame, effect and light-cluster uniforms (the
    /// frame's with each view's camera) and makes their resolve pipelines.
    pub(crate) fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        (tables, samples): (crate::clusters::LightTables, u32),
        kept: &[usize],
        scene: &crate::RenderScene,
        plans: &[ViewPlan],
        uniform: &crate::frame::FrameUniform,
        fx: &crate::fx::FxUniform,
        lights: bool,
        viewport_format: wgpu::TextureFormat,
    ) {
        for p in plans {
            let depth_read = matches!(p.dest, Dest::Texture { depth: Some(_), .. });
            let slot = self.slot(
                device,
                tables,
                p.index,
                p.size,
                samples,
                depth_read,
                std::mem::size_of::<crate::frame::FrameUniform>() as u64,
                std::mem::size_of::<crate::fx::FxUniform>() as u64,
            );
            let frustum = crate::Frustum::from_view_proj(p.frame.view_proj);
            slot.clusters.build_kept(
                &scene.lights,
                kept,
                &crate::clusters::ClusterView {
                    camera: &p.frame,
                    frustum: &frustum,
                    position: p.position,
                    near: p.near,
                    far: p.far,
                    width: p.size.0,
                    height: p.size.1,
                },
                tables.ref_capacity(),
            );
            if lights {
                queue.write_buffer(
                    &slot.light_buffers[1],
                    0,
                    bytemuck::cast_slice(&slot.clusters.table),
                );
                queue.write_buffer(
                    &slot.light_buffers[2],
                    0,
                    bytemuck::cast_slice(&slot.clusters.ref_words()),
                );
            }
            let vp = p.frame.view_proj.to_cols_array_2d();
            let mut u = *uniform;
            u.view_proj = vp;
            u.inv_view_proj = p.frame.view_proj.inverse().to_cols_array_2d();
            u.view_pos = p
                .position
                .extend(p.frame.orthographic as u32 as f32)
                .to_array();
            u.view_forward = p.frame.forward.extend(u.view_forward[3]).to_array();
            u.cluster = slot.clusters.params;
            queue
                .write_buffer(&slot.frame_buffer, 0, bytemuck::bytes_of(&u));
            let mut f = *fx;
            f.ao = [0.0; 4];
            f.view_proj_unjittered = vp;
            f.prev_view_proj = vp;
            let (w, h) = (p.size.0 as f32, p.size.1 as f32);
            f.screen = [w, h, 1.0 / w, 1.0 / h];
            queue
                .write_buffer(&slot.fx_buffer, 0, bytemuck::bytes_of(&f));
        }
        for p in plans {
            match p.dest {
                Dest::Texture { format, depth, .. } => {
                    let f = format.wgpu();
                    self.pipeline(device, f, resolve_entry(f, false), false);
                    if depth.is_some() {
                        let d = wgpu::TextureFormat::Rgba16Float;
                        self.pipeline(device, d, "fs_depth", samples > 1);
                    }
                }
                Dest::Viewport(_) => {
                    let f = viewport_format;
                    self.pipeline(device, f, resolve_entry(f, false), false);
                }
            }
        }
    }

    /// Writes `plan`'s resolve: its HDR (or depth) into `dest` of `format`,
    /// within `rect` (pixels) when given.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        plan: &ViewPlan,
        tonemap: crate::Tonemap,
        dest: &wgpu::TextureView,
        format: wgpu::TextureFormat,
        depth: bool,
        samples: u32,
    ) {
        let entry = resolve_entry(format, depth);
        let ms = depth && samples > 1;
        let slot = self.slots[plan.index].as_ref().expect("made for the plan");
        let rect = match plan.dest {
            Dest::Viewport(r) => r,
            Dest::Texture { .. } => [0, 0, plan.size.0, plan.size.1],
        };
        let u = ResolveUniform {
            params: [
                plan.ev.exp2(),
                match tonemap {
                    crate::Tonemap::Aces => 0.0,
                    crate::Tonemap::None => 1.0,
                    crate::Tonemap::Agx => 2.0,
                },
                plan.flip as u32 as f32,
                (!format.is_srgb() && format != wgpu::TextureFormat::Rgba16Float) as u32 as f32,
            ],
            rect: [rect[0] as f32, rect[1] as f32, 1.0, 1.0],
            depth: [
                plan.near,
                plan.far,
                plan.frame.orthographic as u32 as f32,
                plan.size.0 as f32,
            ],
        };
        queue.write_buffer(&slot.resolve_buffer, 0, bytemuck::bytes_of(&u));
        let pipeline = &self.pipelines[&(format, entry, ms)];
        let layout = pipeline.get_bind_group_layout(0);
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("view resolve"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(if depth {
                        &slot.depth
                    } else {
                        &slot.hdr
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: slot.resolve_buffer.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("view resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: dest,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_viewport(
            rect[0] as f32,
            rect[1] as f32,
            rect[2] as f32,
            rect[3] as f32,
            0.0,
            1.0,
        );
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }

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

fn textures(
    device: &wgpu::Device,
    ((w, h), samples, depth_read): ((u32, u32), u32, bool),
) -> (wgpu::TextureView, Option<wgpu::TextureView>, wgpu::TextureView) {
    let make = |label, format, samples, usage| {
        device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
            .create_view(&Default::default())
    };
    let attach = wgpu::TextureUsages::RENDER_ATTACHMENT;
    let bind = wgpu::TextureUsages::TEXTURE_BINDING;
    let hdr = make("view hdr", crate::HDR_FORMAT, 1, attach | bind);
    let msaa = (samples > 1).then(|| make("view hdr (msaa)", crate::HDR_FORMAT, samples, attach));
    let depth = make(
        "view depth",
        crate::DEPTH_FORMAT,
        samples,
        if depth_read { attach | bind } else { attach },
    );
    (hdr, msaa, depth)
}

fn attachments(device: &wgpu::Device, old: ViewSlot, key: ((u32, u32), u32, bool)) -> ViewSlot {
    let (hdr, msaa, depth) = textures(device, key);
    ViewSlot {
        hdr,
        msaa,
        depth,
        ..old
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
    ) -> Vec<(wgpu::BindGroup, wgpu::BindGroup)> {
        let ibl = self.ibl.views();
        let atlas_view = scene
            .decal_atlas
            .map_or(&self.default_white, |t| &self.cache.textures[t.index()].view);
        plans
            .iter()
            .map(|p| {
                let slot = self.views.slots[p.index].as_ref().expect("written");
                let frame = crate::bind_group(
                    &self.device,
                    "frame (view)",
                    &self.frame_layout,
                    &[
                        slot.frame_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(&self.cascade_map.array_view),
                        wgpu::BindingResource::TextureView(&self.local_map.array_view),
                        wgpu::BindingResource::TextureView(&ibl.irradiance),
                        wgpu::BindingResource::TextureView(&ibl.prefiltered),
                        wgpu::BindingResource::TextureView(&ibl.brdf_lut),
                        wgpu::BindingResource::Sampler(&ibl.sampler),
                        self.light_buffers[0].as_entire_binding(),
                        slot.light_buffers[1].as_entire_binding(),
                        slot.light_buffers[2].as_entire_binding(),
                    ],
                );
                let fx = crate::bind_group(
                    &self.device,
                    "fx (view)",
                    &self.fx_layout,
                    &[
                        slot.fx_buffer.as_entire_binding(),
                        wgpu::BindingResource::TextureView(&self.post.white),
                        wgpu::BindingResource::TextureView(atlas_view),
                        wgpu::BindingResource::Sampler(&self.sprite_sampler),
                    ],
                );
                (frame, fx)
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
        assert!((near - 4.0).abs() < 1e-4);
        let ndc = |p: Vec3| f.view_proj.project_point3(p);
        // The quad's +x end is the image's left (x = -1) before the flip.
        let a = ndc(Vec3::new(1.2, 0.5, 0.0));
        let c = ndc(Vec3::new(-0.8, 1.5, 0.0));
        assert!((a.x + 1.0).abs() < 1e-4 && (a.y + 1.0).abs() < 1e-4, "{a}");
        assert!((c.x - 1.0).abs() < 1e-4 && (c.y - 1.0).abs() < 1e-4, "{c}");
        // On the glass: depth 0; the room in front of it is beyond.
        assert!(a.z.abs() < 1e-4);
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
