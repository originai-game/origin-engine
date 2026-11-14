//! Render graph realization, bindings and post encoding.

use super::*;

mod ao;
mod bloom;
mod chain;
mod contact_shadows;
mod cx;
mod exposure;
mod fx;
mod lens_flare;
mod outline;
mod screen;
mod temporal;
mod tonemap;
use cx::*;

/// The post passes' bind groups for one realization of the graph, by
/// family: each makes its own groups and encodes its own passes.
struct Bindings {
    fx: fx::Fx,
    contact: Option<contact_shadows::ContactShadows>,
    temporal: temporal::Temporal,
    exposure: exposure::Exposure,
    bloom: bloom::Bloom,
    flare: lens_flare::LensFlare,
    tonemap: tonemap::Tonemap,
    chain: chain::Chain,
}

impl Bindings {
    fn new(cx: &BindCx) -> Bindings {
        Bindings {
            fx: fx::Fx::bind(cx),
            contact: contact_shadows::ContactShadows::bind(cx),
            temporal: temporal::Temporal::bind(cx),
            exposure: exposure::Exposure::bind(cx),
            bloom: bloom::Bloom::bind(cx),
            flare: lens_flare::LensFlare::bind(cx),
            tonemap: tonemap::Tonemap::bind(cx),
            chain: chain::Chain::bind(cx),
        }
    }

    /// Records `pass` with the family it is of; `None` for a pass that is
    /// no post pass.
    fn encode(&self, encoder: &mut crate::Encoder, cx: &PostCx, pass: FramePass) -> Option<u32> {
        if pass == FramePass::Contact {
            let contact = self.contact.as_ref().expect("contact bound");
            return Some(contact.encode(encoder, cx));
        }
        self.fx
            .encode(encoder, cx, pass)
            .or_else(|| self.temporal.encode(encoder, cx, pass))
            .or_else(|| self.exposure.encode(encoder, cx, pass))
            .or_else(|| self.bloom.encode(encoder, cx, pass))
            .or_else(|| self.flare.encode(encoder, cx, pass))
            .or_else(|| self.tonemap.encode(encoder, cx, pass))
            .or_else(|| self.chain.encode(encoder, cx, pass))
    }
}

/// The frame's render graph: the compiled plan for the current pass list,
/// the transient textures realized from it (kept in a pool across frames
/// and reused by description) and the bind groups that read them.
#[derive(Default)]
pub(crate) struct FrameGraph {
    passes: Vec<FramePass>,
    cfg: Option<GraphConfig>,
    /// Identifies the grading LUT the bindings were made with.
    lut_key: Option<(u64, usize)>,
    /// Identifies the palette and canvas the stylise bindings were made with.
    stylise_key: u64,
    order: Vec<FramePass>,
    plan: Option<GraphPlan>,
    pool: Vec<(TargetDesc, wgpu::TextureView)>,
    views: HashMap<String, wgpu::TextureView>,
    bindings: Option<Bindings>,
    /// The fog of war texture the bindings hold (`SightTexture::generation`).
    sight_key: u64,
    /// Bumped whenever the targets are realized anew (views change).
    pub generation: u64,
}

impl FrameGraph {
    /// GPU bytes of the targets realized for the current plan.
    pub fn target_bytes(&self) -> u64 {
        self.pool
            .iter()
            .map(|(_, v)| crate::gpu::texture::view_bytes(v))
            .sum()
    }

    /// Forgets everything size-dependent (on resize).
    pub fn invalidate(&mut self) {
        let generation = self.generation + 1;
        *self = FrameGraph::default();
        self.generation = generation;
    }

    /// The plan the last [`FrameGraph::prepare`] compiled.
    pub fn plan(&self) -> Option<&GraphPlan> {
        self.plan.as_ref()
    }

    /// The passes in execution order.
    pub fn order(&self) -> &[FramePass] {
        &self.order
    }

    /// Physical transient textures of the current plan.
    pub fn transient_textures(&self) -> usize {
        self.plan.as_ref().map_or(0, |p| p.physical().len())
    }

    pub fn view(&self, name: &str) -> &wgpu::TextureView {
        self.views
            .get(name)
            .unwrap_or_else(|| panic!("render graph target '{name}' is not realized"))
    }

    /// What custom post pass `i` reads and writes (`post_custom.rs`): the
    /// lit image before it (the TAA history of `slot` when TAA wrote it
    /// last) and its own target.
    pub fn custom_post_views<'a>(
        &'a self,
        persistent: &'a Persistent,
        i: u32,
        slot: usize,
    ) -> (&'a wgpu::TextureView, &'a wgpu::TextureView) {
        let read = match i {
            0 => match lit_names(&self.passes).before_custom {
                Some(n) => self.view(n),
                None => &persistent.history.as_ref().expect("TAA history")[slot],
            },
            i => self.view(CUSTOM_POST_NAMES[i as usize - 1]),
        };
        (read, self.view(CUSTOM_POST_NAMES[i as usize]))
    }

    /// What FSR 3.1 reads and writes in the `taa` pass's place (fsr.rs):
    /// the lit image TAA would read, the depth, the prepass's gbuffer and
    /// view depth, the OIT revealage (white without OIT), the history slot
    /// TAA would write and the one it would read, and the sharpening's
    /// target.
    #[allow(clippy::type_complexity)]
    pub fn fsr_views<'a>(
        &'a self,
        no_reactive: &'a wgpu::TextureView,
        persistent: &'a Persistent,
        slot: usize,
    ) -> Option<(
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
        Option<(&'a wgpu::TextureView, &'a wgpu::TextureView)>,
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
        &'a wgpu::TextureView,
        Option<&'a wgpu::TextureView>,
    )> {
        let history = persistent.history.as_ref()?;
        if !self.passes.contains(&FramePass::Taa) {
            return None;
        }
        let prepass = self.try_view("gbuffer").zip(self.try_view("lindepth"));
        Some((
            self.view(lit_names(&self.passes).main),
            self.view("depth"),
            prepass,
            self.try_view("oit.reveal").unwrap_or(no_reactive),
            &history[slot],
            &history[1 - slot],
            self.try_view("hdr.sharp"),
        ))
    }

    /// The view of `name` when the current graph has it.
    pub fn try_view(&self, name: &str) -> Option<&wgpu::TextureView> {
        self.views.get(name)
    }

    /// Compiles the graph for `passes` and realizes its transient targets,
    /// unless the previous frame had the same passes, configuration and LUT.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        passes: Vec<FramePass>,
        cfg: &GraphConfig,
        post: &PostPasses,
        persistent: &Persistent,
        probes: &wgpu::Buffer,
        lut: (Option<(u64, usize)>, &wgpu::TextureView),
        stylise: (
            &wgpu::TextureView,
            &wgpu::TextureView,
            &wgpu::TextureView,
            u64,
        ),
    ) -> Result<(), GraphError> {
        if self.plan.is_some()
            && passes == self.passes
            && self.cfg.as_ref() == Some(cfg)
            && self.lut_key == lut.0
            && self.stylise_key == stylise.3
            && self.sight_key == post.sight.generation
        {
            return Ok(());
        }
        let plan = build_graph(&passes, cfg).compile()?;
        self.order = plan.order().iter().map(|&i| passes[i]).collect();
        // Physical slots to pool textures: the first unused entry with the
        // same description, else a new texture.
        let mut taken = vec![false; self.pool.len()];
        let mut slot_views = Vec::with_capacity(plan.physical().len());
        for desc in plan.physical() {
            let found = (0..self.pool.len()).find(|&i| !taken[i] && self.pool[i].0 == *desc);
            let index = match found {
                Some(i) => i,
                None => {
                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("graph transient"),
                        size: wgpu::Extent3d {
                            width: desc.width,
                            height: desc.height,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: desc.samples,
                        dimension: wgpu::TextureDimension::D2,
                        format: desc.format,
                        usage: desc.usage,
                        view_formats: &[],
                    });
                    self.pool
                        .push((*desc, texture.create_view(&Default::default())));
                    taken.push(false);
                    self.pool.len() - 1
                }
            };
            taken[index] = true;
            slot_views.push(self.pool[index].1.clone());
        }
        self.views.clear();
        for (name, slot) in plan.transients() {
            self.views
                .insert(name.to_string(), slot_views[slot].clone());
        }
        self.bindings = Some(Bindings::new(&BindCx {
            device,
            graph: self,
            passes: post,
            persistent,
            probes,
            frame: &passes,
            cfg,
            lut: lut.1,
            stylise: (stylise.0, stylise.1, stylise.2),
            names: lit_names(&passes),
        }));
        self.generation += 1;
        self.passes = passes;
        self.cfg = Some(*cfg);
        self.lut_key = lut.0;
        self.stylise_key = stylise.3;
        self.sight_key = post.sight.generation;
        self.plan = Some(plan);
        Ok(())
    }

    /// Records one post or effect pass; returns its draws. Every pass the
    /// frame's plans do not encode (pass.rs) is a post family's.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_post(
        &self,
        encoder: &mut crate::Encoder,
        passes: &PostPasses,
        persistent: &Persistent,
        pass: FramePass,
        slot: usize,
        output: &wgpu::TextureView,
        gtao: bool,
    ) -> u32 {
        let cx = PostCx {
            graph: self,
            passes,
            persistent,
            slot,
            output,
            gtao,
        };
        let draws = self
            .bindings
            .as_ref()
            .expect("prepared")
            .encode(encoder, &cx, pass);
        debug_assert!(draws.is_some(), "no post family encodes {pass:?}");
        draws.unwrap_or(0)
    }
}

/// The graph target of planar reflection slot `k` (0: `reflection`, 1: `reflection.1`) with `suffix`.
pub(crate) fn reflection_target(k: u8, suffix: &str) -> &'static str {
    match (k, suffix) {
        (0, "") => "reflection",
        (0, "-depth") => "reflection-depth",
        (0, "-msaa") => "reflection-msaa",
        (_, "") => "reflection.1",
        (_, "-depth") => "reflection.1-depth",
        _ => "reflection.1-msaa",
    }
}
