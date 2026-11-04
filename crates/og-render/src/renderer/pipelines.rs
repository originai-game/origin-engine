//! Bind group layouts and render pipeline construction.

use super::*;

pub(crate) fn uniform_entry(
    binding: u32,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(crate) fn texture_entry(
    binding: u32,
    filterable: bool,
    dimension: wgpu::TextureViewDimension,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable },
            view_dimension: dimension,
            multisampled: false,
        },
        count: None,
    }
}

pub(crate) fn sampler_entry(
    binding: u32,
    kind: wgpu::SamplerBindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(kind),
        count: None,
    }
}

pub(crate) fn tex2d(binding: u32) -> wgpu::BindGroupLayoutEntry {
    texture_entry(binding, true, wgpu::TextureViewDimension::D2)
}

/// The sRGB view format of a non-sRGB 8-bit output, when the backend can
/// view the texture (`surface`: a surface's) in it: the overlay blends
/// there in linear light, as on an sRGB output. `None` on an sRGB output,
/// or where reinterpretation is missing (GL: textures; GL and Android
/// Vulkan: surfaces), where the overlay shader encodes instead.
pub(crate) fn linear_view_format(
    adapter: &wgpu::Adapter,
    output: wgpu::TextureFormat,
    surface: bool,
) -> Option<wgpu::TextureFormat> {
    let flag = if surface {
        wgpu::DownlevelFlags::SURFACE_VIEW_FORMATS
    } else {
        wgpu::DownlevelFlags::VIEW_FORMATS
    };
    let srgb = output.add_srgb_suffix();
    (srgb != output && adapter.get_downlevel_capabilities().flags.contains(flag)).then_some(srgb)
}

pub(crate) fn target_texture(
    device: &wgpu::Device,
    label: &str,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    usage: wgpu::TextureUsages,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
}

pub(crate) fn solid_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    rgba: [u8; 4],
) -> wgpu::TextureView {
    device
        .create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some("default texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &rgba,
        )
        .create_view(&Default::default())
}

pub(crate) fn shader(device: &wgpu::Device, label: &str, source: String) -> wgpu::ShaderModule {
    device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(source.into()),
    })
}

/// The pipeline state shared by the passes that draw into the HDR target.
pub(crate) struct HdrPipeline<'a> {
    pub label: &'a str,
    pub layout: &'a wgpu::PipelineLayout,
    pub module: &'a wgpu::ShaderModule,
    pub vs: &'a str,
    pub fs: &'a str,
    pub buffers: &'a [Option<wgpu::VertexBufferLayout<'a>>],
    pub cull: Option<wgpu::Face>,
    pub depth_write: bool,
    pub depth_compare: wgpu::CompareFunction,
    pub blend: Option<wgpu::BlendState>,
    pub samples: u32,
    /// Alpha to coverage (with samples above 1): the fragment's alpha masks its samples.
    pub alpha_to_coverage: bool,
}

/// An untonemapped sprite's blend (`SpriteFx::tone_mapped` false): its
/// colour over the scene as alpha blending does, and the HDR target's
/// alpha (1 under the sprites, `fs_cover`) taken towards 0 by its
/// coverage, so the tonemap knows how much of each pixel to leave
/// uncurved (post.wgsl `tonemapped`). A tone-mapped sprite or glyph drawn
/// over it takes the alpha back towards 1 (`ALPHA_BLENDING`).
pub(crate) const UNTONEMAPPED_BLEND: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::SrcAlpha,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
    alpha: wgpu::BlendComponent {
        src_factor: wgpu::BlendFactor::Zero,
        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
        operation: wgpu::BlendOperation::Add,
    },
};

/// Whether TAA's history was drawn from where the camera is now: the two
/// view-projections equal to a millionth of each entry, below what any
/// pixel could show. TAA then reads every pixel's own history texel
/// (taa.wgsl `still`), decided here rather than from the reprojection's
/// arithmetic on the GPU.
pub(crate) fn still_camera(prev: &Mat4, cur: &Mat4) -> bool {
    prev.to_cols_array()
        .iter()
        .zip(cur.to_cols_array())
        .all(|(p, c)| (p - c).abs() <= 1e-6 * c.abs().max(1.0))
}

pub(crate) fn hdr_pipeline(device: &wgpu::Device, p: HdrPipeline) -> wgpu::RenderPipeline {
    hdr_pipeline_with(device, &[], p)
}

/// [`hdr_pipeline`] made when a pass first sets it (deferred.rs), at
/// `samples` per pixel and again at the renderer's count after it changes
/// ([`Sampled::set_samples`]): `state` describes it over the layout and
/// module it is handed (its `samples` is the count it is made at).
pub(crate) fn deferred_hdr(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: impl AsModule + 'static,
    samples: u32,
    state: impl for<'a> Fn(&'a wgpu::PipelineLayout, &'a wgpu::ShaderModule) -> HdrPipeline<'a>
    + 'static,
) -> Sampled<wgpu::RenderPipeline> {
    let (device, layout) = (device.clone(), layout.clone());
    Sampled::new(samples, move |samples| {
        hdr_pipeline(
            &device,
            HdrPipeline {
                samples,
                ..state(&layout, module.module())
            },
        )
    })
}

/// The sprite pass's vertex buffers.
pub(crate) const SPRITE_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
    &[Some(SPRITE_LAYOUT)];
/// The ribbon pass's vertex buffers.
pub(crate) const RIBBON_BUFFERS: &[Option<wgpu::VertexBufferLayout<'static>>] =
    &[Some(ribbons::RIBBON_LAYOUT)];

/// [`hdr_pipeline`] with pipeline-overridable constants for both stages.
pub(crate) fn hdr_pipeline_with(
    device: &wgpu::Device,
    constants: &[(&str, f64)],
    p: HdrPipeline,
) -> wgpu::RenderPipeline {
    let options = wgpu::PipelineCompilationOptions {
        constants,
        ..Default::default()
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(p.label),
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
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(p.depth_write),
            depth_compare: Some(p.depth_compare),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: p.samples,
            alpha_to_coverage_enabled: p.alpha_to_coverage && p.samples > 1,
            ..Default::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: p.module,
            entry_point: Some(p.fs),
            compilation_options: options.clone(),
            targets: &[Some(wgpu::ColorTargetState {
                format: HDR_FORMAT,
                blend: p.blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// Whether `adapter` can render the HDR target and depth with 4x MSAA and resolve it.
pub(crate) fn supports_msaa(adapter: &wgpu::Adapter) -> bool {
    let hdr = adapter.get_texture_format_features(HDR_FORMAT).flags;
    let depth = adapter.get_texture_format_features(DEPTH_FORMAT).flags;
    hdr.sample_count_supported(MSAA_SAMPLES)
        && hdr.contains(wgpu::TextureFormatFeatureFlags::MULTISAMPLE_RESOLVE)
        && depth.sample_count_supported(MSAA_SAMPLES)
}

/// The clear colour of a pass for the scene's background (black under a sky).
pub(crate) fn background_clear(background: &Background) -> wgpu::Color {
    match background {
        Background::Color(c) => wgpu::Color {
            r: c.x as f64,
            g: c.y as f64,
            b: c.z as f64,
            a: 1.0,
        },
        Background::Sky | Background::Procedural { .. } => wgpu::Color::BLACK,
    }
}

pub(crate) fn cull_mode(side: Side) -> Option<wgpu::Face> {
    match side {
        Side::Front => Some(wgpu::Face::Back),
        Side::Back => Some(wgpu::Face::Front),
        Side::Double => None,
    }
}

/// The lit pipeline for one (side, blending, skinned) combination.
pub(crate) fn pbr_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    key: PbrKey,
    samples: u32,
) -> wgpu::RenderPipeline {
    lit_pipeline(device, layouts, module, key, samples, false)
}

/// [`pbr_pipeline`] for the standard or the physical lit shader (whose
/// vertex stages also hand on the model's scale).
pub(crate) fn lit_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    (side, transparent, skinned, effects): PbrKey,
    samples: u32,
    physical: bool,
) -> wgpu::RenderPipeline {
    let plain = [Some(VERTEX_LAYOUT), Some(INSTANCE_LAYOUT)];
    let skin = [
        Some(VERTEX_LAYOUT),
        Some(skin::SKINNED_INSTANCE_LAYOUT),
        Some(skin::SKIN_VERTEX_LAYOUT),
    ];
    hdr_pipeline_with(
        device,
        &[("EFFECTS", effects as u32 as f64)],
        HdrPipeline {
            // Opaque lit pixels write alpha 1 (every sample); alpha-to-coverage materials write their coverage.
            alpha_to_coverage: !transparent,
            label: match (physical, transparent) {
                (false, false) => "pbr",
                (false, true) => "pbr blended",
                (true, false) => "physical",
                (true, true) => "physical blended",
            },
            layout: if skinned { layouts.1 } else { layouts.0 },
            module,
            vs: match (physical, skinned) {
                (false, false) => "vs",
                (false, true) => "vs_skinned",
                (true, false) => "vs_physical",
                (true, true) => "vs_physical_skinned",
            },
            fs: "fs",
            buffers: if skinned { &skin } else { &plain },
            cull: cull_mode(side),
            // Blended surfaces test against the opaque depth but do not write it.
            depth_write: !transparent,
            depth_compare: wgpu::CompareFunction::Less,
            blend: transparent.then_some(wgpu::BlendState::ALPHA_BLENDING),
            samples,
        },
    )
}

/// The lean lit pipeline of a plain material ([`GpuMaterial::lean`]) for
/// (side, effects, derivatives, compact): opaque and unskinned by
/// construction, `pbr_lean.wgsl`'s entry points with `EFFECTS` as
/// [`lit_pipeline`]'s and without the discard unless the fragments take
/// derivatives, reading [`Vertex`] or the compact vertices.
pub(crate) fn lean_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    (side, effects, derivatives, compact): LeanKey,
    samples: u32,
) -> wgpu::RenderPipeline {
    let mut constants = vec![("EFFECTS", effects as u32 as f64)];
    if !derivatives {
        constants.push(("DISCARD", 0.0));
    }
    hdr_pipeline_with(
        device,
        &constants,
        HdrPipeline {
            alpha_to_coverage: false,
            label: if compact {
                "pbr lean compact"
            } else {
                "pbr lean"
            },
            layout,
            module,
            vs: if compact {
                "vs_lean_compact"
            } else {
                "vs_lean"
            },
            fs: "fs_lean",
            buffers: &[
                Some(if compact {
                    compact::LEAN_LAYOUT
                } else {
                    LEAN_VERTEX_LAYOUT
                }),
                Some(INSTANCE_LAYOUT),
            ],
            cull: cull_mode(side),
            depth_write: true,
            depth_compare: wgpu::CompareFunction::Less,
            blend: None,
            samples,
        },
    )
}

/// `PrevIn` in pbr.wgsl: the previous transform (motion vectors).
const PREV_INSTANCE_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: INSTANCE_STRIDE,
    step_mode: wgpu::VertexStepMode::Instance,
    attributes: &wgpu::vertex_attr_array![11 => Float32x3, 12 => Float32x3, 13 => Float32x3, 14 => Float32x3],
};

/// The depth-normal prepass pipeline for (side, skinned); `lean`: a plain
/// material's (unskinned, `pbr_lean.wgsl`'s entry points), as
/// `Some((derivatives, compact))` ([`Batch::derivatives`]: with them the
/// discard stays; `compact`: from the compact vertices).
pub(crate) fn prepass_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    side: Side,
    skinned: bool,
    lean: Option<(bool, bool)>,
    // The meshlet path's (vertex, fragment) entry points: no vertex
    // buffers (meshlet.wgsl pulls its vertices); with the third true, the
    // analytic gradients (pbr_fs.wgsl `ANALYTIC`: the visibility buffer's
    // resolve).
    meshlet: Option<(&str, &str, bool)>,
) -> wgpu::RenderPipeline {
    prepass_variant(device, layouts, module, side, skinned, lean, meshlet, None)
}

pub(crate) fn surface_prepass_pipeline(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    side: Side,
    array: bool,
) -> wgpu::RenderPipeline {
    prepass_variant(
        device,
        layouts,
        module,
        side,
        false,
        None,
        None,
        Some(if array {
            "vs_prepass_surface_array"
        } else {
            "vs_prepass_surface"
        }),
    )
}

#[allow(clippy::too_many_arguments)]
fn prepass_variant(
    device: &wgpu::Device,
    layouts: (&wgpu::PipelineLayout, &wgpu::PipelineLayout),
    module: &wgpu::ShaderModule,
    side: Side,
    skinned: bool,
    lean: Option<(bool, bool)>,
    meshlet: Option<(&str, &str, bool)>,
    surface: Option<&str>,
) -> wgpu::RenderPipeline {
    let (derivatives, compact) = lean.unwrap_or_default();
    let lean = lean.is_some();
    let plain = [
        Some(match (lean, compact) {
            (true, true) => compact::POSITION_NORMAL_LAYOUT,
            (true, false) => POSITION_NORMAL_LAYOUT,
            (false, _) => VERTEX_LAYOUT,
        }),
        Some(INSTANCE_LAYOUT),
        Some(PREV_INSTANCE_LAYOUT),
    ];
    let surface_vertices = [
        Some(VERTEX_LAYOUT),
        Some(INSTANCE_LAYOUT),
        Some(PREV_INSTANCE_LAYOUT),
        Some(wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![9 => Float32x3],
        }),
    ];
    let constants = [("DISCARD", 0.0)];
    let analytic = [("ANALYTIC", 1.0)];
    let options = wgpu::PipelineCompilationOptions {
        constants: if lean && !derivatives {
            &constants
        } else if meshlet.is_some_and(|m| m.2) {
            &analytic
        } else {
            &[]
        },
        ..Default::default()
    };
    let skin = [
        Some(VERTEX_LAYOUT),
        Some(skin::SKINNED_INSTANCE_LAYOUT),
        Some(skin::SKIN_VERTEX_LAYOUT),
    ];
    let target = |format| {
        Some(wgpu::ColorTargetState {
            format,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        })
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(if lean { "prepass lean" } else { "prepass" }),
        layout: Some(if skinned { layouts.1 } else { layouts.0 }),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(match (skinned, lean) {
                _ if let Some(vs) = surface => vs,
                _ if let Some((vs, _, _)) = meshlet => vs,
                (true, _) => "vs_prepass_skinned",
                (false, true) if compact => "vs_prepass_lean_compact",
                (false, true) => "vs_prepass_lean",
                (false, false) => "vs_prepass",
            }),
            compilation_options: options.clone(),
            // The meshlet entry pulls its vertices (meshlet.wgsl).
            buffers: match (meshlet.is_some(), skinned) {
                _ if surface.is_some() => &surface_vertices,
                (true, _) => &[],
                (false, true) => &skin,
                (false, false) => &plain,
            },
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: cull_mode(side),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(match meshlet {
                Some((_, fs, _)) => fs,
                None if lean && !skinned => "fs_prepass_lean",
                None => "fs_prepass",
            }),
            compilation_options: options,
            targets: &[
                target(post::GBUFFER_FORMAT),
                target(post::LINDEPTH_FORMAT),
                target(post::OUTLINE_FORMAT),
            ],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// The shadow pipeline for casters drawn with (side, skinned, dissolve);
/// `hooked`: with the module's material shader (its vertex offset and
/// surface colour, the material bound). Casters that are neither skinned,
/// dissolving nor hooked need only their positions: `shadow.wgsl`'s lean
/// entry points (the same depths), from the compact vertices when
/// `compact`.
pub(crate) fn shadow_pipeline(
    device: &wgpu::Device,
    layouts: &[wgpu::PipelineLayout; 3],
    module: &wgpu::ShaderModule,
    (side, skinned, dissolve): ShadowKey,
    hooked: bool,
    compact: bool,
) -> wgpu::RenderPipeline {
    let lean = !skinned && !dissolve && !hooked;
    let compact = lean && compact;
    let plain = [
        Some(match (lean, compact) {
            (_, true) => compact::POSITION_LAYOUT,
            (true, false) => POSITION_LAYOUT,
            (false, false) => VERTEX_LAYOUT,
        }),
        Some(INSTANCE_LAYOUT),
    ];
    let skin = [
        Some(VERTEX_LAYOUT),
        Some(skin::SKINNED_INSTANCE_LAYOUT),
        Some(skin::SKIN_VERTEX_LAYOUT),
    ];
    let layout = match (skinned, dissolve || hooked) {
        (true, _) => &layouts[2],
        (false, true) => &layouts[1],
        (false, false) => &layouts[0],
    };
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(match (skinned, hooked) {
                (true, true) => "vs_skinned_hooked",
                (true, false) => "vs_skinned",
                (false, true) => "vs_hooked",
                (false, false) if compact => "vs_lean_compact",
                (false, false) if lean => "vs_lean",
                (false, false) => "vs",
            }),
            compilation_options: Default::default(),
            buffers: if skinned { &skin } else { &plain },
        },
        // Front-sided casters draw back faces only: in a cascade the stored
        // depth is the far side of a closed caster, so its lit side never
        // compares against itself. Back-sided ones draw front faces;
        // double-sided ones (open surfaces) both. A local light's view is
        // mirrored (frame.rs `LIT_SIDE`), which turns this around: its
        // layers store a caster's lit side.
        primitive: wgpu::PrimitiveState {
            cull_mode: match side {
                Side::Front => Some(wgpu::Face::Front),
                Side::Back => Some(wgpu::Face::Back),
                Side::Double => None,
            },
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(match (hooked, dissolve) {
                (true, _) => "fs_hooked",
                (false, true) => "fs_dissolve",
                (false, false) if lean => "fs_lean",
                (false, false) => "fs",
            }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::RED,
            })],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}

/// [`shadow_pipeline`]'s state for the meshlet path's casters
/// (`meshlet_shadow.wgsl`), pulled with no vertex buffer: the lean entry
/// points, or with `full` the whole vertex and the alpha-tested and
/// dissolving casters' fragment.
pub(crate) fn meshlet_shadow_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    side: Side,
    full: bool,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadow meshlet"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(if full {
                "vs_meshlet"
            } else {
                "vs_meshlet_lean"
            }),
            compilation_options: Default::default(),
            buffers: &[],
        },
        // As `shadow_pipeline`'s casters.
        primitive: wgpu::PrimitiveState {
            cull_mode: match side {
                Side::Front => Some(wgpu::Face::Front),
                Side::Back => Some(wgpu::Face::Back),
                Side::Double => None,
            },
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        }),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(if full { "fs_dissolve" } else { "fs_lean" }),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R32Float,
                blend: None,
                write_mask: wgpu::ColorWrites::RED,
            })],
        }),
        multiview_mask: None,
        cache: crate::pipeline_cache::of(device).as_ref(),
    })
}
