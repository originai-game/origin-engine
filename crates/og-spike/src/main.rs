//! Native headless run: renders the reference scene offscreen and writes a PNG,
//! reporting frame time. This is the executor path (lavapipe or a GPU).
//!
//!   og-spike --out frame.png [--width 1280 --height 720] [--frames 60] [--count 25] model.glb...

use std::time::Instant;

fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let mut out = String::from("frame.png");
    let (mut width, mut height, mut frames, mut count) = (1280u32, 720u32, 60u32, 25u32);
    let mut debug = 0.0f32;
    let mut sphere = false;
    let mut moving = false;
    let mut models = Vec::new();
    while let Some(a) = args.next() {
        match a.as_str() {
            "--out" => out = args.next().expect("--out path"),
            "--width" => width = args.next().and_then(|v| v.parse().ok()).expect("--width n"),
            "--height" => height = args.next().and_then(|v| v.parse().ok()).expect("--height n"),
            "--frames" => frames = args.next().and_then(|v| v.parse().ok()).expect("--frames n"),
            "--count" => count = args.next().and_then(|v| v.parse().ok()).expect("--count n"),
            "--debug" => debug = args.next().and_then(|v| v.parse().ok()).expect("--debug n"),
            "--sphere" => sphere = true,
            "--moving" => moving = true,
            path => models.push(path.to_string()),
        }
    }
    let loaded: Vec<og_spike::scene::Model> = models
        .iter()
        .map(|p| {
            let bytes = std::fs::read(p).unwrap_or_else(|e| panic!("read {p}: {e}"));
            og_spike::scene::load_glb(&bytes).unwrap_or_else(|e| panic!("{p}: {e}"))
        })
        .collect();
    let loaded = if sphere { vec![og_spike::scene::sphere(16, 12)] } else { loaded };
    let scene = og_spike::reference_scene(loaded, count, if sphere { 1.2 } else { 3.0 });

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..wgpu::InstanceDescriptor::new_without_display_handle() });
    let t0 = Instant::now();
    let mut renderer = pollster::block_on(og_spike::Renderer::new(&instance, None, width, height, None, &scene)).expect("renderer");
    let setup_ms = t0.elapsed().as_secs_f64() * 1000.0;
    renderer.debug_view = debug;
    renderer.moving = moving;

    let device = renderer.device.clone();
    let queue = renderer.queue.clone();
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("target"),
        size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: renderer.output_format(),
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());

    // Warm up (pipeline compilation, IBL submit), then time `frames` frames with a GPU wait at the end.
    renderer.render(&target_view, 0.0);
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let t1 = Instant::now();
    for i in 0..frames {
        renderer.render(&target_view, i as f32 / 60.0);
    }
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let frame_ms = t1.elapsed().as_secs_f64() * 1000.0 / frames as f64;

    // Read back the last frame.
    let bytes_per_row = (width * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some("readback"), size: (bytes_per_row * height) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo { texture: &target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(bytes_per_row), rows_per_image: Some(height) } },
        wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
    );
    queue.submit(Some(encoder.finish()));
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |r| r.expect("map"));
    device.poll(wgpu::PollType::wait_indefinitely()).expect("poll");
    let data = slice.get_mapped_range().expect("mapped range");
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);
    for row in 0..height {
        let start = (row * bytes_per_row) as usize;
        pixels.extend_from_slice(&data[start..start + (width * 4) as usize]);
    }
    drop(data);
    buffer.unmap();
    image::save_buffer(&out, &pixels, width, height, image::ColorType::Rgba8).expect("png");

    println!(
        "og-spike: {}x{} triangles={} draws={} setup={:.1}ms frame={:.3}ms ({:.0} fps) -> {}",
        width,
        height,
        renderer.triangles,
        renderer.draw_calls(),
        setup_ms,
        frame_ms,
        1000.0 / frame_ms,
        out
    );
}
