//! Where a shadow layer's static casters are kept while dynamic ones are
//! drawn over the live layer (`shadow_cache.rs`), and how they come back.
//!
//! A kept layer is a texture of the layer's values (and, for a local
//! light's layer, of its depth: a cascade's value is its depth). Saving
//! copies the live layer into it. Putting it back is the first draw of the
//! pass that draws the dynamic casters: one fullscreen triangle writing
//! the kept value and its depth into a cleared target, so the casters
//! after it are tested against the static ones and the layer ends as the
//! nearest of both.
//!
//! The layer used to come back as two texture copies (value and depth)
//! into a pass that then loaded both. On a tile-based GPU that is the
//! layer read and written three times before a caster is drawn, and it
//! cost more than drawing a terrain's casters again (M4 Pro, 2048 px
//! cascades: 0.7 ms a frame over four cascades). One draw in the pass
//! reads the kept texture once.

use crate::{DEPTH_FORMAT, LOCAL_SHADOW_FORMAT, ShadowTarget};

/// The cascades' format: their casters' depth in red.
pub(crate) const CASCADE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;

pub(crate) fn source() -> String {
    format!(
        "{}{}",
        crate::post::FULLSCREEN_WGSL,
        wgsl!("shadow_keep.wgsl")
    )
}

/// What puts a kept layer of one format back.
struct Put {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
}

/// The pipelines that put kept layers back: a cascade's and a local
/// light's. Made when a layer is first kept.
pub(crate) struct ShadowKeep {
    cascade: Put,
    local: Put,
}

/// One shadow layer's static casters: the layer's values, and its depth
/// where the values are not it.
pub(crate) struct KeptLayer {
    pub(crate) size: u32,
    color: wgpu::Texture,
    depth: Option<wgpu::Texture>,
    bind_group: wgpu::BindGroup,
}

impl ShadowKeep {
    pub(crate) fn new(device: &wgpu::Device) -> ShadowKeep {
        let module = crate::shader(device, "shadow keep", source());
        let put = |label: &str, entry: &str, format, with_depth: bool| {
            let mut entries = vec![crate::post::layout_entry(
                0,
                crate::post::float_texture(false),
            )];
            if with_depth {
                entries.push(crate::post::layout_entry(
                    1,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ));
            }
            let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(label),
                entries: &entries,
            });
            let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                primitive: Default::default(),
                // The kept depth as it is: the target was cleared.
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::Always),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
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
                cache: crate::pipeline_cache::of(device).as_ref(),
            });
            Put { pipeline, layout }
        };
        ShadowKeep {
            cascade: put("shadow keep", "fs_cascade", CASCADE_FORMAT, false),
            local: put("shadow keep local", "fs_local", LOCAL_SHADOW_FORMAT, true),
        }
    }

    /// A texture to keep a layer of `format` and side `size` in.
    pub(crate) fn layer(
        &self,
        device: &wgpu::Device,
        size: u32,
        format: wgpu::TextureFormat,
    ) -> KeptLayer {
        let usage = wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING;
        let texture = |label, format| {
            crate::renderer::target_texture(device, label, size, size, format, usage)
        };
        let color = texture("kept shadow", format);
        let local = format == LOCAL_SHADOW_FORMAT;
        let depth = local.then(|| texture("kept shadow depth", DEPTH_FORMAT));
        let view = |t: &wgpu::Texture| t.create_view(&Default::default());
        let views: Vec<wgpu::TextureView> =
            std::iter::once(&color).chain(&depth).map(view).collect();
        let entries: Vec<wgpu::BindGroupEntry> = views
            .iter()
            .enumerate()
            .map(|(i, v)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: wgpu::BindingResource::TextureView(v),
            })
            .collect();
        let put = if local { &self.local } else { &self.cascade };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("kept shadow"),
            layout: &put.layout,
            entries: &entries,
        });
        KeptLayer {
            size,
            color,
            depth,
            bind_group,
        }
    }

    /// Draws `layer` into a pass that cleared its targets, before the
    /// casters drawn over it. `targets`: the pass's ([`ShadowTarget`]).
    pub(crate) fn put_back(
        &self,
        pass: &mut wgpu::RenderPass,
        layer: &KeptLayer,
        targets: &[ShadowTarget],
    ) {
        let put = if targets == [ShadowTarget::Cascade] {
            &self.cascade
        } else {
            &self.local
        };
        pass.set_pipeline(&put.pipeline);
        pass.set_bind_group(0, &layer.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

impl KeptLayer {
    /// Whether the pass that draws what is saved keeps its depth for
    /// [`KeptLayer::save`].
    pub(crate) fn keeps_depth(&self) -> bool {
        self.depth.is_some()
    }

    /// Copies a live layer (the array, its index and the depth buffer its
    /// pass drew with) into the kept texture.
    pub(crate) fn save(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        (array, layer, live_depth): (&wgpu::Texture, u32, &wgpu::Texture),
    ) {
        let at = |texture, z| wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d { x: 0, y: 0, z },
            aspect: wgpu::TextureAspect::All,
        };
        let size = wgpu::Extent3d {
            width: self.size,
            height: self.size,
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_texture(at(array, layer), at(&self.color, 0), size);
        if let Some(depth) = &self.depth {
            encoder.copy_texture_to_texture(at(live_depth, 0), at(depth, 0), size);
        }
    }
}
