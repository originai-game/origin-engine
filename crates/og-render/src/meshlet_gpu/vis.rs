//! The visibility buffer (R1 batch 4; meshlet.rs states its steps on the
//! CPU, meshlet_raster.wgsl runs them): the camera's small clusters of
//! occluder batches, which the cull appends to the soft list, rasterised
//! in compute into a depth and an id word per pixel, their attributes
//! rebuilt per pixel into records, and each soft batch's pixels shaded by
//! one full-screen resolve in the camera's lit pass and prepass
//! (meshlet.wgsl `fs_meshlet_resolve`, `fs_prepass_meshlet_resolve`),
//! which write the rasteriser's depth so the hardware's clusters and the
//! resolved pixels meet by the depth test. The resolve's fragment stage
//! reads one storage buffer (each pixel's record slot, then the records)
//! and one uniform beside the lit pipelines' own: no texture, as the lit
//! fragment stage already samples 15 of WebGPU's 16.
//!
//! The resolve's pipelines are made inside error scopes. A device that
//! refuses one (the Pixel 10's PowerVR D-Series driver answers
//! VK_ERROR_UNKNOWN to the lit resolve, a valid pipeline its compiler does
//! not fit: tests/evidence/compat/android/) turns the visibility buffer off
//! for good, and its clusters draw on the hardware path as they do with
//! `RenderScene::visibility_buffer` off; until the device has answered
//! (the web answers on a later frame) the soft clusters are not resolved.
//! Drawing with a refused pipeline invalidated the whole frame's commands.

use super::{
    RECORD_ROWS, RECORD_ROWS_EXTRA, RasterParams, WORKGROUPS_PER_ROW, buffer, entry, storage,
};
use crate::Side;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

/// A pipeline's error scope, until the device answers it.
type Check = Pin<Box<dyn Future<Output = Option<wgpu::Error>>>>;

pub(crate) struct VisBuffer {
    layout: wgpu::BindGroupLayout,
    /// The wind field's lean the foliage sways by (`wind_field.rs`).
    wind_field: wgpu::TextureView,
    depth: wgpu::ComputePipeline,
    id: wgpu::ComputePipeline,
    setup: wgpu::ComputePipeline,
    params: wgpu::Buffer,
    /// The depth words, the id words and the records' count.
    words: wgpu::Buffer,
    /// Each pixel's record slot plus one, then the records.
    attrs: wgpu::Buffer,
    /// The resolve's (width, height, the records' first word).
    screen: wgpu::Buffer,
    pub(crate) resolve_layout: wgpu::BindGroupLayout,
    pub(crate) resolve_group: Option<wgpu::BindGroup>,
    /// The resolve's lit pipelines by `effects`, and its prepass one.
    pub(crate) lit: HashMap<bool, wgpu::RenderPipeline>,
    pub(crate) prepass: Option<wgpu::RenderPipeline>,
    /// The resolve's pipelines' error scopes the device has not answered.
    checks: Vec<Check>,
    /// What the device said when it refused one: the visibility buffer is
    /// off from then on.
    refused: Option<String>,
}

/// What [`VisBuffer::encode`] reads this frame.
pub(crate) struct VisFrame<'a> {
    pub view_proj: glam::Mat4,
    pub size: (u32, u32),
    /// The soft list's first item and capacity in clusters.
    pub list: (u32, u32),
    /// The cull's count of the soft list: its buffer and byte offset.
    pub count: (&'a wgpu::Buffer, u64),
    pub instances: &'a wgpu::Buffer,
    pub previous: &'a wgpu::Buffer,
    pub items: &'a wgpu::Buffer,
    pub clusters: &'a wgpu::Buffer,
    pub indices: &'a wgpu::Buffer,
    pub vertices: &'a wgpu::Buffer,
    pub lod_debug: bool,
    /// The frame's retro 3D (`Retro3d::uniform`): its vertices snap, its
    /// depth steps and its uv go affine here as in pbr.wgsl.
    pub stylise: [f32; 4],
    /// The foliage's wind this frame and the last drawn
    /// (`foliage::foliage_winds`).
    pub wind: [crate::foliage::FoliageWind; 2],
    /// The material style's surface is on (the records carry the local
    /// position's gradients its pattern reads, pbr_fs.wgsl) or a foliage
    /// mesh is in the pools (its canopy dome).
    pub extra: bool,
    pub max_binding: u64,
}

impl VisBuffer {
    pub fn new(device: &wgpu::Device, wind_field: wgpu::TextureView) -> VisBuffer {
        let c = wgpu::ShaderStages::COMPUTE;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet raster"),
            entries: &[
                super::uniform(0, c),
                storage(1, true, c),
                storage(2, true, c),
                storage(3, true, c),
                storage(4, true, c),
                storage(5, true, c),
                storage(6, false, c),
                storage(7, false, c),
                storage(8, true, c),
                // The wind field's lean (wind_field.rs; meshlet_sway.wgsl).
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: c,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("meshlet raster"),
            source: wgpu::ShaderSource::Wgsl(super::raster_source().into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("meshlet raster"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("meshlet raster"),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: crate::pipeline_cache::of(device).as_ref(),
            })
        };
        let f = wgpu::ShaderStages::FRAGMENT;
        let resolve_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("meshlet resolve"),
            entries: &[super::uniform(15, f), storage(16, true, f)],
        });
        let usage = wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST;
        VisBuffer {
            wind_field,
            depth: pipeline("raster_depth"),
            id: pipeline("raster_id"),
            setup: pipeline("setup"),
            layout,
            params: buffer(
                device,
                "meshlet raster params",
                std::mem::size_of::<RasterParams>() as u64,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            words: buffer(device, "meshlet visibility buffer", 0, usage),
            attrs: buffer(device, "meshlet visibility records", 0, usage),
            screen: buffer(
                device,
                "meshlet resolve screen",
                16,
                wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            ),
            resolve_layout,
            resolve_group: None,
            lit: HashMap::new(),
            prepass: None,
            checks: Vec::new(),
            refused: None,
        }
    }

    /// Whether the soft clusters can be resolved: no resolve pipeline was
    /// refused and the device has answered for every one made. Polls the
    /// answers first; a refusal forgets the pipelines and is logged once.
    pub fn usable(&mut self) -> bool {
        let mut cx = Context::from_waker(Waker::noop());
        let mut refused = None;
        self.checks.retain_mut(|c| match c.as_mut().poll(&mut cx) {
            Poll::Ready(Some(e)) => {
                refused.get_or_insert_with(|| e.to_string());
                false
            }
            Poll::Ready(None) => false,
            Poll::Pending => true,
        });
        if let Some(e) = refused {
            log::warn!(
                "og-render: the device refused the visibility buffer's resolve; its clusters draw on the hardware path: {e}"
            );
            self.checks.clear();
            self.clear_pipelines();
            self.refused = Some(e);
        }
        self.ready()
    }

    /// No resolve pipeline refused and every one answered for, as last
    /// polled: the draw resolves the soft clusters only then.
    pub fn ready(&self) -> bool {
        self.refused.is_none() && self.checks.is_empty()
    }

    /// Rasterises the frame's soft list into `encoder` and makes the
    /// resolve's group.
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        f: VisFrame<'_>,
    ) {
        let (w, h) = (f.size.0.max(1), f.size.1.max(1));
        let pixels = w as u64 * h as u64;
        // The slots, rounded up to a vec4, then at most a record a pixel
        // within the binding limit.
        let base = pixels.div_ceil(4) * 4;
        // The extra records' four more rows only when the frame reads
        // them (fewer records fit the binding).
        let rows = if f.extra || f.stylise[2] > 0.0 {
            RECORD_ROWS_EXTRA
        } else {
            RECORD_ROWS
        };
        let records = (pixels.min((f.max_binding / 4).saturating_sub(base) / (rows * 4))).max(1);
        let attrs = (base + records * rows * 4) * 4;
        let words = (2 * pixels + 1) * 4;
        if words > self.words.size() {
            self.words = buffer(
                device,
                "meshlet visibility buffer",
                words,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            self.resolve_group = None;
        }
        if attrs > self.attrs.size() {
            self.attrs = buffer(
                device,
                "meshlet visibility records",
                attrs,
                wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            );
            self.resolve_group = None;
        }
        queue.write_buffer(
            &self.screen,
            0,
            bytemuck::cast_slice(&[w, h, base as u32, rows as u32]),
        );
        let params = RasterParams {
            view_proj: f.view_proj.to_cols_array_2d(),
            size: [w, h, records as u32, f.lod_debug as u32],
            soft: [f.list.0, f.list.1, 0, rows as u32],
            stylise: f.stylise,
            wind: f.wind,
        };
        queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
        // The cull's count into `soft.z`.
        encoder.copy_buffer_to_buffer(f.count.0, f.count.1, &self.params, 64 + 16 + 8, 4);
        encoder.clear_buffer(&self.words, 0, Some(words));
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("meshlet raster"),
            layout: &self.layout,
            entries: &[
                entry(0, &self.params),
                entry(1, f.instances),
                entry(2, f.items),
                entry(3, f.clusters),
                entry(4, f.indices),
                entry(5, f.vertices),
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.words,
                        offset: 0,
                        size: wgpu::BufferSize::new(words),
                    }),
                },
                entry(7, &self.attrs),
                entry(8, f.previous),
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: wgpu::BindingResource::TextureView(&self.wind_field),
                },
            ],
        });
        let clusters = f.list.1;
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("meshlet raster"),
            timestamp_writes: None,
        });
        pass.set_bind_group(0, &group, &[]);
        for pipeline in [&self.depth, &self.id] {
            pass.set_pipeline(pipeline);
            pass.dispatch_workgroups(
                clusters.min(WORKGROUPS_PER_ROW),
                clusters.div_ceil(WORKGROUPS_PER_ROW),
                1,
            );
        }
        pass.set_pipeline(&self.setup);
        pass.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        drop(pass);
        if self.resolve_group.is_none() {
            self.resolve_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("meshlet resolve"),
                layout: &self.resolve_layout,
                entries: &[entry(15, &self.screen), entry(16, &self.attrs)],
            }));
        }
    }

    /// The resolve's pipelines for the soft batches of `effects` (the lit
    /// pass's) and, with `prepass`, its prepass pipeline: groups 0 to 2
    /// the lit pipelines', 3 the resolve's.
    pub fn ensure_pipelines(
        &mut self,
        device: &wgpu::Device,
        layouts: [&wgpu::BindGroupLayout; 3],
        module: &wgpu::ShaderModule,
        effects: impl IntoIterator<Item = bool>,
        prepass: bool,
        samples: u32,
    ) {
        if self.refused.is_some() {
            return;
        }
        // Inner first: a validation error, else the driver's own (internal).
        let internal = device.push_error_scope(wgpu::ErrorFilter::Internal);
        let validation = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("meshlet resolve"),
            bind_group_layouts: &[
                Some(layouts[0]),
                Some(layouts[1]),
                Some(layouts[2]),
                Some(&self.resolve_layout),
            ],
            immediate_size: 0,
        });
        for e in effects {
            self.lit.entry(e).or_insert_with(|| {
                crate::hdr_pipeline_with(
                    device,
                    &[("EFFECTS", e as u32 as f64), ("ANALYTIC", 1.0)],
                    crate::HdrPipeline {
                        alpha_to_coverage: false,
                        label: "meshlet resolve",
                        layout: &layout,
                        module,
                        vs: "vs_meshlet_resolve",
                        fs: "fs_meshlet_resolve",
                        buffers: &[],
                        cull: None,
                        depth_write: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        blend: None,
                        samples,
                    },
                )
            });
        }
        if prepass && self.prepass.is_none() {
            self.prepass = Some(crate::prepass_pipeline(
                device,
                (&layout, &layout),
                module,
                Side::Double,
                false,
                None,
                Some(("vs_meshlet_resolve", "fs_prepass_meshlet_resolve", true)),
            ));
        }
        self.checks.push(Box::pin(validation.pop()));
        self.checks.push(Box::pin(internal.pop()));
        // Native devices answer at once: a refusal is known before the draw.
        let _ = self.usable();
    }

    /// Forgets the resolve's pipelines (the lit module changed).
    pub fn clear_pipelines(&mut self) {
        self.lit.clear();
        self.prepass = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_resolve_pipeline_the_device_refuses_turns_the_visibility_buffer_off() {
        // The backend a renderer picks (RenderDesc's default).
        let instance = crate::RenderDesc::default().backend.instance();
        let adapter = pollster::block_on(instance.request_adapter(&Default::default()))
            .expect("a GPU adapter (og-render's tests draw on the GPU)");
        let (device, _queue) =
            pollster::block_on(adapter.request_device(&Default::default())).expect("a device");
        let wind = device
            .create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba16Float,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        let mut vis = VisBuffer::new(&device, wind);
        assert!(vis.usable(), "nothing asked of the device yet");
        let empty = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        });
        // A lit module without the resolve's entry points: the device
        // refuses its pipelines as the PowerVR driver refuses the real one.
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                "@vertex fn vs() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }".into(),
            ),
        });
        vis.ensure_pipelines(
            &device,
            [&empty, &empty, &empty],
            &module,
            [false, true],
            true,
            1,
        );
        // Native devices answer at once: no pipeline is left to draw with.
        assert!(!vis.usable());
        assert!(vis.lit.is_empty() && vis.prepass.is_none());
        assert!(
            vis.refused
                .as_deref()
                .is_some_and(|e| e.contains("fs_meshlet_resolve") || e.contains("entry")),
            "{:?}",
            vis.refused.as_deref()
        );
        // Asked again (a later frame's soft list), it makes none.
        vis.ensure_pipelines(
            &device,
            [&empty, &empty, &empty],
            &module,
            [false],
            false,
            1,
        );
        assert!(vis.lit.is_empty() && !vis.usable());
    }
}
