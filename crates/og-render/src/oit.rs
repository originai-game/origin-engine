//! Order-independent transparency (`docs/design.md` §2.6): the blended
//! surfaces of the main camera (transparent meshes of every material
//! family, skinned ones, alpha billboards and their drop shadows, alpha
//! particles and alpha strips) drawn in any order into targets whose
//! result does not depend on that order, then resolved over the opaque
//! scene before the additive effects, the sprites and the post chain.
//!
//! - [`Transparency::Weighted`], the default: weighted blended OIT
//!   (McGuire and Bavoil, JCGT 2(2), 2013) with one exact layer (hybrid
//!   transparency: Maule, Comba, Torchelsen and Bastos, I3D 2013). A
//!   marking pass leaves each pixel's nearest depth of a fragment at least
//!   1/8 opaque in the arena (an `atomicMax` of the inverted depth bits,
//!   the same in any order); the OIT pass sends that fragment to a front
//!   target and every other one to the weighted tail, where it adds its
//!   premultiplied colour times a weight of its alpha and depth into a
//!   half-float accumulation target and multiplies its transmittance into
//!   a revealage target. The resolve composites the front exactly over the
//!   tail's weighted mean, so a billboard before glass keeps its colour;
//!   only the layers behind the nearest read as their mean. Chosen over
//!   moment-based OIT (Münstermann et al. 2018: two shading passes and
//!   32-bit float moments, whose blending is optional on the web, still
//!   approximate in front) and over k > 1 layers (k-buffers, stochastic
//!   layered alpha: ordered pixel access or 64-bit atomics, which WebGPU
//!   lacks). Weighted alone where the device writes no fragment storage.
//! - [`Transparency::LinkedList`], the high tier: every fragment is also
//!   kept in its pixel's own nodes in one storage arena (group 0 binding
//!   12, [`Arena`]), up to [`RenderDesc::transparency_layers`] a pixel;
//!   the resolve sorts a pixel's nodes by depth (equal depths by a stable
//!   key of the draw and primitive) and composites them exactly. A pixel
//!   with more fragments than nodes is resolved whole from the weighted
//!   targets, which every fragment still reaches, and is counted
//!   ([`FrameStats::transparency_fallback_pixels`]): no fragment is ever
//!   dropped, and which pixels fall back does not depend on draw order,
//!   since no pixel's nodes are shared.
//!
//! The arena is sized against the device's actual limits (the smaller of
//! its largest storage binding and buffer, never a limit of its own
//! asking): the nodes per pixel are what was asked, as far as that fits
//! the scene's pixels; where not one node a pixel fits the frame draws
//! weighted. Fragment stores need not wait for the depth test, so the
//! resolve tests each node against the opaque depth of each MSAA sample,
//! and keeps it only for the samples it covered. The revealage is TAA's
//! reactive mask (`taa.wgsl`): transparent surfaces have no depth or
//! motion of their own, so the history is trusted less under them.
//!
//! What stays as it was: additive particles, strips and billboards (added
//! after the resolve, so lightning is never dimmed by weighting), the
//! distortion particles and debris, lines, 2D sprites and text in their
//! layer order, the UI, and the secondary cameras, reflections and probe
//! faces, which sort their blended surfaces back to front. Transparent
//! surfaces still write no depth or velocity, so depth of field, motion
//! blur and contact shadows see the opaque scene behind them.

use crate::HdrPipeline;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

/// How the main camera's blended surfaces are composited
/// ([`RenderDesc::transparency`]); [`RenderInfo::transparency`] is what the
/// renderer resolved it to.
///
/// [`RenderDesc::transparency`]: crate::RenderDesc::transparency
/// [`RenderInfo::transparency`]: crate::RenderInfo::transparency
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Transparency {
    /// Weighted blended order-independent transparency: one pass, any
    /// order, approximate where layers of similar alpha overlap.
    #[default]
    Weighted,
    /// Per-pixel fragment lists, sorted and composited exactly up to
    /// [`crate::RenderDesc::transparency_layers`] fragments a pixel; a pixel
    /// with more is resolved weighted. Weighted where the device cannot
    /// write storage from fragment shaders.
    LinkedList,
}

/// The nodes a pixel keeps by default in the linked-list mode.
pub const DEFAULT_TRANSPARENCY_LAYERS: u32 = 4;
/// The most nodes a pixel sorts (`oit_list.wgsl` `OIT_SORT_LIMIT`).
pub const MAX_TRANSPARENCY_LAYERS: u32 = 32;

/// What a pass draws its blended surfaces into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Oit {
    /// The weighted targets only (a device without fragment storage).
    Weighted,
    /// The weighted mode's hybrid: the weighted targets, the pixel's
    /// nearest layer kept exact in the front target, chosen by the depth
    /// [`Oit::Mark`] left in the arena (`oit_arena.wgsl` `oit_hybrid`).
    Front,
    /// The front layer's marking pass before [`Oit::Front`]: no colour
    /// targets, each fragment's depth into its pixel's arena word.
    Mark,
    /// The weighted targets and the linked lists' arena.
    List,
}

impl Oit {
    /// Whether its passes bind the arena (the frame group's binding 14).
    pub fn arena(self) -> bool {
        self != Oit::Weighted
    }
}

/// The accumulation target: premultiplied colour and alpha times weight.
pub(crate) const ACCUM_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
/// The revealage target: the product of the tail fragments'
/// transmittance (r) and of the front layer's (g).
pub(crate) const REVEAL_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
/// The front layer: its premultiplied colour and alpha, added (one
/// fragment, or the coplanar ones that share the front).
pub(crate) const FRONT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// The participants' shared fragment code.
pub(crate) const OIT_WGSL: &str = wgsl!("oit.wgsl");
/// The arena binding and the node store (linked-list entry points).
pub(crate) const ARENA_WGSL: &str = wgsl!("oit_arena.wgsl");
const RESOLVE_WGSL: &str = wgsl!("oit_resolve.wgsl");
const LIST_WGSL: &str = wgsl!("oit_list.wgsl");

/// The fragment entry of `fs` drawing into `mode`'s targets: `fs_oit`,
/// `fs_oit_front`, `fs_oit_mark`, `fs_oit_list` (every participant module
/// defines them beside `fs`).
pub(crate) fn fragment_entry(fs: &str, mode: Oit) -> String {
    match mode {
        Oit::Weighted => format!("{fs}_oit"),
        Oit::Front => format!("{fs}_oit_front"),
        Oit::Mark => format!("{fs}_oit_mark"),
        Oit::List => format!("{fs}_oit_list"),
    }
}

/// The three targets an OIT pass draws: the tail's colour and alpha
/// added, the revealages multiplied by each fragment's transmittance, the
/// front layer added.
pub(crate) fn targets() -> [Option<wgpu::ColorTargetState>; 3] {
    let add = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::One,
        dst_factor: wgpu::BlendFactor::One,
        operation: wgpu::BlendOperation::Add,
    };
    let reveal = wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::OneMinusSrc,
        operation: wgpu::BlendOperation::Add,
    };
    [
        Some(wgpu::ColorTargetState {
            format: ACCUM_FORMAT,
            blend: Some(wgpu::BlendState {
                color: add,
                alpha: add,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        }),
        Some(wgpu::ColorTargetState {
            format: REVEAL_FORMAT,
            blend: Some(wgpu::BlendState {
                color: reveal,
                alpha: reveal,
            }),
            write_mask: wgpu::ColorWrites::RED | wgpu::ColorWrites::GREEN,
        }),
        Some(wgpu::ColorTargetState {
            format: FRONT_FORMAT,
            blend: Some(wgpu::BlendState {
                color: add,
                alpha: add,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        }),
    ]
}

/// The OIT variant of a blended HDR pipeline `p` (its module, vertex
/// state, culling and depth test): the fragment entry of `mode`, the OIT
/// targets, no depth write and no alpha to coverage. `p.layout` must be
/// the arena's ([`OitState::layout`]) for the modes that bind it
/// ([`Oit::arena`]); [`Oit::Mark`] draws no colour.
pub(crate) fn pipeline(
    device: &wgpu::Device,
    constants: &[(&str, f64)],
    p: HdrPipeline,
    mode: Oit,
) -> wgpu::RenderPipeline {
    // Only the linked lists read the vertex stages' OIT key (oit.wgsl).
    let mut constants = constants.to_vec();
    if mode == Oit::List {
        constants.push(("OIT_KEY", 1.0));
    }
    let options = wgpu::PipelineCompilationOptions {
        constants: &constants,
        ..Default::default()
    };
    let fs = fragment_entry(p.fs, mode);
    let targets = targets();
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{} (oit)", p.label)),
        layout: Some(p.layout),
        vertex: wgpu::VertexState {
            module: p.module,
            entry_point: Some(p.vs),
            compilation_options: options.clone(),
            buffers: p.buffers,
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: p.cull,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: crate::DEPTH_FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(p.depth_compare),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: p.samples,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: p.module,
            entry_point: Some(&fs),
            compilation_options: options,
            targets: if mode == Oit::Mark { &[] } else { &targets },
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// The arena's entry in the linked-list frame group: binding 14, after
/// the frame group's own (`frame.wgsl`, bindings 0 to 13).
pub(crate) fn arena_layout_entry() -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 14,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

/// Whether the device can keep the linked lists: storage written from
/// fragment shaders, and room for the arena beside the seven storage
/// buffers a skinned lit pipeline binds (the light tables, the light
/// volumes, the LTC table, the decals and the palette).
pub(crate) fn supports_lists(flags: wgpu::DownlevelFlags, limits: &wgpu::Limits) -> bool {
    flags.contains(wgpu::DownlevelFlags::FRAGMENT_WRITABLE_STORAGE)
        && limits.max_storage_buffers_per_shader_stage >= 8
}

/// The bytes the device lets one arena take: the smaller of its largest
/// storage binding and its largest buffer.
pub(crate) fn arena_budget(limits: &wgpu::Limits) -> u64 {
    limits
        .max_storage_buffer_binding_size
        .min(limits.max_buffer_size)
}

/// `oit_arena.wgsl`'s layout: four header words, a fragment count per
/// pixel, then each pixel's nodes of seven words.
const HEADER_BYTES: u64 = 16;
const COUNT_BYTES: u64 = 4;
const NODE_BYTES: u64 = 28;

/// An arena's shape: the pass's size and the nodes each pixel keeps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArenaSize {
    pub width: u32,
    pub height: u32,
    pub nodes_per_pixel: u32,
    pub bytes: u64,
}

impl ArenaSize {
    /// `requested` nodes a pixel (0: [`DEFAULT_TRANSPARENCY_LAYERS`], at
    /// most [`MAX_TRANSPARENCY_LAYERS`]) as far as `max_bytes` holds them
    /// for every pixel of `width` x `height`; `None` where not one does
    /// (the frame draws weighted). Every pixel gets the same number of its
    /// own, so no order of fragments lets one pixel take another's.
    pub fn new(width: u32, height: u32, requested: u32, max_bytes: u64) -> Option<Self> {
        let pixels = u64::from(width) * u64::from(height);
        if pixels == 0 {
            return None;
        }
        // Word indices are u32 in the shaders.
        let max_bytes = max_bytes.min(u64::from(u32::MAX) * 4);
        let per_pixel = max_bytes.checked_sub(HEADER_BYTES)? / pixels;
        let fit = per_pixel.checked_sub(COUNT_BYTES)? / NODE_BYTES;
        let requested = match requested {
            0 => DEFAULT_TRANSPARENCY_LAYERS,
            n => n,
        }
        .min(MAX_TRANSPARENCY_LAYERS);
        let nodes_per_pixel = u64::from(requested).min(fit) as u32;
        (nodes_per_pixel > 0).then(|| ArenaSize {
            width,
            height,
            nodes_per_pixel,
            bytes: HEADER_BYTES + pixels * (COUNT_BYTES + NODE_BYTES * u64::from(nodes_per_pixel)),
        })
    }

    /// The weighted mode's front layer (`oit_arena.wgsl` `oit_front_mark`):
    /// the header and one word a pixel, in the place of the counts; `None`
    /// where `max_bytes` cannot hold it.
    pub fn front(width: u32, height: u32, max_bytes: u64) -> Option<Self> {
        let pixels = u64::from(width) * u64::from(height);
        let bytes = HEADER_BYTES + pixels * COUNT_BYTES;
        (pixels > 0 && bytes <= max_bytes.min(u64::from(u32::MAX) * 4)).then_some(ArenaSize {
            width,
            height,
            nodes_per_pixel: 0,
            bytes,
        })
    }

    /// The header words the shaders read: width, height, nodes per pixel,
    /// and the overflowing pixels' count (cleared each frame).
    pub fn header(self) -> [u32; 4] {
        [self.width, self.height, self.nodes_per_pixel, 0]
    }

    /// What each frame clears before its OIT pass (byte offset, size): the
    /// overflow count and every pixel's fragment count; the nodes are
    /// overwritten as they are taken.
    pub fn cleared(self) -> (u64, u64) {
        let pixels = u64::from(self.width) * u64::from(self.height);
        (HEADER_BYTES - 4, 4 + pixels * COUNT_BYTES)
    }
}

/// The linked lists' arena: one storage buffer of [`ArenaSize`], and the
/// readbacks of its overflow count (a frame or two after the frame). The
/// readbacks are behind cells: the main pass, which holds the renderer
/// shared, queues the copy.
pub(crate) struct Arena {
    pub buffer: wgpu::Buffer,
    pub size: ArenaSize,
    readbacks: RefCell<Vec<Readback>>,
    /// The newest overflow count read back, with its frame.
    latest: Cell<Option<(u64, u32)>>,
}

struct Readback {
    buffer: wgpu::Buffer,
    mapped: Arc<Mutex<Option<bool>>>,
    /// The frame copied into it, until it is read.
    frame: Option<u64>,
    /// Copied into this frame, not yet mapped.
    unmapped: bool,
}

impl Arena {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: ArenaSize) -> Arena {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("oit arena"),
            size: size.bytes,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&size.header()));
        let readbacks = (0..3)
            .map(|_| Readback {
                buffer: device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("oit overflow readback"),
                    size: 4,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }),
                mapped: Arc::new(Mutex::new(None)),
                frame: None,
                unmapped: false,
            })
            .collect();
        Arena {
            buffer,
            size,
            readbacks: RefCell::new(readbacks),
            latest: Cell::new(None),
        }
    }

    /// Clears the counts before a frame's OIT pass.
    pub fn clear(&self, encoder: &mut crate::Encoder) {
        let (offset, size) = self.size.cleared();
        encoder.clear_buffer(&self.buffer, offset, Some(size));
    }

    /// After the OIT pass: copies the overflow count of `frame` into a free
    /// readback (none free: this frame's is not read).
    pub fn copy_count(&self, encoder: &mut crate::Encoder, frame: u64) {
        let mut readbacks = self.readbacks.borrow_mut();
        if let Some(r) = readbacks.iter_mut().find(|r| r.frame.is_none()) {
            encoder.copy_buffer_to_buffer(&self.buffer, HEADER_BYTES - 4, &r.buffer, 0, 4);
            r.frame = Some(frame);
            r.unmapped = true;
        }
    }

    /// After the frame's submit: maps what `copy_count` copied.
    pub fn after_submit(&self) {
        for r in self.readbacks.borrow_mut().iter_mut() {
            if std::mem::take(&mut r.unmapped) {
                let mapped = r.mapped.clone();
                r.buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |result| {
                        *mapped.lock().unwrap_or_else(|e| e.into_inner()) = Some(result.is_ok());
                    });
            }
        }
    }

    /// Reads the readbacks that have landed; answers the newest count read
    /// back, with its frame.
    pub fn poll(&self) -> Option<(u64, u32)> {
        for r in self.readbacks.borrow_mut().iter_mut() {
            let Some(frame) = r.frame.filter(|_| !r.unmapped) else {
                continue;
            };
            let done = r.mapped.lock().unwrap_or_else(|e| e.into_inner()).take();
            match done {
                None => continue,
                Some(true) => {
                    let count = r
                        .buffer
                        .slice(..)
                        .get_mapped_range()
                        .ok()
                        .map(|data| u32::from_le_bytes([data[0], data[1], data[2], data[3]]));
                    r.buffer.unmap();
                    if let Some(count) = count
                        && self.latest.get().is_none_or(|(f, _)| f < frame)
                    {
                        self.latest.set(Some((frame, count)));
                    }
                }
                Some(false) => {}
            }
            r.frame = None;
        }
        self.latest.get()
    }
}

/// The linked lists' arena as the last frame left it
/// ([`crate::Renderer::read_transparency_counts`]): its shape, the pixels
/// it counted as overflowing, and each pixel's fragment count (rows top to
/// bottom), every fragment the pixel's blended surfaces stored or tried to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransparencyCounts {
    pub width: u32,
    pub height: u32,
    /// The nodes a pixel keeps; a pixel whose count is above it was
    /// resolved weighted.
    pub layers: u32,
    pub overflow_pixels: u32,
    pub counts: Vec<u32>,
}

impl Arena {
    /// Reads the header and the counts back (blocking).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn read_counts(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<TransparencyCounts, String> {
        let pixels = u64::from(self.size.width) * u64::from(self.size.height);
        let bytes = HEADER_BYTES + pixels * COUNT_BYTES;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("oit counts readback"),
            size: bytes,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.buffer, 0, &buffer, 0, bytes);
        queue.submit(Some(encoder.finish()));
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| format!("reading the oit counts back: {e}"))?;
        let words: Vec<u32> = {
            let data = slice
                .get_mapped_range()
                .map_err(|e| format!("reading the oit counts back: {e:?}"))?;
            bytemuck::cast_slice(&data).to_vec()
        };
        buffer.unmap();
        Ok(TransparencyCounts {
            width: words[0],
            height: words[1],
            layers: words[2],
            overflow_pixels: words[3],
            counts: words[4..].to_vec(),
        })
    }
}

/// What an OIT pipeline draws (with its pass mode and samples, its key in
/// [`OitState::pipelines`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Source {
    /// A blended mesh of a lit module: which, its side, skinned, effects.
    Lit(LitModule, crate::Side, bool, bool),
    /// A lightmapped blended mesh: its family, side and effects.
    Lightmap(crate::lightmap::Family, crate::Side, bool),
    /// Alpha billboards and their drop shadows (sprite.wgsl `fs`).
    Sprite,
    /// Billboards' text (sprite.wgsl `fs_msdf`).
    Msdf,
    /// Alpha strips from their vertices (ribbon.wgsl `vs`).
    Ribbon,
    /// Alpha strips from their points (ribbon.wgsl `vs_strip`).
    Strip,
    /// Alpha particle billboards (particles.wgsl `fs`).
    Particles,
}

/// The lit module a blended mesh draws with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum LitModule {
    Standard,
    Array,
    Physical,
    /// A material shader's standard module.
    Shader(usize),
    /// A material shader's physical module.
    PhysicalShader(usize),
}

/// The order-independent passes' state: the resolved mode, the frame
/// group layout with the arena (linked lists), the arena, the resolve
/// and the participants' OIT pipelines, made at first use.
pub(crate) struct OitState {
    /// What the renderer resolved [`crate::RenderDesc::transparency`] to.
    pub mode: Transparency,
    /// The nodes a pixel asks for ([`crate::RenderDesc::transparency_layers`]).
    pub layers: u32,
    /// The device's arena budget ([`arena_budget`]).
    pub budget: u64,
    /// The frame group with binding 14, the arena (the linked lists, and
    /// the weighted mode's front layer), where the device writes storage
    /// from fragment shaders.
    pub frame_list_layout: Option<wgpu::BindGroupLayout>,
    pub arena: Option<Arena>,
    pub resolver: Resolver,
    pub pipelines: std::collections::HashMap<(Source, Oit, u32), wgpu::RenderPipeline>,
    layouts: std::collections::HashMap<(&'static str, bool), wgpu::PipelineLayout>,
}

impl OitState {
    /// `requested` on a device with `flags` and `limits`: the linked lists
    /// only where it can keep them ([`supports_lists`]).
    pub fn new(
        device: &wgpu::Device,
        requested: Transparency,
        layers: u32,
        flags: wgpu::DownlevelFlags,
        limits: &wgpu::Limits,
        frame_entries: &[wgpu::BindGroupLayoutEntry],
    ) -> OitState {
        let mode = match requested {
            Transparency::LinkedList if supports_lists(flags, limits) => Transparency::LinkedList,
            _ => Transparency::Weighted,
        };
        let frame_list_layout = supports_lists(flags, limits).then(|| {
            let mut entries = frame_entries.to_vec();
            entries.push(arena_layout_entry());
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("frame (oit arena)"),
                entries: &entries,
            })
        });
        OitState {
            mode,
            layers,
            budget: arena_budget(limits),
            frame_list_layout,
            arena: None,
            resolver: Resolver::default(),
            pipelines: Default::default(),
            layouts: Default::default(),
        }
    }

    /// The pass mode of a frame drawn at `width` x `height`: the linked
    /// lists while an arena of at least one node a pixel fits, the
    /// weighted mode with its front layer while one word a pixel fits
    /// (the arena made or made again here at that size), else weighted
    /// alone (no arena: the device writes no storage from fragments).
    pub fn frame_mode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> Oit {
        if self.frame_list_layout.is_none() {
            return Oit::Weighted;
        }
        let list = (self.mode == Transparency::LinkedList)
            .then(|| ArenaSize::new(width, height, self.layers, self.budget))
            .flatten();
        let (size, mode) = match list {
            Some(size) => (Some(size), Oit::List),
            None => (ArenaSize::front(width, height, self.budget), Oit::Front),
        };
        let Some(size) = size else {
            self.arena = None;
            return Oit::Weighted;
        };
        if self.arena.as_ref().is_none_or(|a| a.size != size) {
            self.arena = Some(Arena::new(device, queue, size));
        }
        mode
    }

    /// The pipeline layout `name` of an OIT pass in `mode`: group 0 the
    /// frame group (with the arena for the linked lists), then `rest`.
    pub fn layout(
        &mut self,
        device: &wgpu::Device,
        name: &'static str,
        mode: Oit,
        frame: &wgpu::BindGroupLayout,
        rest: &[&wgpu::BindGroupLayout],
    ) -> &wgpu::PipelineLayout {
        let list = mode.arena();
        let frame = match (&self.frame_list_layout, list) {
            (Some(l), true) => l,
            _ => frame,
        };
        self.layouts.entry((name, list)).or_insert_with(|| {
            let groups: Vec<_> = std::iter::once(Some(frame))
                .chain(rest.iter().map(|l| Some(*l)))
                .collect();
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(name),
                bind_group_layouts: &groups,
                immediate_size: 0,
            })
        })
    }

    /// The arena's bytes (GPU memory).
    pub fn bytes(&self) -> u64 {
        self.arena.as_ref().map_or(0, |a| a.size.bytes)
    }
}

/// The resolve's module: the full-screen triangle, the targets it reads
/// (multisampled with `msaa`, per sample) and, with `list`, the arena and
/// the depth, then `oit_resolve.wgsl` (and `oit_list.wgsl`).
pub(crate) fn resolve_source(msaa: bool, list: bool) -> String {
    let fullscreen = wgsl!("fullscreen.wgsl");
    let (texture, sample) = if msaa {
        ("texture_multisampled_2d<f32>", "i32(s)")
    } else {
        ("texture_2d<f32>", "0")
    };
    let mut s = format!(
        "{fullscreen}
@group(0) @binding(0) var oit_accum: {texture};
@group(0) @binding(1) var oit_reveal: {texture};
@group(0) @binding(2) var oit_front: {texture};
fn oit_accum_at(p: vec2<i32>, s: u32) -> vec4<f32> {{ return textureLoad(oit_accum, p, {sample}); }}
fn oit_reveal_at(p: vec2<i32>, s: u32) -> vec2<f32> {{ return textureLoad(oit_reveal, p, {sample}).rg; }}
fn oit_front_at(p: vec2<i32>, s: u32) -> vec4<f32> {{ return textureLoad(oit_front, p, {sample}); }}
"
    );
    if list {
        let depth = if msaa {
            "texture_depth_multisampled_2d"
        } else {
            "texture_depth_2d"
        };
        s.push_str(&format!(
            "@group(0) @binding(3) var oit_depth: {depth};
@group(0) @binding(4) var<storage, read> oit_nodes: array<u32>;
fn oit_depth_at(p: vec2<i32>, s: u32) -> f32 {{ return textureLoad(oit_depth, p, {sample}); }}
{LIST_WGSL}"
        ));
    } else {
        s.push_str("fn oit_exact(p: vec2<i32>, s: u32) -> vec4<f32> { return vec4<f32>(-1.0); }\n");
    }
    s.push_str(RESOLVE_WGSL);
    s.push_str(if msaa {
        "@fragment
fn fs_resolve(@builtin(position) pos: vec4<f32>, @builtin(sample_index) s: u32) -> @location(0) vec4<f32> {
  return oit_resolve(vec2<i32>(pos.xy), s);
}
"
    } else {
        "@fragment
fn fs_resolve(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
  return oit_resolve(vec2<i32>(pos.xy), 0u);
}
"
    });
    s
}

/// The resolve's pipelines, by (samples, list), made at first use.
#[derive(Default)]
pub(crate) struct Resolver {
    pipelines:
        std::collections::HashMap<(u32, bool), (wgpu::BindGroupLayout, wgpu::RenderPipeline)>,
}

impl Resolver {
    /// The resolve's layout and pipeline into the HDR target at `samples`.
    pub fn ensure(
        &mut self,
        device: &wgpu::Device,
        samples: u32,
        list: bool,
    ) -> &(wgpu::BindGroupLayout, wgpu::RenderPipeline) {
        self.pipelines.entry((samples, list)).or_insert_with(|| {
            let msaa = samples > 1;
            let texture = |binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: msaa,
                },
                count: None,
            };
            let mut entries = vec![texture(0), texture(1), texture(2)];
            if list {
                entries.push(wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: msaa,
                    },
                    count: None,
                });
                entries.push(wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                });
            }
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("oit resolve"),
                entries: &entries,
            });
            let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("oit resolve"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            let module = crate::shader(device, "oit resolve", resolve_source(msaa, list));
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("oit resolve"),
                layout: Some(&pl),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: samples,
                    ..Default::default()
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs_resolve"),
                    compilation_options: Default::default(),
                    // Premultiplied over the scene; the HDR alpha (the
                    // untonemapped sprites' cover) is left as it was.
                    targets: &[Some(wgpu::ColorTargetState {
                        format: crate::HDR_FORMAT,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::Zero,
                                dst_factor: wgpu::BlendFactor::One,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: crate::pipeline_cache::of(device).as_ref(),
            });
            (layout, pipeline)
        })
    }

    /// The pipeline [`Resolver::ensure`] made.
    pub fn get(
        &self,
        samples: u32,
        list: bool,
    ) -> Option<&(wgpu::BindGroupLayout, wgpu::RenderPipeline)> {
        self.pipelines.get(&(samples, list))
    }
}

/// The weighted blend's difference between two orders of the same `n`
/// fragments, as a bound on one resolved, premultiplied linear channel
/// whose weighted mean colour is `mean` (accumulated colour over
/// accumulated weight) under revealage `reveal`.
///
/// RGBA16F rounds each blend to the nearest half float, a relative error
/// of at most u = 2^-11 of the running sum (the sums only grow: every term
/// is non-negative), and the first fragment's own rounding depends on
/// which fragment comes first: n order-dependent roundings, so each sum of
/// one order is within n·u of the exact one, and two orders within 2n·u of
/// each other; likewise the revealage product. The resolved channel
/// `mean · (1 − reveal)` then moves by at most
/// `mean · (1 − reveal) · 2·(2n·u) + mean · reveal · 2n·u`
/// `= 2n·u · mean · (2 − reveal)` (first order).
pub fn weighted_order_bound(n: u32, mean: f32, reveal: f32) -> f32 {
    let u = 2f32.powi(-11);
    2.0 * n as f32 * u * mean * (2.0 - reveal.clamp(0.0, 1.0))
}

/// [`weighted_order_bound`] as SDR channel codes after the sRGB transfer
/// at linear value `linear`: its slope there times the bound, rounded up
/// to whole codes (two values any closer can still round to neighbouring
/// codes).
pub fn weighted_order_codes(n: u32, mean: f32, reveal: f32, linear: f32) -> u32 {
    let x = linear.clamp(1e-6, 1.0);
    let slope = if x <= 0.0031308 {
        12.92
    } else {
        1.055 / 2.4 * x.powf(1.0 / 2.4 - 1.0)
    };
    (weighted_order_bound(n, mean, reveal) * slope * 255.0).ceil() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_budget_has_exact_pixel_boundaries() {
        // 3x2 pixels, one count word and one seven-word node each:
        // 16 + 6 * 32. One byte less cannot hold a node for every pixel.
        assert_eq!(ArenaSize::new(3, 2, 4, 207), None);
        let one = ArenaSize::new(3, 2, 4, 208).unwrap();
        assert_eq!(one.nodes_per_pixel, 1);
        assert_eq!(one.bytes, 208);
        assert_eq!(one.header(), [3, 2, 1, 0]);
        // Two nodes need 16 + 6 * 60 = 376.
        assert_eq!(ArenaSize::new(3, 2, 4, 375).unwrap().nodes_per_pixel, 1);
        assert_eq!(ArenaSize::new(3, 2, 4, 376).unwrap().nodes_per_pixel, 2);
        assert_eq!(ArenaSize::new(3, 2, 1, 400).unwrap().nodes_per_pixel, 1);
        // The clear covers the overflow count and the six counts only.
        assert_eq!(one.cleared(), (12, 4 + 6 * 4));
        assert_eq!(ArenaSize::new(0, 2, 4, 1 << 20), None);
    }

    #[test]
    fn the_device_budget_resolves_the_requested_layers() {
        // WebGPU's baseline storage binding (128 MiB).
        let limit = 128 * 1024 * 1024;
        assert_eq!(
            ArenaSize::new(1280, 720, 0, limit).unwrap().nodes_per_pixel,
            4
        );
        assert_eq!(
            ArenaSize::new(1920, 1080, 0, limit)
                .unwrap()
                .nodes_per_pixel,
            2
        );
        assert_eq!(ArenaSize::new(3840, 2160, 0, limit), None);
        let high = ArenaSize::new(3840, 2160, 4, 1024 * 1024 * 1024).unwrap();
        assert_eq!(high.nodes_per_pixel, 4);
        assert!(high.bytes <= 1024 * 1024 * 1024);
        assert_eq!(
            ArenaSize::new(1, 1, 100, limit).unwrap().nodes_per_pixel,
            MAX_TRANSPARENCY_LAYERS
        );
        // Word indices stay within u32 whatever the device allows.
        let huge = ArenaSize::new(16384, 16384, 32, u64::MAX).unwrap();
        assert!(huge.bytes <= u64::from(u32::MAX) * 4);
    }

    /// IEEE half precision, round to nearest even (what an RGBA16F blend
    /// stores).
    fn half(x: f32) -> f32 {
        let b = x.to_bits();
        let sign = (b >> 16) & 0x8000;
        let exp = ((b >> 23) & 0xff) as i32 - 127 + 15;
        let mant = b & 0x7f_ffff;
        // Drops `shift` bits of `m`, rounding to nearest even.
        let round = |m: u32, shift: u32| {
            let kept = m >> shift;
            let rest = m & ((1 << shift) - 1);
            let halfway = 1 << (shift - 1);
            kept + u32::from(rest > halfway || (rest == halfway && kept & 1 == 1))
        };
        let h = if exp >= 31 {
            sign | 0x7c00
        } else if exp <= 0 {
            if exp < -10 {
                sign
            } else {
                sign | round(mant | 0x80_0000, (14 - exp) as u32)
            }
        } else {
            // A carry out of the mantissa is the next exponent's, as it
            // should be (and 0x7c00 is infinity).
            sign | round(((exp as u32) << 23) | mant, 13)
        };
        let (s, e, m) = (h >> 15, (h >> 10) & 31, h & 1023);
        let magnitude = match e {
            0 => m as f32 * 2f32.powi(-24),
            31 => f32::INFINITY,
            e => (1.0 + m as f32 / 1024.0) * 2f32.powi(e as i32 - 15),
        };
        if s == 1 { -magnitude } else { magnitude }
    }

    /// `oit.wgsl` `oit_weight`, on the CPU.
    fn weight(alpha: f32, depth: f32) -> f32 {
        let a = (alpha * 10.0).min(1.0) + 0.01;
        let z = 1.0 - depth.clamp(0.0, 1.0) * 0.9;
        (a * a * a * 1e8 * z * z * z).clamp(1e-2, 3e3) / 3e3
    }

    /// One red channel through the weighted targets in `order`: the
    /// colour sum, the weight sum and the revealage, each blend rounded
    /// to half precision as RGBA16F and R16F store it.
    fn accumulate(order: &[(f32, f32, f32)]) -> (f32, f32, f32) {
        let (mut color, mut weights, mut reveal) = (0.0f32, 0.0f32, 1.0f32);
        for &(red, alpha, depth) in order {
            let w = weight(alpha, depth);
            color = half(color + red * alpha * w);
            weights = half(weights + alpha * w);
            reveal = half(reveal * (1.0 - alpha));
        }
        (color, weights, reveal)
    }

    fn srgb_code(linear: f32) -> u8 {
        let x = linear.clamp(0.0, 1.0);
        let s = if x <= 0.0031308 {
            x * 12.92
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round() as u8
    }

    /// The handoff's counterexample (OIT closing report): four fragments
    /// (linear red, alpha, window depth) whose weighted result is not
    /// the same in both orders, red 218 against 219 after the sRGB8
    /// encode. The weighted fallback is held to the bound its RGBA16F
    /// rounding allows ([`weighted_order_codes`]), at most one code a
    /// channel, never to bit equality.
    #[test]
    // The report's values, as it gives them.
    #[allow(clippy::excessive_precision)]
    fn weighted_results_of_two_orders_stay_within_the_half_float_bound() {
        let forward = [
            (0.926_398_86, 0.691_721_2, 0.901_566_6),
            (0.289_832_96, 0.384_999_8, 0.392_899_4),
            (0.998_792_5, 0.580_259, 0.360_709_3),
            (0.428_052_75, 0.297_639_72, 0.048_268_097),
        ];
        let reverse: Vec<_> = forward.iter().rev().copied().collect();
        let f = accumulate(&forward);
        let r = accumulate(&reverse);
        // The sums the report gives for each order.
        assert_eq!(f, (1.458_984_4, 1.955_078_1, 0.055_908_203));
        assert_eq!(r, (1.459_960_9, 1.954_101_6, 0.055_877_686));
        let resolve = |(c, w, rev): (f32, f32, f32)| c / w * (1.0 - rev);
        let (a, b) = (resolve(f), resolve(r));
        assert_eq!((srgb_code(a), srgb_code(b)), (218, 219));
        let mean = f.0 / f.1;
        assert!((a - b).abs() <= weighted_order_bound(4, mean, f.2));
        let codes = weighted_order_codes(4, mean, f.2, a);
        assert_eq!(codes, 1, "the tolerance for four fragments here");
        assert!(u32::from(srgb_code(b).abs_diff(srgb_code(a))) <= codes);
    }

    #[test]
    fn half_rounding_matches_ieee() {
        assert_eq!(half(1.0), 1.0);
        assert_eq!(half(65504.0), 65504.0);
        assert!(half(70000.0).is_infinite());
        // 1 + 2^-11 is a tie between 1 and 1 + 2^-10: to the even, 1.
        assert_eq!(half(1.0 + 2f32.powi(-11)), 1.0);
        assert_eq!(half(1.0 + 3.0 * 2f32.powi(-11)), 1.0 + 2.0 * 2f32.powi(-10));
        assert_eq!(half(2f32.powi(-24)), 2f32.powi(-24));
    }
}
