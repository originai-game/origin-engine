//! Whether the Forward+ cluster build would be faster in compute on the
//! full tier (og-render's README, "What phase 2 owes"). Ignored: a
//! measurement, run by hand (`cargo test -p og-tests --release --test
//! cluster_compute -- --ignored --nocapture`), whose numbers are in
//! `tests/evidence/P4/perf/clusters-compute.md`.
//!
//! It times the cheapest compute build there could be for `lights-256`
//! (243 lights in view): one dispatch over the 16 x 9 x 24 clusters, each
//! testing every light's view-space sphere against its box and writing at
//! most 64 references, without the CPU build's importance order (so not
//! byte-identical, and a lower bound of a real one). Per frame it uploads
//! the light spheres, encodes the pass and submits it; the GPU time is the
//! wall time of submitting and waiting, less an empty submission's.

use og_render::wgpu;
use og_render::{RenderTarget, Renderer};
use og_tests::scenes;
use std::time::Instant;

const SHADER: &str = r#"
struct Params { grid: vec4<u32>, view: vec4<f32> };
@group(0) @binding(0) var<uniform> p: Params;
@group(0) @binding(1) var<storage, read> lights: array<vec4<f32>>;
@group(0) @binding(2) var<storage, read_write> table: array<u32>;
@group(0) @binding(3) var<storage, read_write> refs: array<u32>;

@compute @workgroup_size(4, 4, 4)
fn assign(@builtin(global_invocation_id) id: vec3<u32>) {
  if (any(id >= p.grid.xyz)) { return; }
  let c = id.x + p.grid.x * (id.y + p.grid.y * id.z);
  let near = p.view.z;
  let far = p.view.w;
  let z0 = near * pow(far / near, f32(id.z) / f32(p.grid.z));
  let z1 = near * pow(far / near, f32(id.z + 1u) / f32(p.grid.z));
  let tx0 = (2.0 * f32(id.x) / f32(p.grid.x) - 1.0) * p.view.x;
  let tx1 = (2.0 * f32(id.x + 1u) / f32(p.grid.x) - 1.0) * p.view.x;
  let ty0 = (1.0 - 2.0 * f32(id.y + 1u) / f32(p.grid.y)) * p.view.y;
  let ty1 = (1.0 - 2.0 * f32(id.y) / f32(p.grid.y)) * p.view.y;
  let lo = vec3<f32>(min(tx0 * z0, tx0 * z1), min(ty0 * z0, ty0 * z1), -z1);
  let hi = vec3<f32>(max(tx1 * z0, tx1 * z1), max(ty1 * z0, ty1 * z1), -z0);
  var n = 0u;
  for (var i = 0u; i < p.grid.w; i++) {
    let l = lights[i];
    let q = clamp(l.xyz, lo, hi);
    let d = l.xyz - q;
    if (dot(d, d) <= l.w * l.w && n < 64u) {
      refs[c * 64u + n] = i;
      n += 1u;
    }
  }
  table[c] = n;
}
"#;

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

#[test]
#[ignore]
fn cluster_build_in_compute_against_the_cpu_build() {
    let built = scenes::build("lights-256").expect("scene");
    let mut renderer = Renderer::new_blocking(&built.desc).expect("renderer");
    renderer
        .render(&built.scene, &RenderTarget::Offscreen)
        .expect("render");
    let (device, queue) = (renderer.device().clone(), renderer.queue().clone());
    let cam = &built.scene.camera;
    let view = glam::camera::rh::view::look_at_mat4(cam.position, cam.target, cam.up);
    let spheres: Vec<[f32; 4]> = built
        .scene
        .lights
        .iter()
        .map(|l| {
            let c = view.transform_point3(l.position);
            [c.x, c.y, c.z, l.range]
        })
        .collect();
    let aspect = built.desc.width as f32 / built.desc.height as f32;
    let half = (50f32.to_radians() / 2.0).tan();
    let params: [u32; 8] = [
        16,
        9,
        24,
        spheres.len() as u32,
        (half * aspect).to_bits(),
        half.to_bits(),
        cam.near.to_bits(),
        cam.far.to_bits(),
    ];
    let buffer = |label, size: u64, usage| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    };
    let clusters = 16 * 9 * 24u64;
    let uniform = buffer(
        "params",
        32,
        wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    );
    let lights = buffer(
        "lights",
        (spheres.len() as u64 * 16).max(16),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    );
    let table = buffer("table", clusters * 4, wgpu::BufferUsages::STORAGE);
    let refs = buffer("refs", clusters * 64 * 4, wgpu::BufferUsages::STORAGE);
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("clusters"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("clusters"),
        layout: None,
        module: &module,
        entry_point: Some("assign"),
        compilation_options: Default::default(),
        cache: None,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("clusters"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: lights.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: table.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: refs.as_entire_binding(),
            },
        ],
    });
    let wait = || {
        device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("poll");
    };
    let frame = |dispatch: bool| {
        queue.write_buffer(&uniform, 0, og_core::bytemuck::cast_slice(&params));
        queue.write_buffer(&lights, 0, og_core::bytemuck::cast_slice(&spheres));
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        if dispatch {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &group, &[]);
            pass.dispatch_workgroups(4, 3, 6);
        }
        queue.submit(Some(encoder.finish()));
    };
    for _ in 0..50 {
        frame(true);
        wait();
    }
    let (mut cpu, mut with, mut without) = (Vec::new(), Vec::new(), Vec::new());
    for _ in 0..500 {
        let t = Instant::now();
        frame(true);
        cpu.push(t.elapsed().as_secs_f64() * 1e6);
        wait();
        with.push(t.elapsed().as_secs_f64() * 1e6);
        let t = Instant::now();
        frame(false);
        wait();
        without.push(t.elapsed().as_secs_f64() * 1e6);
    }
    let (cpu, with, without) = (median(cpu), median(with), median(without));
    println!(
        "compute cluster build, {} lights on {}: encode and submit {cpu:.1} us; submit and wait {with:.1} us against {without:.1} us for an empty submission ({:.1} us of GPU and synchronisation)",
        spheres.len(),
        renderer.info().adapter,
        with - without
    );
}
