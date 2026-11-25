//! Hillaire (EGSR 2020 §5.3) Sky-View LUT (`sky_lut.wgsl`): precomputes the
//! physical atmosphere's 32-step spherical scattering into a 192×108
//! `Rgba16Float` texture whenever the atmosphere, sun, moon, or viewer
//! altitude changes, turning per-pixel 24–48 step raymarching into a single
//! bilinear lookup across the main sky pass, planar reflections, and live sky
//! cubemap faces (0 ms in steady state).

use crate::sky_physical::{SCATTER_VEC4S, sky_scatter_wgsl};

/// LUT resolution (Hillaire 2020 §5.3: 192×108).
pub(crate) const LUT_WIDTH: u32 = 192;
pub(crate) const LUT_HEIGHT: u32 = 108;

/// WGSL module for the Sky-View LUT compute pass.
pub(crate) fn source() -> String {
    format!(
        "{}{}{}",
        wgsl!("common.wgsl"),
        sky_scatter_wgsl("lut.scatter"),
        wgsl!("sky_lut.wgsl")
    )
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct SkyLutUniform {
    scatter: [[f32; 4]; SCATTER_VEC4S],
    sun_dir: [f32; 4],
    sun_light: [f32; 4],
    moon_dir: [f32; 4],
    moon_light: [f32; 4],
}

#[derive(Clone, PartialEq, Eq)]
struct LutKey {
    bits: Vec<u32>,
}

impl LutKey {
    fn of(u: &SkyLutUniform) -> Self {
        let mut bits: Vec<u32> =
            bytemuck::cast_slice::<SkyLutUniform, u32>(std::slice::from_ref(u)).to_vec();
        // Quantize viewer altitude (sun_dir[3], km) to 10 m steps so micro camera
        // bobbing never forces redundant LUT rebuilds.
        let h0_slot = SCATTER_VEC4S * 4 + 3;
        let h0_quant = (u.sun_dir[3] * 100.0).round() as i32;
        bits[h0_slot] = h0_quant as u32;
        Self { bits }
    }
}

struct SkyLutPass {
    pipeline: crate::deferred::Deferred<wgpu::ComputePipeline>,
    uniform: wgpu::Buffer,
    group: wgpu::BindGroup,
}

/// The renderer's Hillaire Sky-View LUT state: a 1×1 placeholder until a
/// physical sky is active on a compute-capable device.
pub(crate) struct SkyLut {
    pub(crate) view: wgpu::TextureView,
    _texture: wgpu::Texture,
    pass: Option<SkyLutPass>,
    last_key: Option<LutKey>,
    pending: bool,
    ready_altitude: f32,
}

impl SkyLut {
    pub(crate) fn placeholder(device: &wgpu::Device) -> Self {
        let texture = lut_texture(device, 1, 1);
        let view = texture.create_view(&Default::default());
        Self {
            view,
            _texture: texture,
            pass: None,
            last_key: None,
            pending: false,
            ready_altitude: -1.0,
        }
    }

    /// Viewer altitude (km) currently baked in the LUT, or `-1.0` when not yet
    /// populated (signaling `sky.wgsl` to fall back to analytic marching).
    pub(crate) fn baked_altitude_km(&self) -> f32 {
        self.ready_altitude
    }

    fn ensure(&mut self, device: &wgpu::Device) {
        if self.pass.is_some() {
            return;
        }
        let texture = lut_texture(device, LUT_WIDTH, LUT_HEIGHT);
        let view = texture.create_view(&Default::default());
        let uniform = crate::traffic::create_buffer(
            device,
            &wgpu::BufferDescriptor {
                label: Some("sky view lut params"),
                size: std::mem::size_of::<SkyLutUniform>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            },
        );
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky view lut"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba16Float,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky view lut"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
            ],
        });
        let pipeline = {
            let device = device.clone();
            crate::deferred::Deferred::anywhere(move || {
                crate::deferred::note(|| "sky view lut".into());
                let module = crate::gpu::shader_module(
                    &device,
                    wgpu::ShaderModuleDescriptor {
                        label: Some("sky view lut"),
                        source: wgpu::ShaderSource::Wgsl(source().into()),
                    },
                );
                let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("sky view lut"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                });
                crate::traffic::create_compute_pipeline(
                    &device,
                    &wgpu::ComputePipelineDescriptor {
                        label: Some("sky view lut"),
                        layout: Some(&pl),
                        module: &module,
                        entry_point: Some("cs_sky_lut"),
                        compilation_options: Default::default(),
                        cache: crate::pipeline_cache::of(&device).as_ref(),
                    },
                )
            })
        };
        crate::deferred::Start::start(&pipeline);
        self.view = view;
        self._texture = texture;
        self.pass = Some(SkyLutPass {
            pipeline,
            uniform,
            group,
        });
        self.last_key = None;
        self.ready_altitude = -1.0;
    }

    /// Updates the LUT uniform and schedules a dispatch when the physical sky
    /// parameters or quantized viewer altitude changed. Returns `true` when the
    /// LUT texture was newly allocated (caller must rebuild the sky group).
    pub(crate) fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        look: &crate::SkyLook,
        sun_dir: glam::Vec3,
        sun_color: glam::Vec3,
        viewer_y: f32,
    ) -> bool {
        let Some(air) = &look.physical else {
            self.ready_altitude = -1.0;
            return false;
        };
        let had = self.pass.is_some();
        self.ensure(device);
        let scatter = air.gpu();
        let sun = sun_dir.normalize_or(glam::Vec3::Y);
        let h0 = (scatter[3][2] + viewer_y * 0.001).max(0.001);
        let moon_on = look.moon_size.clamp(0.0, 20.0) > 0.0;
        let moon = if moon_on { look.moon_toward(sun) } else { sun };
        let moon_light = if moon_on {
            look.moon_light()
        } else {
            glam::Vec3::ZERO
        };
        let u = SkyLutUniform {
            scatter,
            sun_dir: [sun.x, sun.y, sun.z, h0],
            sun_light: [sun_color.x, sun_color.y, sun_color.z, look.brightness],
            moon_dir: [moon.x, moon.y, moon.z, if moon_on { 1.0 } else { 0.0 }],
            moon_light: [moon_light.x, moon_light.y, moon_light.z, 0.0],
        };
        let key = LutKey::of(&u);
        if !self.pending && self.last_key.as_ref() == Some(&key) {
            return !had;
        }
        if let Some(pass) = &self.pass {
            crate::traffic::write_buffer(queue, &pass.uniform, 0, bytemuck::bytes_of(&u));
            self.last_key = Some(key);
            self.pending = true;
            // Record the altitude once the pipeline is ready so `sky_params`
            // only enables the LUT on frames where `encode` actually fills it.
            if pass.pipeline.ready().is_some() {
                self.ready_altitude = h0;
            }
        }
        !had
    }

    /// Dispatches the LUT compute pass when dirty and pipeline is compiled.
    pub(crate) fn encode(&mut self, encoder: &mut crate::Encoder) {
        if !self.pending {
            return;
        }
        let Some(pass) = &self.pass else {
            return;
        };
        let Some(pipeline) = pass.pipeline.ready() else {
            return;
        };
        let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("sky view lut"),
            timestamp_writes: None,
        });
        compute.set_pipeline(pipeline);
        compute.set_bind_group(0, &pass.group, &[]);
        compute.dispatch_workgroups(LUT_WIDTH.div_ceil(8), LUT_HEIGHT.div_ceil(8), 1);
        self.pending = false;
    }
}

fn lut_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sky view lut"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba16Float,
        usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
        texture_binding_view_dimension: None,
    })
}
