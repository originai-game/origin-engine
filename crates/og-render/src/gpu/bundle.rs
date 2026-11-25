//! Render bundles: a part of a pass whose draws stay the same from frame
//! to frame, recorded once into a `wgpu::RenderBundle` and replayed by
//! one call. On the web one `executeBundles` takes the place of the
//! hundreds of calls (and their crossings into JavaScript) the part makes;
//! natively the bundle's commands were checked when it was recorded.
//!
//! The draw helpers write to an [`Encode`]: a render pass, or [`Draws`],
//! which records them. [`Bundles::play`] draws a part through `Draws`
//! every frame and compares what it recorded with the part's last frame:
//! the same commands into the same targets execute the part's bundle
//! (recorded the second frame they repeat), anything else is played
//! straight into the pass and kept for the next frame's comparison. The
//! commands hold their pipelines, groups and buffers, so a resource
//! remade (a buffer grown, a group rebuilt) is a different command.

use std::collections::HashMap;
use std::ops::Range;
use std::sync::Mutex;

/// What the draw helpers draw with: the subset of `wgpu::RenderPass` a
/// render bundle records (no viewport, scissor, blend constant or stencil
/// reference).
pub(crate) trait Encode {
    fn set_pipeline(&mut self, pipeline: &wgpu::RenderPipeline);
    fn set_bind_group<'g>(
        &mut self,
        index: u32,
        group: impl Into<Option<&'g wgpu::BindGroup>>,
        offsets: &[wgpu::DynamicOffset],
    );
    fn set_vertex_buffer(&mut self, slot: u32, slice: wgpu::BufferSlice<'_>);
    fn set_index_buffer(&mut self, slice: wgpu::BufferSlice<'_>, format: wgpu::IndexFormat);
    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>);
    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>);
    fn draw_indirect(&mut self, args: &wgpu::Buffer, offset: wgpu::BufferAddress);
    fn draw_indexed_indirect(&mut self, args: &wgpu::Buffer, offset: wgpu::BufferAddress);
    fn multi_draw_indexed_indirect(
        &mut self,
        args: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        count: u32,
    );
}

impl Encode for wgpu::RenderPass<'_> {
    fn set_pipeline(&mut self, pipeline: &wgpu::RenderPipeline) {
        wgpu::RenderPass::set_pipeline(self, pipeline);
    }
    fn set_bind_group<'g>(
        &mut self,
        index: u32,
        group: impl Into<Option<&'g wgpu::BindGroup>>,
        offsets: &[wgpu::DynamicOffset],
    ) {
        wgpu::RenderPass::set_bind_group(self, index, group.into(), offsets);
    }
    fn set_vertex_buffer(&mut self, slot: u32, slice: wgpu::BufferSlice<'_>) {
        wgpu::RenderPass::set_vertex_buffer(self, slot, slice);
    }
    fn set_index_buffer(&mut self, slice: wgpu::BufferSlice<'_>, format: wgpu::IndexFormat) {
        wgpu::RenderPass::set_index_buffer(self, slice, format);
    }
    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        wgpu::RenderPass::draw(self, vertices, instances);
    }
    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        wgpu::RenderPass::draw_indexed(self, indices, base_vertex, instances);
    }
    fn draw_indirect(&mut self, args: &wgpu::Buffer, offset: wgpu::BufferAddress) {
        wgpu::RenderPass::draw_indirect(self, args, offset);
    }
    fn draw_indexed_indirect(&mut self, args: &wgpu::Buffer, offset: wgpu::BufferAddress) {
        wgpu::RenderPass::draw_indexed_indirect(self, args, offset);
    }
    fn multi_draw_indexed_indirect(
        &mut self,
        args: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        count: u32,
    ) {
        if count == 1 {
            wgpu::RenderPass::draw_indexed_indirect(self, args, offset);
        } else if count > 1 {
            wgpu::RenderPass::multi_draw_indexed_indirect(self, args, offset, count);
        }
    }
}

#[derive(Clone, PartialEq)]
enum Command {
    Pipeline(wgpu::RenderPipeline),
    /// The group's dynamic offsets are `Draws::offsets[range]`.
    Group(u32, Option<wgpu::BindGroup>, Range<usize>),
    Vertex(u32, wgpu::Buffer, wgpu::BufferAddress, wgpu::BufferAddress),
    Index(
        wgpu::Buffer,
        wgpu::IndexFormat,
        wgpu::BufferAddress,
        wgpu::BufferAddress,
    ),
    Draw(Range<u32>, Range<u32>),
    DrawIndexed(Range<u32>, i32, Range<u32>),
    DrawIndirect(wgpu::Buffer, wgpu::BufferAddress),
    DrawIndexedIndirect(wgpu::Buffer, wgpu::BufferAddress),
    MultiDrawIndexedIndirect(wgpu::Buffer, wgpu::BufferAddress, u32),
}

/// A part's commands in order, as the draw helpers wrote them.
#[derive(Default, PartialEq)]
pub(crate) struct Draws {
    commands: Vec<Command>,
    offsets: Vec<wgpu::DynamicOffset>,
}

impl Draws {
    fn clear(&mut self) {
        self.commands.clear();
        self.offsets.clear();
    }
}

impl Encode for Draws {
    fn set_pipeline(&mut self, pipeline: &wgpu::RenderPipeline) {
        if matches!(self.commands.last(), Some(Command::Pipeline(p)) if p == pipeline) {
            return;
        }
        self.commands.push(Command::Pipeline(pipeline.clone()));
    }
    fn set_bind_group<'g>(
        &mut self,
        index: u32,
        group: impl Into<Option<&'g wgpu::BindGroup>>,
        offsets: &[wgpu::DynamicOffset],
    ) {
        let g = group.into();
        if offsets.is_empty()
            && matches!(
                self.commands.last(),
                Some(Command::Group(i, prev, range)) if *i == index && range.is_empty() && prev.as_ref() == g
            )
        {
            return;
        }
        let at = self.offsets.len();
        self.offsets.extend_from_slice(offsets);
        self.commands
            .push(Command::Group(index, g.cloned(), at..self.offsets.len()));
    }
    fn set_vertex_buffer(&mut self, slot: u32, slice: wgpu::BufferSlice<'_>) {
        let (buf, off, size) = (slice.buffer(), slice.offset(), slice.size());
        if matches!(
            self.commands.last(),
            Some(Command::Vertex(s, b, o, sz)) if *s == slot && b == buf && *o == off && *sz == size
        ) {
            return;
        }
        self.commands
            .push(Command::Vertex(slot, buf.clone(), off, size));
    }
    fn set_index_buffer(&mut self, slice: wgpu::BufferSlice<'_>, format: wgpu::IndexFormat) {
        let (buf, off, size) = (slice.buffer(), slice.offset(), slice.size());
        if matches!(
            self.commands.last(),
            Some(Command::Index(b, f, o, sz)) if b == buf && *f == format && *o == off && *sz == size
        ) {
            return;
        }
        self.commands
            .push(Command::Index(buf.clone(), format, off, size));
    }
    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.commands.push(Command::Draw(vertices, instances));
    }
    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.commands
            .push(Command::DrawIndexed(indices, base_vertex, instances));
    }
    fn draw_indirect(&mut self, args: &wgpu::Buffer, offset: wgpu::BufferAddress) {
        self.commands
            .push(Command::DrawIndirect(args.clone(), offset));
    }
    fn draw_indexed_indirect(&mut self, args: &wgpu::Buffer, offset: wgpu::BufferAddress) {
        self.commands
            .push(Command::DrawIndexedIndirect(args.clone(), offset));
    }
    fn multi_draw_indexed_indirect(
        &mut self,
        args: &wgpu::Buffer,
        offset: wgpu::BufferAddress,
        count: u32,
    ) {
        if count == 1 {
            self.commands
                .push(Command::DrawIndexedIndirect(args.clone(), offset));
        } else if count > 1 {
            self.commands.push(Command::MultiDrawIndexedIndirect(
                args.clone(),
                offset,
                count,
            ));
        }
    }
}

/// Plays `draws` into `$out`: a render pass (`multi = true`) or a bundle
/// encoder (`multi = false`, where multi-draw expands into indexed indirect).
macro_rules! replay {
    ($draws:expr, $out:expr, $multi:tt) => {{
        let (draws, out) = ($draws, $out);
        for c in &draws.commands {
            match c {
                Command::Pipeline(p) => out.set_pipeline(p),
                Command::Group(i, g, o) => {
                    out.set_bind_group(*i, g.as_ref(), &draws.offsets[o.clone()])
                }
                Command::Vertex(s, b, at, size) => {
                    out.set_vertex_buffer(*s, b.slice(*at..at + size))
                }
                Command::Index(b, f, at, size) => out.set_index_buffer(b.slice(*at..at + size), *f),
                Command::Draw(v, i) => out.draw(v.clone(), i.clone()),
                Command::DrawIndexed(x, b, i) => out.draw_indexed(x.clone(), *b, i.clone()),
                Command::DrawIndirect(b, at) => out.draw_indirect(b, *at),
                Command::DrawIndexedIndirect(b, at) => out.draw_indexed_indirect(b, *at),
                Command::MultiDrawIndexedIndirect(b, at, n) => {
                    replay!(@mdi out, b, *at, *n, $multi);
                }
            }
        }
    }};
    (@mdi $out:expr, $b:expr, $at:expr, $n:expr, true) => {
        $out.multi_draw_indexed_indirect($b, $at, $n);
    };
    (@mdi $out:expr, $b:expr, $at:expr, $n:expr, false) => {
        for k in 0..$n {
            $out.draw_indexed_indirect($b, $at + u64::from(k) * 20);
        }
    };
}

/// The attachments a part draws into, which its bundle must match.
#[derive(Clone, PartialEq, Debug)]
pub(crate) struct Target {
    colors: Vec<Option<wgpu::TextureFormat>>,
    depth: Option<wgpu::RenderBundleDepthStencil>,
    samples: u32,
}

impl Target {
    /// A pass's colour views and depth view (with whether the pass only
    /// reads its depth), as its descriptor names them.
    pub(crate) fn of(
        colors: &[Option<&wgpu::TextureView>],
        depth: Option<(&wgpu::TextureView, bool)>,
    ) -> Target {
        let samples = colors
            .iter()
            .flatten()
            .map(|v| v.texture().sample_count())
            .chain(depth.map(|(v, _)| v.texture().sample_count()))
            .next()
            .unwrap_or(1);
        Target {
            colors: colors
                .iter()
                .map(|v| v.map(|v| v.texture().format()))
                .collect(),
            depth: depth.map(|(v, read_only)| wgpu::RenderBundleDepthStencil {
                format: v.texture().format(),
                depth_read_only: read_only,
                stencil_read_only: true,
            }),
            samples,
        }
    }
}

/// A part's last frame: what it drew, into what, and its bundle once the
/// same commands came twice.
struct Part {
    target: Target,
    draws: Draws,
    bundle: Option<wgpu::RenderBundle>,
    /// The frame it was last played.
    played: u64,
}

/// What [`Bundles::play`] did with a part this frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Played {
    /// Its commands changed (or it is new): played straight into the pass.
    Direct,
    /// The same as its last frame, the first time: its bundle recorded and
    /// executed.
    Recorded,
    /// Its bundle executed again.
    Executed,
    /// Nothing to draw.
    Empty,
}

impl Played {
    /// Counts the part into the frame's stats.
    pub(crate) fn count(self, stats: &mut crate::FrameStats) {
        match self {
            Played::Direct => stats.bundles_direct += 1,
            Played::Recorded => {
                stats.bundles += 1;
                stats.bundles_recorded += 1;
            }
            Played::Executed => stats.bundles += 1,
            Played::Empty => {}
        }
    }
}

/// A pass part: a name the pass gives it (`"main: instances"`, a shadow
/// layer's) and an index among parts of that name.
pub(crate) type PartName = (&'static str, u32);

/// Each pass part's last frame and bundle, a reusable scratch buffer, and the
/// frame they were last pruned on.
#[derive(Default)]
pub(crate) struct Bundles {
    parts: Mutex<(u64, Draws, HashMap<PartName, Part>)>,
}

/// Frames a part is kept after it was last played: the shadow cache
/// redraws a layer's static casters only some frames.
const KEEP_FRAMES: u64 = 8;

impl Bundles {
    /// Draws part `part` of `pass` (attachments `target`) by `draw` on
    /// frame `frame`. `draw` starts with nothing bound and leaves `pass`
    /// with nothing bound that the pass may count on: a bundle resets
    /// the pass's pipeline, groups and buffers.
    pub(crate) fn play<R>(
        &self,
        device: &wgpu::Device,
        pass: &mut wgpu::RenderPass<'_>,
        part: PartName,
        target: &Target,
        frame: u64,
        draw: impl FnOnce(&mut Draws) -> R,
    ) -> (R, Played) {
        let mut draws = {
            let mut guard = self.parts.lock().expect("bundles");
            let mut d = std::mem::take(&mut guard.1);
            d.clear();
            d
        };
        let out = draw(&mut draws);
        let mut guard = self.parts.lock().expect("bundles");
        let (last, scratch, parts) = &mut *guard;
        if *last != frame {
            *last = frame;
            parts.retain(|_, p| p.played + KEEP_FRAMES >= frame);
        }
        if draws.commands.is_empty() {
            parts.remove(&part);
            draws.clear();
            *scratch = draws;
            return (out, Played::Empty);
        }
        if let Some(p) = parts.get_mut(&part)
            && p.target == *target
            && p.draws.commands.len() == draws.commands.len()
            && p.draws.offsets.len() == draws.offsets.len()
            && p.draws == draws
        {
            p.played = frame;
            let played = if p.bundle.is_some() {
                Played::Executed
            } else {
                Played::Recorded
            };
            let bundle = p
                .bundle
                .get_or_insert_with(|| record(device, &p.draws, target, part.0));
            pass.execute_bundles([&*bundle]);
            draws.clear();
            *scratch = draws;
            return (out, played);
        }
        replay!(&draws, &mut *pass, true);
        if let Some(prev) = parts.insert(
            part,
            Part {
                target: target.clone(),
                draws,
                bundle: None,
                played: frame,
            },
        ) {
            let mut recycled = prev.draws;
            recycled.clear();
            *scratch = recycled;
        }
        (out, Played::Direct)
    }
}

fn record(
    device: &wgpu::Device,
    draws: &Draws,
    target: &Target,
    label: &'static str,
) -> wgpu::RenderBundle {
    let mut encoder = device.create_render_bundle_encoder(&wgpu::RenderBundleEncoderDescriptor {
        label: Some(label),
        color_formats: &target.colors,
        depth_stencil: target.depth,
        sample_count: target.samples,
        multiview: None,
    });
    replay!(draws, &mut encoder, false);
    encoder.finish(&wgpu::RenderBundleDescriptor { label: Some(label) })
}
