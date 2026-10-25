//! Custom post passes (the S8 depth stream, `docs/depth.md` §8): a game's
//! WGSL fragment run over the frame's lit image, in the order of
//! [`crate::RenderScene::custom_post`], after the built-in effects (depth of
//! field, motion blur) and before exposure, bloom and the tone curve (its
//! output is linear HDR radiance, exposed and tone mapped with the frame).
//!
//! ```wgsl
//! fn post(f: PostInput, p: Params) -> vec4<f32>
//! struct PostInput { uv: vec2<f32>, pixel: vec2<f32>, size: vec2<f32>, time: f32 }
//! ```
//!
//! What it reads: `previous(uv)` the pass before it (the lit image for the
//! first), `scene_color(uv)` the lit image before any custom pass (both
//! linear HDR, filtered), `depth(uv)` the linear view depth in world
//! units (1e9 or more: the sky), `normal(uv)` the world normal, and `p`, the
//! parameters the asset declares (at most [`crate::MAX_SHADER_PARAMS`],
//! set with [`crate::RenderScene::set_post_param`]).
//!
//! The cook validates a `type post` asset with naga for every tier. On the
//! device the module and its pipeline are made inside an error scope: one
//! that fails is reported ([`crate::Renderer::post_errors`]) and skipped,
//! and the frame draws as without it.

use crate::hooks::{ParamKind, ShaderParam};
use crate::{MAX_SHADER_PARAMS, RenderScene};
use std::collections::HashMap;

/// The most custom post passes one frame runs.
pub const MAX_CUSTOM_POSTS: usize = 8;

/// A custom post pass: its WGSL `post` function and parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct PostShader {
    pub name: String,
    pub code: String,
    pub params: Vec<ShaderParam>,
}

impl PostShader {
    /// Parameter `name`'s slot and type.
    pub fn param(&self, name: &str) -> Option<(usize, ParamKind)> {
        self.params
            .iter()
            .position(|p| p.name == name)
            .map(|i| (i, self.params[i].kind))
    }

    /// The parameters at their defaults.
    pub fn defaults(&self) -> [[f32; 4]; MAX_SHADER_PARAMS] {
        let mut out = [[0.0; 4]; MAX_SHADER_PARAMS];
        for (slot, p) in out.iter_mut().zip(&self.params) {
            *slot = p.default;
        }
        out
    }

    /// The parameters' count and names, and that `post` is defined.
    pub fn check(&self) -> Result<(), String> {
        let as_material = crate::MaterialShader {
            name: self.name.clone(),
            code: "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> { return c; }"
                .into(),
            params: self.params.clone(),
            mode: Default::default(),
        };
        as_material.check()?;
        if !crate::hooks::defines(&self.code, "post") {
            return Err(
                "a post pass defines `fn post(f: PostInput, p: Params) -> vec4<f32>`".into(),
            );
        }
        Ok(())
    }
}

/// A post pass added to a scene ([`RenderScene::add_post`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PostHandle(pub u32);

impl PostHandle {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl RenderScene {
    /// Adds a custom post pass; it runs while [`RenderScene::custom_post`]
    /// lists it.
    pub fn add_post(&mut self, post: PostShader) -> PostHandle {
        self.post_params.push(post.defaults());
        self.posts.push(post);
        PostHandle(self.posts.len() as u32 - 1)
    }

    /// Sets parameter `name` of a post pass (as many numbers as its type
    /// takes; missing ones are 0). False for a bad handle or name.
    pub fn set_post_param(&mut self, post: PostHandle, name: &str, value: &[f32]) -> bool {
        let Some(shader) = self.posts.get(post.index()) else {
            return false;
        };
        let Some((slot, kind)) = shader.param(name) else {
            return false;
        };
        let mut v = [0.0; 4];
        for (d, s) in v.iter_mut().zip(value.iter().take(kind.count())) {
            *d = *s;
        }
        self.post_params[post.index()][slot] = v;
        true
    }
}

const POST_CUSTOM_WGSL: &str = wgsl!("post_custom.wgsl");

/// A post pass's module source.
pub fn post_module_source(post: &PostShader) -> String {
    let mut s = String::from(POST_CUSTOM_WGSL);
    s.push_str("struct Params {\n");
    if post.params.is_empty() {
        s.push_str("  unused: f32,\n");
    }
    for p in &post.params {
        s.push_str(&format!("  {}: {},\n", p.name, p.kind.wgsl()));
    }
    s.push_str("};\nfn post_params() -> Params {\n  var p: Params;\n");
    for (i, p) in post.params.iter().enumerate() {
        let swizzle = match p.kind {
            ParamKind::Float => ".x",
            ParamKind::Vec2 => ".xy",
            ParamKind::Vec3 => ".xyz",
            ParamKind::Vec4 => "",
        };
        s.push_str(&format!("  p.{} = u.params[{i}]{swizzle};\n", p.name));
    }
    s.push_str("  return p;\n}\n// The pass's code (post_custom.rs).\n");
    s.push_str(&post.code);
    s.push('\n');
    s
}

/// `PostUniform` in post_custom.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    params: [[f32; 4]; MAX_SHADER_PARAMS],
    /// xy: the target's size in pixels, z: seconds, w: unused.
    screen: [f32; 4],
}

/// A compiled pass, or why it is skipped.
enum Compiled {
    Ready {
        pipeline: wgpu::RenderPipeline,
        uniform: wgpu::Buffer,
        /// Its error scope, while the device has not answered (the web).
        pending: Option<std::pin::Pin<Box<dyn std::future::Future<Output = Option<wgpu::Error>>>>>,
    },
    Failed(String),
}

/// The scene's post passes on the device.
#[derive(Default)]
pub(crate) struct CustomPosts {
    /// By (scene id, post index) and the code it was made from.
    compiled: HashMap<(u64, usize), (String, Compiled)>,
    layout: Option<(wgpu::BindGroupLayout, wgpu::PipelineLayout, wgpu::Sampler)>,
}

impl CustomPosts {
    fn layout(&mut self, device: &wgpu::Device) {
        if self.layout.is_some() {
            return;
        }
        let tex = |binding, filterable| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let group = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("custom post"),
            entries: &[
                tex(0, true),
                tex(1, true),
                tex(2, false),
                tex(3, false),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("custom post"),
            bind_group_layouts: &[Some(&group)],
            immediate_size: 0,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("custom post"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        self.layout = Some((group, pipeline, sampler));
    }

    /// Compiles the scene's listed passes not made yet (inside an error
    /// scope) and answers the passes that run this frame, in order: those
    /// listed, live and compiled. A pass that failed is left out.
    pub fn prepare(&mut self, device: &wgpu::Device, scene: &RenderScene) -> Vec<usize> {
        self.layout(device);
        let mut run = Vec::new();
        for h in scene.custom_post.iter().take(MAX_CUSTOM_POSTS) {
            let i = h.index();
            let Some(post) = scene.posts.get(i) else {
                continue;
            };
            let key = (scene.id(), i);
            if self
                .compiled
                .get(&key)
                .is_none_or(|(code, _)| *code != post.code)
            {
                let compiled = self.compile(device, post);
                self.compiled.insert(key, (post.code.clone(), compiled));
            }
            let entry = &mut self.compiled.get_mut(&key).expect("made above").1;
            if let Compiled::Ready { pending, .. } = entry
                && let Some(f) = pending
            {
                let waker = std::task::Waker::noop();
                let mut cx = std::task::Context::from_waker(waker);
                if let std::task::Poll::Ready(e) = f.as_mut().poll(&mut cx) {
                    *pending = None;
                    if let Some(e) = e {
                        *entry = Compiled::Failed(e.to_string());
                    }
                }
            }
            if let Compiled::Ready { .. } = entry {
                run.push(i);
            }
        }
        run
    }

    fn compile(&self, device: &wgpu::Device, post: &PostShader) -> Compiled {
        if let Err(e) = post.check() {
            return Compiled::Failed(e);
        }
        let (_, layout, _) = self.layout.as_ref().expect("made in prepare");
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&post.name),
            source: wgpu::ShaderSource::Wgsl(post_module_source(post).into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&post.name),
            layout: Some(layout),
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
                entry_point: Some("fs_custom_post"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: crate::HDR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let mut pending = Box::pin(scope.pop());
        // Native devices answer at once; the web's on a later frame.
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        let pending = match pending.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(Some(e)) => return Compiled::Failed(e.to_string()),
            std::task::Poll::Ready(None) => None,
            std::task::Poll::Pending => {
                Some(pending as std::pin::Pin<Box<dyn std::future::Future<Output = _>>>)
            }
        };
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("custom post"),
            size: std::mem::size_of::<PostUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Compiled::Ready {
            pipeline,
            uniform,
            pending,
        }
    }

    /// The passes that failed, with the device's message.
    pub fn errors(&self, scene: u64) -> Vec<(PostHandle, String)> {
        let mut out: Vec<_> = self
            .compiled
            .iter()
            .filter(|((s, _), _)| *s == scene)
            .filter_map(|((_, i), (_, c))| match c {
                Compiled::Failed(e) => Some((PostHandle(*i as u32), e.clone())),
                _ => None,
            })
            .collect();
        og_core::sort::sort_by_key(&mut out, |(h, _)| *h);
        out
    }

    /// Records post pass `post` (the scene's index) from `read` (with the
    /// lit image `scene_color`, `lindepth` and `gbuffer`) into `write`.
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut crate::Encoder,
        scene: &RenderScene,
        post: usize,
        views: [&wgpu::TextureView; 5],
        size: (u32, u32),
    ) -> u32 {
        let Some((
            _,
            Compiled::Ready {
                pipeline, uniform, ..
            },
        )) = self.compiled.get(&(scene.id(), post))
        else {
            return 0;
        };
        let (group_layout, _, sampler) = self.layout.as_ref().expect("made in prepare");
        let u = PostUniform {
            params: scene.post_params[post],
            screen: [
                size.0 as f32,
                size.1 as f32,
                scene.tick as f32 / scene.tick_rate.max(1e-6),
                0.0,
            ],
        };
        queue.write_buffer(uniform, 0, bytemuck::bytes_of(&u));
        let [read, scene_color, depth, gbuffer, write] = views;
        let group = crate::bind_group(
            device,
            "custom post",
            group_layout,
            &[
                wgpu::BindingResource::TextureView(read),
                wgpu::BindingResource::TextureView(scene_color),
                wgpu::BindingResource::TextureView(depth),
                wgpu::BindingResource::TextureView(gbuffer),
                wgpu::BindingResource::Sampler(sampler),
                uniform.as_entire_binding(),
            ],
        );
        crate::post::fullscreen_pass(
            encoder,
            &scene.posts[post].name,
            write,
            wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            pipeline,
            &group,
        );
        1
    }
}

/// A CRT look as a post pass: the frame bent onto a curved screen (black
/// outside it), scanlines, an RGB shadow mask and a vignette (validated
/// with the engine's modules, `shaders.rs`; og-tests' `post-crt`).
pub fn example_crt() -> PostShader {
    let param = |name: &str, kind, default| ShaderParam {
        name: name.into(),
        kind,
        default,
    };
    PostShader {
        name: "crt".into(),
        code: CRT_WGSL.into(),
        params: vec![
            param("curve", ParamKind::Float, [0.12, 0.0, 0.0, 0.0]),
            param("scanlines", ParamKind::Float, [0.35, 0.0, 0.0, 0.0]),
            param("lines", ParamKind::Float, [240.0, 0.0, 0.0, 0.0]),
            param("vignette", ParamKind::Float, [0.6, 0.0, 0.0, 0.0]),
        ],
    }
}

const CRT_WGSL: &str = "fn post(f: PostInput, p: Params) -> vec4<f32> {
  // The screen bulges: uv pushed outwards with its distance from the centre.
  let c = f.uv * 2.0 - 1.0;
  let bent = c * (1.0 + p.curve * dot(c, c));
  let uv = bent * 0.5 + 0.5;
  if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0))) {
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
  }
  var color = previous(uv).rgb;
  // Scanlines across the bent image.
  let line = 0.5 + 0.5 * cos(uv.y * p.lines * 6.2831853);
  color *= 1.0 - p.scanlines * line;
  // The shadow mask: each pixel column favours red, green or blue.
  let column = u32(f.pixel.x) % 3u;
  var mask = vec3<f32>(0.85);
  mask[column] = 1.15;
  color *= mask;
  // Darker towards the rounded corners.
  let edge = uv * (1.0 - uv);
  color *= pow(clamp(edge.x * edge.y * 16.0, 0.0, 1.0), p.vignette * 0.5);
  return vec4<f32>(color, 1.0);
}
";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_params_set_by_name() {
        let mut scene = RenderScene::new();
        let h = scene.add_post(example_crt());
        assert_eq!(scene.post_params[0][0][0], 0.12);
        assert!(scene.set_post_param(h, "curve", &[0.3, 9.0]));
        assert_eq!(scene.post_params[0][0], [0.3, 0.0, 0.0, 0.0]);
        assert!(!scene.set_post_param(h, "nope", &[1.0]));
        assert!(!scene.set_post_param(PostHandle(4), "curve", &[1.0]));
        let mut bad = example_crt();
        bad.code = "fn other() {}".into();
        assert!(bad.check().is_err());
    }
}
