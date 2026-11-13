use super::*;

/// The height-correlated Smith visibility `G2 / (4 n.l n.v)`, built
/// from the masking function's Λ (Heitz 2014, equations 72 and 99)
/// in place of the closed form ggx.wgsl evaluates.
fn v_smith(n_dot_v: f64, n_dot_l: f64, roughness: f64) -> f64 {
    let a2 = roughness.powi(4);
    let lambda = |c: f64| 0.5 * ((1.0 + a2 * (1.0 - c * c) / (c * c)).sqrt() - 1.0);
    1.0 / ((1.0 + lambda(n_dot_v) + lambda(n_dot_l)) * 4.0 * n_dot_v * n_dot_l)
}

fn device() -> (wgpu::Device, wgpu::Queue) {
    let instance = crate::RenderDesc::default().backend.instance();
    let adapter =
        pollster::block_on(instance.request_adapter(&Default::default())).expect("a GPU adapter");
    pollster::block_on(adapter.request_device(&Default::default())).expect("a device")
}

/// The texels (row by row, four floats each) of a `width` x `height`
/// target of `format` after `draw`'s one pass into it.
fn drawn(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
    (width, height): (u32, u32),
    draw: impl FnOnce(&mut wgpu::RenderPass),
) -> Vec<[f32; 4]> {
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let texel = format.block_copy_size(None).expect("a colour format") as usize;
    let row = (width as usize * texel).next_multiple_of(256);
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (row * height as usize) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.create_view(&Default::default()),
                depth_slice: None,
                resolve_target: None,
                ops: Default::default(),
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        draw(&mut pass);
    }
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row as u32),
                rows_per_image: None,
            },
        },
        target.size(),
    );
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let bytes = staging.slice(..).get_mapped_range().unwrap();
    let float = |b: &[u8]| match texel {
        8 => crate::probes::f16_to_f32(u16::from_le_bytes([b[0], b[1]])),
        _ => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
    };
    bytes
        .chunks(row)
        .flat_map(|r| r[..width as usize * texel].chunks(texel))
        .map(|t| std::array::from_fn(|c| float(&t[c * texel / 4..])))
        .collect()
}

/// n.v, n.l and the roughness the visibility is checked at: the
/// issue's worst cases (grazing and rough, V above 1), then the rest
/// of the range.
const POINTS: [[f64; 3]; 8] = [
    [0.1, 0.5, 0.78],
    [0.05, 0.2, 0.3],
    [0.05, 0.2, 0.78],
    [0.2, 0.5, 0.5],
    [0.5, 0.9, 0.78],
    [1.0, 0.5, 1.0],
    [0.9, 0.3, 0.2],
    [0.6, 0.8, 0.1],
];

/// Direct light's specular visibility is the height-correlated Smith
/// one for alpha = roughness squared, with no remap: read out of the
/// shipped `brdf` and `physical_brdf` (a white metal's specular over
/// its D and n.l) and compared with [`v_smith`]. The separable G with
/// Karis's k that this replaced gives 0.60 of it at the first point
/// and 0.25 to 0.89 over the others. The anisotropic branch with one
/// alpha both ways is the same lobe, so the specular does not jump
/// as the anisotropy reaches 0 (capped at 1, it gave 1 for the first
/// four points' 1.3 to 15.9).
#[test]
#[ignore = "requires GPU"]
fn direct_light_has_the_height_correlated_smith_visibility() {
    let (device, queue) = device();
    let points: String = POINTS
        .iter()
        .map(|[v, l, r]| format!("vec3<f32>({v:?}, {l:?}, {r:?}),"))
        .collect();
    let count = POINTS.len();
    let source = format!(
            "{}
@vertex
fn ggx_test_vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {{
  return vec4<f32>(f32(i32(i & 1u) * 4 - 1), f32(i32(i & 2u) * 2 - 1), 0.0, 1.0);
}}

// Red: `brdf`'s visibility; green: `physical_brdf`'s; blue: its
// anisotropic branch's with the same alpha both ways.
@fragment
fn ggx_test_fs(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {{
  var points = array<vec3<f32>, {count}>({points});
  let q = points[u32(at.x)];
  let n = vec3<f32>(0.0, 0.0, 1.0);
  let v = vec3<f32>(sqrt(1.0 - q.x * q.x), 0.0, q.x);
  let l = vec3<f32>(0.0, sqrt(1.0 - q.y * q.y), q.y);
  let h = normalize(v + l);
  var s: Surface;
  s.metallic = 1.0;
  s.roughness = q.z;
  s.f0 = vec3<f32>(1.0);
  let a = q.z * q.z;
  let lobe = d_ggx(h.z, a) * q.y;
  var k: Lobes;
  let isotropic = physical_brdf(s, k, n, v, l).specular.x / lobe;
  k.anisotropy = 1.0;
  k.aniso_t = vec3<f32>(1.0, 0.0, 0.0);
  k.aniso_b = vec3<f32>(0.0, 1.0, 0.0);
  k.alpha_t = a;
  let stretched = physical_brdf(s, k, n, v, l).specular.x / (d_ggx_aniso(a, a, h.z, h.x, h.y) * q.y);
  return vec4<f32>(brdf(s, n, v, l).x / lobe, isotropic, stretched, 1.0);
}}
",
            crate::physical_source(&crate::hooks::default_hooks())
        );
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let format = wgpu::TextureFormat::Rgba32Float;
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("ggx_test_vs"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("ggx_test_fs"),
            compilation_options: Default::default(),
            targets: &[Some(format.into())],
        }),
        multiview_mask: None,
        cache: None,
    });
    // The sheen's albedo table, which `physical_brdf` reads only
    // under a sheen: the two bindings the entry point keeps.
    let ibl = IblGenerator::new(&device);
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(&ibl.views.brdf_lut),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::Sampler(&ibl.views.sampler),
            },
        ],
    });
    let texels = drawn(&device, &queue, format, (count as u32, 1), |pass| {
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    });
    for (&[n_dot_v, n_dot_l, roughness], texel) in POINTS.iter().zip(&texels) {
        let expected = v_smith(n_dot_v, n_dot_l, roughness);
        for (name, got) in ["brdf", "physical_brdf", "physical_brdf, anisotropic"]
            .iter()
            .zip(texel)
        {
            assert!(
                (f64::from(*got) / expected - 1.0).abs() < 1e-4,
                "{name} at n.v {n_dot_v}, n.l {n_dot_l}, roughness {roughness}: \
                     visibility {got}, expected {expected}"
            );
        }
    }
}

/// The BRDF table is integrated with direct light's visibility:
/// `0.04 A + B` where the lit shaders sample it (n.v, roughness)
/// against issue 39's values for the height-correlated visibility
/// (65,536 samples, checked there against a hemisphere integral).
/// The separable G with k = r²/2 this replaced gives the third number,
/// 10 to 34 % lower.
#[test]
#[ignore = "requires GPU"]
fn the_brdf_table_is_integrated_with_direct_lights_visibility() {
    let (device, queue) = device();
    let ibl = IblGenerator::new(&device);
    let size = LUT_SIZE as usize;
    let table = drawn(&device, &queue, HDR_FORMAT, (LUT_SIZE, LUT_SIZE), |pass| {
        pass.set_pipeline(&ibl.pipelines[ibl.lut_pass.pipeline]);
        pass.set_bind_group(0, &ibl.lut_pass.group, &[0]);
        pass.draw(0..3, 0..1);
    });
    // As the sampler reads it: bilinear, the edges clamped.
    let sample = |n_dot_v: f64, roughness: f64| {
        let (x, y) = (n_dot_v * size as f64 - 0.5, roughness * size as f64 - 0.5);
        let at = |x: f64, y: f64| {
            let texel = table[y.clamp(0.0, size as f64 - 1.0) as usize * size
                + x.clamp(0.0, size as f64 - 1.0) as usize];
            0.04 * f64::from(texel[0]) + f64::from(texel[1])
        };
        let (fx, fy) = (x - x.floor(), y - y.floor());
        let (x, y) = (x.floor(), y.floor());
        (at(x, y) * (1.0 - fx) + at(x + 1.0, y) * fx) * (1.0 - fy)
            + (at(x, y + 1.0) * (1.0 - fx) + at(x + 1.0, y + 1.0) * fx) * fy
    };
    for (roughness, n_dot_v, expected, separable) in [
        (0.78, 0.05, 0.0927, 0.0636),
        (0.78, 0.1, 0.0790, 0.0567),
        (0.78, 0.2, 0.0599, 0.0459),
        (0.78, 0.5, 0.0327, 0.0283),
        (1.0, 0.1, 0.0533, 0.0353),
        (1.0, 0.5, 0.0209, 0.0187),
    ] {
        let got = sample(n_dot_v, roughness);
        assert!(
            (got / expected - 1.0).abs() < 0.03,
            "roughness {roughness}, n.v {n_dot_v}: 0.04 A + B is {got}, expected \
                 {expected} (the separable visibility's: {separable})"
        );
    }
}
