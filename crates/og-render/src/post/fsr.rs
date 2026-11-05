//! AMD FidelityFX Super Resolution 3.1's temporal upscaler (FSR 3.1.6,
//! MIT: third_party/fidelityfx), ported to WGSL compute passes
//! (`shaders/fsr_*.wgsl`) on core WebGPU: no fp16, no subgroups, no float
//! atomics, no device-wide coherence, so it runs on every host the same.
//! It takes the place of the TAA resolve ([`crate::Upscaler::Fsr3`]):
//! the frame graph's `taa` pass dispatches its passes in turn and writes
//! the same history target TAA writes, and `taa-sharpen` runs its RCAS.
//! Everything else FSR keeps between frames (its own history with locks,
//! the accumulation, the luma history, the frame info) lives here, sized
//! for the output so a render scale that moves (dynamic resolution) keeps
//! it.

use super::*;
use glam::{Mat4, Vec2};

/// The passes in dispatch order, with their entry points and modules.
const SHADERS: [(&str, &str); 6] = [
    ("fsr prepare inputs", wgsl!("fsr_prepare_inputs.wgsl")),
    ("fsr luma pyramid", wgsl!("fsr_luma_pyramid.wgsl")),
    (
        "fsr shading change pyramid",
        wgsl!("fsr_shading_change_pyramid.wgsl"),
    ),
    ("fsr shading change", wgsl!("fsr_shading_change.wgsl")),
    (
        "fsr prepare reactivity",
        wgsl!("fsr_prepare_reactivity.wgsl"),
    ),
    ("fsr accumulate", wgsl!("fsr_accumulate.wgsl")),
];
const RCAS: (&str, &str) = ("fsr rcas", wgsl!("fsr_rcas.wgsl"));
const COMMON: &str = wgsl!("fsr_common.wgsl");

/// A pass's module: fullscreen.wgsl (for `sample_exact`), fsr_common.wgsl
/// and its body.
fn source(body: &str) -> String {
    format!("{}{COMMON}{body}", crate::post::FULLSCREEN_WGSL)
}

/// The modules as the renderer compiles them (shaders.rs validates them).
pub(crate) fn sources() -> Vec<(&'static str, String)> {
    SHADERS
        .iter()
        .chain([&RCAS])
        .map(|(label, body)| (*label, source(body)))
        .collect()
}

/// `struct Fsr` in fsr_common.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FsrUniform {
    inv_view_proj: [[f32; 4]; 4],
    cur_view_proj: [[f32; 4]; 4],
    prev_view_proj: [[f32; 4]; 4],
    render: [i32; 4],
    upscale: [i32; 4],
    max_size: [i32; 4],
    device_to_view: [f32; 4],
    jitter: [f32; 4],
    frame: [f32; 4],
    tuning: [f32; 4],
    extra: [f32; 4],
    misc: [f32; 4],
}

/// The most reactivity the OIT revealage gives a pixel: AMD's guidance
/// keeps reactive values near 1 for what has no history at all, and
/// glass, smoke and particles keep some (`fsr_prepare_reactivity.wgsl`).
const REACTIVE_CAP: f32 = 0.9;

/// What one binding is.
#[derive(Clone, Copy)]
enum B {
    Uniform,
    Tex(bool),
    Storage(wgpu::TextureFormat),
    Rw,
    Ro,
    Sampler,
}

fn layout(device: &wgpu::Device, label: &str, bindings: &[B]) -> wgpu::BindGroupLayout {
    let entries: Vec<_> = bindings
        .iter()
        .enumerate()
        .map(|(i, b)| wgpu::BindGroupLayoutEntry {
            binding: i as u32,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: match *b {
                B::Uniform => UNIFORM,
                B::Tex(filterable) => float_texture(filterable),
                B::Storage(format) => wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                B::Rw | B::Ro => wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: matches!(b, B::Ro),
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                B::Sampler => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            },
            count: None,
        })
        .collect();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &entries,
    })
}

use wgpu::TextureFormat as F;
const PREPARE_INPUTS: &[B] = &[
    B::Uniform,
    B::Tex(false),
    B::Tex(false),
    B::Tex(false),
    B::Tex(false),
    B::Storage(F::R32Float),
    B::Storage(F::Rg32Float),
    B::Storage(F::R32Float),
    B::Storage(F::R32Float),
    B::Rw,
];
const LUMA_PYRAMID: &[B] = &[
    B::Uniform,
    B::Tex(false),
    B::Tex(false),
    B::Storage(F::Rgba16Float),
    B::Rw,
    B::Rw,
];
const SHADING_CHANGE_PYRAMID: &[B] = &[
    B::Uniform,
    B::Tex(false),
    B::Tex(false),
    B::Tex(false),
    B::Ro,
    B::Storage(F::Rgba16Float),
    B::Storage(F::Rgba16Float),
    B::Storage(F::Rgba16Float),
];
const SHADING_CHANGE: &[B] = &[
    B::Uniform,
    B::Tex(true),
    B::Sampler,
    B::Storage(F::Rgba8Unorm),
];
const PREPARE_REACTIVITY: &[B] = &[
    B::Uniform,
    B::Tex(false),
    B::Tex(false),
    B::Ro,
    B::Tex(false),
    B::Tex(true),
    B::Tex(true),
    B::Tex(false),
    B::Ro,
    B::Sampler,
    B::Storage(F::Rgba8Unorm),
    B::Storage(F::Rgba8Unorm),
    B::Rw,
    B::Tex(true),
    B::Storage(F::Rgba16Float),
    B::Storage(F::R32Float),
];
const ACCUMULATE: &[B] = &[
    B::Uniform,
    B::Tex(true),
    B::Tex(false),
    B::Tex(false),
    B::Tex(true),
    B::Tex(true),
    B::Tex(false),
    B::Ro,
    B::Rw,
    B::Sampler,
    B::Tex(true),
    B::Storage(F::Rgba16Float),
    B::Storage(F::Rgba16Float),
];
const RCAS_BINDINGS: &[B] = &[B::Uniform, B::Tex(false), B::Ro, B::Storage(F::Rgba16Float)];

/// A compute pass: its layout and its pipeline (made at first use).
struct Pass {
    layout: wgpu::BindGroupLayout,
    pipeline: Deferred<wgpu::ComputePipeline>,
}

fn pass(
    device: &wgpu::Device,
    (label, body): (&'static str, &'static str),
    bindings: &[B],
    entry: &'static str,
) -> Pass {
    let layout = layout(device, label, bindings);
    let (device, group) = (device.clone(), layout.clone());
    Pass {
        layout,
        pipeline: Deferred::new(move || {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(label),
                source: wgpu::ShaderSource::Wgsl(source(body).into()),
            });
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(&group)],
                immediate_size: 0,
            });
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(label),
                layout: Some(&pl),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(&device).as_ref(),
            })
        }),
    }
}

/// A storage texture of FSR's own.
fn texture(
    device: &wgpu::Device,
    label: &str,
    (w, h): (u32, u32),
    format: F,
    mips: u32,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

fn view(t: &wgpu::Texture) -> wgpu::TextureView {
    t.create_view(&Default::default())
}

fn pair(device: &wgpu::Device, label: &str, size: (u32, u32), format: F) -> [wgpu::TextureView; 2] {
    [0, 1].map(|_| view(&texture(device, label, size, format, 1)))
}

fn buffer(device: &wgpu::Device, label: &str, size: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size.max(16),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// What FSR keeps between frames, for an output of `upscale` (render sizes
/// up to it).
struct State {
    upscale: (u32, u32),
    dilated_depth: wgpu::TextureView,
    dilated_motion: wgpu::TextureView,
    farthest_depth: wgpu::TextureView,
    farthest_depth_mip1: wgpu::TextureView,
    /// This frame's and the previous frame's luma (by frame parity).
    luma: [wgpu::TextureView; 2],
    spd_mips: wgpu::TextureView,
    spd_levels: [wgpu::TextureView; 3],
    shading_change: wgpu::TextureView,
    dilated_reactive: wgpu::TextureView,
    accumulation: [wgpu::TextureView; 2],
    luma_history: [wgpu::TextureView; 2],
    luma_instability: wgpu::TextureView,
    /// FSR's own history: colour and lock.
    history: [wgpu::TextureView; 2],
    reconstructed_depth: wgpu::Buffer,
    new_locks: wgpu::Buffer,
    luma_partials: wgpu::Buffer,
    frame_info: wgpu::Buffer,
}

impl State {
    fn new(device: &wgpu::Device, upscale: (u32, u32)) -> State {
        let half = ((upscale.0 / 2).max(1), (upscale.1 / 2).max(1));
        let spd = texture(device, "fsr spd mips", half, F::Rgba16Float, 3);
        let level = |m| {
            spd.create_view(&wgpu::TextureViewDescriptor {
                base_mip_level: m,
                mip_level_count: Some(1),
                ..Default::default()
            })
        };
        let pixels = u64::from(upscale.0) * u64::from(upscale.1);
        let groups = u64::from(upscale.0.div_ceil(32)) * u64::from(upscale.1.div_ceil(32));
        State {
            upscale,
            dilated_depth: view(&texture(
                device,
                "fsr dilated depth",
                upscale,
                F::R32Float,
                1,
            )),
            dilated_motion: view(&texture(
                device,
                "fsr dilated motion",
                upscale,
                F::Rg32Float,
                1,
            )),
            farthest_depth: view(&texture(
                device,
                "fsr farthest depth",
                upscale,
                F::R32Float,
                1,
            )),
            farthest_depth_mip1: view(&texture(
                device,
                "fsr farthest depth mip 1",
                half,
                F::Rgba16Float,
                1,
            )),
            luma: pair(device, "fsr luma", upscale, F::R32Float),
            spd_levels: [level(0), level(1), level(2)],
            spd_mips: view(&spd),
            shading_change: view(&texture(
                device,
                "fsr shading change",
                half,
                F::Rgba8Unorm,
                1,
            )),
            dilated_reactive: view(&texture(
                device,
                "fsr dilated reactive",
                upscale,
                F::Rgba8Unorm,
                1,
            )),
            accumulation: pair(device, "fsr accumulation", upscale, F::Rgba8Unorm),
            luma_history: pair(device, "fsr luma history", upscale, F::Rgba16Float),
            luma_instability: view(&texture(
                device,
                "fsr luma instability",
                upscale,
                F::R32Float,
                1,
            )),
            history: pair(device, "fsr history", upscale, F::Rgba16Float),
            reconstructed_depth: buffer(device, "fsr reconstructed depth", pixels * 4),
            new_locks: buffer(device, "fsr new locks", pixels * 4),
            luma_partials: buffer(device, "fsr luma partials", groups * 8),
            frame_info: buffer(device, "fsr frame info", 16),
        }
    }

    fn bytes(&self) -> u64 {
        let views = [
            &self.dilated_depth,
            &self.dilated_motion,
            &self.farthest_depth,
            &self.farthest_depth_mip1,
            &self.shading_change,
            &self.dilated_reactive,
            &self.luma_instability,
            &self.spd_mips,
        ];
        views
            .into_iter()
            .chain(&self.luma)
            .chain(&self.accumulation)
            .chain(&self.luma_history)
            .chain(&self.history)
            .map(crate::renderer::view_bytes)
            .sum::<u64>()
            + self.reconstructed_depth.size()
            + self.new_locks.size()
            + self.luma_partials.size()
            + self.frame_info.size()
    }
}

/// What a frame hands FSR (`FsrPasses::prepare`).
pub(crate) struct FsrFrame<'a> {
    pub render: (u32, u32),
    pub upscale: (u32, u32),
    pub jitter: Vec2,
    pub phases: u64,
    /// The current frame's clip (jittered) to world, the unjittered
    /// view-projections now and at the history's frame, and the projection.
    pub inv_view_proj: Mat4,
    pub view_proj: Mat4,
    pub prev_view_proj: Mat4,
    pub proj: Mat4,
    /// No history to blend (a cut, a resize, the first frame).
    pub reset: bool,
    pub still: bool,
    /// The scene's tick and ticks per second: the exposure FSR weighs by
    /// adapts over the ticks since its last frame (at least one a frame),
    /// never the wall clock, so captures repeat.
    pub tick: u64,
    pub tick_rate: f32,
    /// RCAS's strength, 0 to 1 (`RenderDesc::taa_sharpen`).
    pub sharpness: f32,
    pub color: &'a wgpu::TextureView,
    pub depth: &'a wgpu::TextureView,
    /// The prepass's gbuffer and view depth, when it drew.
    pub prepass: Option<(&'a wgpu::TextureView, &'a wgpu::TextureView)>,
    pub reveal: &'a wgpu::TextureView,
    /// The engine's history this frame writes and the one it read.
    pub output: &'a wgpu::TextureView,
    pub previous_output: &'a wgpu::TextureView,
    /// RCAS's target (`hdr.sharp`), with the sharpening pass.
    pub sharpened: Option<&'a wgpu::TextureView>,
}

/// The frame's bind groups and dispatch sizes.
struct Recorded {
    groups: [wgpu::BindGroup; 7],
    rcas: Option<wgpu::BindGroup>,
    render: (u32, u32),
    upscale: (u32, u32),
    luma_groups: (u32, u32),
    reconstructed_depth: wgpu::Buffer,
}

/// FSR 3.1's passes and its state.
/// A frame's jitter, render size and output size.
type Previous = (Vec2, (u32, u32), (u32, u32));

pub(crate) struct FsrPasses {
    device: wgpu::Device,
    uniform: wgpu::Buffer,
    sampler: wgpu::Sampler,
    passes: [Pass; 7],
    rcas: Pass,
    state: Option<State>,
    /// Frames since the state was made: its ping-pong parity.
    index: u64,
    /// The previous frame's jitter, render size and output size.
    previous: Option<Previous>,
    /// Frames since the last reset (FSR's FrameIndex).
    frame_index: u64,
    last_tick: Option<u64>,
    recorded: Option<Recorded>,
}

impl FsrPasses {
    pub fn new(device: &wgpu::Device) -> FsrPasses {
        let [prepare, luma, scp, sc, reactivity, accumulate] = SHADERS;
        FsrPasses {
            device: device.clone(),
            uniform: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("fsr"),
                size: std::mem::size_of::<FsrUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            sampler: device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("fsr linear clamp"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::MipmapFilterMode::Linear,
                ..Default::default()
            }),
            passes: [
                pass(device, prepare, PREPARE_INPUTS, "cs"),
                pass(device, luma, LUMA_PYRAMID, "cs_reduce"),
                pass(device, luma, LUMA_PYRAMID, "cs_finish"),
                pass(device, scp, SHADING_CHANGE_PYRAMID, "cs"),
                pass(device, sc, SHADING_CHANGE, "cs"),
                pass(device, reactivity, PREPARE_REACTIVITY, "cs"),
                pass(device, accumulate, ACCUMULATE, "cs"),
            ],
            rcas: pass(device, RCAS, RCAS_BINDINGS, "cs"),
            state: None,
            index: 0,
            previous: None,
            frame_index: 0,
            last_tick: None,
            recorded: None,
        }
    }

    /// Whether this frame resolves with FSR ([`FsrPasses::prepare`] ran
    /// since the last [`FsrPasses::release`]).
    pub fn active(&self) -> bool {
        self.recorded.is_some()
    }

    /// GPU bytes FSR holds besides the engine's history.
    pub fn bytes(&self) -> u64 {
        self.state.as_ref().map_or(0, State::bytes)
    }

    /// Lets go of the state (a frame drawn without FSR): the next FSR
    /// frame starts from no history.
    pub fn release(&mut self) {
        self.state = None;
        self.previous = None;
        self.recorded = None;
        self.last_tick = None;
    }

    /// Makes the state for the frame's output, writes its uniform and its
    /// bind groups; [`FsrPasses::encode`] records it.
    pub fn prepare(&mut self, queue: &wgpu::Queue, f: &FsrFrame) {
        let mut reset = f.reset;
        if self.state.as_ref().is_none_or(|s| s.upscale != f.upscale) {
            self.state = Some(State::new(&self.device, f.upscale));
            self.index = 0;
            self.previous = None;
            reset = true;
        }
        if reset {
            self.frame_index = 0;
        }
        let ticks = self
            .last_tick
            .map_or(1, |t| f.tick.saturating_sub(t).max(1));
        let dt = (ticks as f32 / f.tick_rate.max(1e-3)).clamp(0.0, 1.0);
        self.last_tick = Some(f.tick);
        let (prev_jitter, prev_render, prev_upscale) =
            self.previous.unwrap_or((f.jitter, f.render, f.upscale));
        // The view depth of a device depth from the inverse projection's
        // rows 2 and 3 (fsr_common.wgsl `view_depth`).
        let ip = f.proj.inverse().transpose();
        let device_to_view = [ip.z_axis.z, ip.z_axis.w, ip.w_axis.z, ip.w_axis.w];
        let luma_groups = (f.render.0.div_ceil(32), f.render.1.div_ceil(32));
        // RCAS's sharpness in stops (FsrRcasCon of 2 - 2 s).
        let sharp = (-(2.0 - 2.0 * f.sharpness.clamp(0.0, 1.0))).exp2();
        let i = |(x, y): (u32, u32)| [x as i32, y as i32];
        let u = FsrUniform {
            inv_view_proj: f.inv_view_proj.to_cols_array_2d(),
            cur_view_proj: f.view_proj.to_cols_array_2d(),
            prev_view_proj: f.prev_view_proj.to_cols_array_2d(),
            render: [i(f.render), i(prev_render)].concat().try_into().unwrap(),
            upscale: [i(f.upscale), i(prev_upscale)].concat().try_into().unwrap(),
            max_size: [i(f.upscale), i(f.upscale)].concat().try_into().unwrap(),
            device_to_view,
            jitter: [f.jitter.x, f.jitter.y, prev_jitter.x, prev_jitter.y],
            frame: [dt, 1.0, self.frame_index as f32, f.phases as f32],
            tuning: [1.0, 1.0, 1.0, 1.0 / 3.0],
            extra: [
                -1.0 / 3.0,
                f32::from(u8::from(f.still)),
                f32::from(u8::from(f.prepass.is_some())),
                sharp,
            ],
            misc: [
                (luma_groups.0 * luma_groups.1) as f32,
                REACTIVE_CAP,
                0.0,
                0.0,
            ],
        };
        queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&u));

        let s = self.state.as_ref().expect("made above");
        let (cur, prev) = ((self.index % 2) as usize, (1 - self.index % 2) as usize);
        let (velocity, view_depth) = f.prepass.unwrap_or((f.depth, f.depth));
        use wgpu::BindingResource as R;
        let t = R::TextureView;
        fn buf(b: &wgpu::Buffer) -> wgpu::BindingResource<'_> {
            b.as_entire_binding()
        }
        let sampler = R::Sampler(&self.sampler);
        let group = |p: &Pass, res: Vec<R>| {
            let entries: Vec<_> = std::iter::once(self.uniform.as_entire_binding())
                .chain(res)
                .enumerate()
                .map(|(i, resource)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource,
                })
                .collect();
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fsr"),
                layout: &p.layout,
                entries: &entries,
            })
        };
        let [p0, p1, p2, p3, p4, p5, p6] = &self.passes;
        let luma_pyramid = || {
            vec![
                t(&s.luma[cur]),
                t(&s.farthest_depth),
                t(&s.farthest_depth_mip1),
                buf(&s.luma_partials),
                buf(&s.frame_info),
            ]
        };
        let groups = [
            group(
                p0,
                vec![
                    t(f.color),
                    t(f.depth),
                    t(velocity),
                    t(view_depth),
                    t(&s.dilated_depth),
                    t(&s.dilated_motion),
                    t(&s.farthest_depth),
                    t(&s.luma[cur]),
                    buf(&s.reconstructed_depth),
                ],
            ),
            group(p1, luma_pyramid()),
            group(p2, luma_pyramid()),
            group(
                p3,
                vec![
                    t(&s.dilated_motion),
                    t(&s.luma[cur]),
                    t(&s.luma[prev]),
                    buf(&s.frame_info),
                    t(&s.spd_levels[0]),
                    t(&s.spd_levels[1]),
                    t(&s.spd_levels[2]),
                ],
            ),
            group(
                p4,
                vec![t(&s.spd_mips), sampler.clone(), t(&s.shading_change)],
            ),
            group(
                p5,
                vec![
                    t(&s.dilated_motion),
                    t(&s.dilated_depth),
                    buf(&s.reconstructed_depth),
                    t(f.reveal),
                    t(&s.shading_change),
                    t(&s.accumulation[prev]),
                    t(&s.luma[cur]),
                    buf(&s.frame_info),
                    sampler.clone(),
                    t(&s.dilated_reactive),
                    t(&s.accumulation[cur]),
                    buf(&s.new_locks),
                    t(&s.luma_history[prev]),
                    t(&s.luma_history[cur]),
                    t(&s.luma_instability),
                ],
            ),
            group(
                p6,
                vec![
                    t(&s.dilated_reactive),
                    t(&s.dilated_motion),
                    t(&s.history[prev]),
                    t(&s.farthest_depth_mip1),
                    t(f.color),
                    t(&s.luma_instability),
                    buf(&s.frame_info),
                    buf(&s.new_locks),
                    sampler.clone(),
                    t(f.previous_output),
                    t(&s.history[cur]),
                    t(f.output),
                ],
            ),
        ];
        let rcas = f
            .sharpened
            .map(|target| group(&self.rcas, vec![t(f.output), buf(&s.frame_info), t(target)]));
        self.recorded = Some(Recorded {
            groups,
            rcas,
            render: f.render,
            upscale: f.upscale,
            luma_groups,
            reconstructed_depth: s.reconstructed_depth.clone(),
        });
        self.previous = Some((f.jitter, f.render, f.upscale));
        self.index += 1;
        self.frame_index += 1;
    }

    /// Records the upscale (the frame graph's `taa` pass); its dispatches.
    pub fn encode(&self, encoder: &mut crate::Encoder) -> u32 {
        let Some(r) = &self.recorded else {
            return 0;
        };
        encoder.clear_buffer(&r.reconstructed_depth, 0, None);
        let (rw, rh) = r.render;
        let (uw, uh) = r.upscale;
        let half = ((rw / 2).max(1), (rh / 2).max(1));
        let dispatches = [
            (rw.div_ceil(8), rh.div_ceil(8)),
            r.luma_groups,
            (1, 1),
            (rw.div_ceil(32), rh.div_ceil(32)),
            (half.0.div_ceil(8), half.1.div_ceil(8)),
            (rw.div_ceil(8), rh.div_ceil(8)),
            (uw.div_ceil(8), uh.div_ceil(8)),
        ];
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("fsr"),
            timestamp_writes: None,
        });
        for ((p, g), (x, y)) in self.passes.iter().zip(&r.groups).zip(dispatches) {
            pass.set_pipeline(p.pipeline.get());
            pass.set_bind_group(0, g, &[]);
            pass.dispatch_workgroups(x, y, 1);
        }
        dispatches.len() as u32
    }

    /// Records RCAS (the frame graph's `taa-sharpen` pass).
    pub fn encode_rcas(&self, encoder: &mut crate::Encoder) -> u32 {
        let Some((r, g)) = self
            .recorded
            .as_ref()
            .and_then(|r| r.rcas.as_ref().map(|g| (r, g)))
        else {
            return 0;
        };
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("fsr rcas"),
            timestamp_writes: None,
        });
        pass.set_pipeline(self.rcas.pipeline.get());
        pass.set_bind_group(0, g, &[]);
        pass.dispatch_workgroups(r.upscale.0.div_ceil(8), r.upscale.1.div_ceil(8), 1);
        1
    }
}

/// FSR 3.1's jitter sequence length for a render of `render_width` shown
/// at `display_width` (ffxFsr3UpscalerGetJitterPhaseCount).
#[cfg(test)]
fn amd_phase_count(render_width: u32, display_width: u32) -> i32 {
    (8.0 * (display_width as f32 / render_width as f32).powi(2)) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The engine's jitter cycle is FSR's at the render scales the presets
    /// use (8 at native, 32 at half), so the jitter walks the same samples.
    #[test]
    fn the_jitter_cycle_matches_amd_phase_count() {
        for (render, out) in [(1920, 1920), (960, 1920), (1280, 1920)] {
            let ours = crate::frame::jitter_phases((render, render), (out, out));
            assert_eq!(
                ours as i32,
                amd_phase_count(render, out).clamp(8, 32),
                "{render}/{out}"
            );
        }
    }

    /// The uniform matches `struct Fsr` (fsr_common.wgsl): three matrices
    /// and nine vec4 rows.
    #[test]
    fn the_uniform_is_the_shaders_size() {
        assert_eq!(std::mem::size_of::<FsrUniform>(), 3 * 64 + 9 * 16);
    }
}
