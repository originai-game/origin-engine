//! Shared frames (`og_render::shared_frames`): a host on the other end of
//! the socket, playing Origin Desktop's part with a device of its own,
//! sees in each ready slot exactly the frame the renderer drew offscreen;
//! a resize it asks for reallocates the slots at that size; a renderer
//! whose host is not there yet draws on and links once it listens.

use glam::{Mat4, Vec3};
use og_render::shared_frames::{
    FrameLink, Msg, OUTPUT_FORMAT, Presented, SLOTS, UnixListener, UnixStream, import_slot, recv,
    recv_hello, send,
};
use og_render::{Camera, Material, RenderDesc, RenderScene, Renderer, shapes};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn ball(colour: [f32; 4]) -> RenderScene {
    let mut scene = RenderScene::new();
    let ball = scene.add_mesh(shapes::sphere(32, 16));
    let m = scene.add_material(Material {
        base_color: colour,
        ..Default::default()
    });
    scene.add_instance(ball, m, Mat4::IDENTITY);
    scene.camera = Camera {
        position: Vec3::new(0.4, 0.3, 3.0),
        target: Vec3::ZERO,
        ..Default::default()
    };
    scene
}

fn renderer(
    width: u32,
    height: u32,
    output_format: Option<og_render::wgpu::TextureFormat>,
) -> Option<Renderer> {
    Renderer::new_blocking(&RenderDesc {
        width,
        height,
        output_format,
        ..Default::default()
    })
    .ok()
}

/// A texture's pixels as RGBA8 rows, top first (BGRA reordered).
fn read_rgba(
    device: &og_render::wgpu::Device,
    queue: &og_render::wgpu::Queue,
    texture: &og_render::wgpu::Texture,
) -> Vec<u8> {
    use og_render::wgpu;
    let (w, h) = (texture.width(), texture.height());
    let row = w * 4;
    let padded = row.div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (padded * h) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: Some(h),
            },
        },
        texture.size(),
    );
    queue.submit([encoder.finish()]);
    buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let data = buffer.slice(..).get_mapped_range().expect("mapped");
    let mut pixels: Vec<u8> = (0..h)
        .flat_map(|y| data[(y * padded) as usize..(y * padded + row) as usize].to_vec())
        .collect();
    if matches!(
        texture.format(),
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    ) {
        for px in pixels.as_chunks_mut::<4>().0 {
            px.swap(0, 2);
        }
    }
    pixels
}

fn socket(name: &str) -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("og-frames-test-{name}-{}.sock", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

/// What the host saw in a ready slot.
struct Seen {
    generation: u32,
    slot: u32,
    seq: u64,
    width: u32,
    pixels: Vec<u8>,
}

/// The host: accepts the engine, imports each generation's slots on its own
/// device, reads every ready slot back (keeping it, so the engine runs out)
/// and, after `resize_after` frames, asks for `resize`.
fn host(
    listener: UnixListener,
    resize_after: usize,
    resize: (u32, u32),
    frames: usize,
    seen: mpsc::Sender<Seen>,
) {
    let host = renderer(16, 16, None).expect("the host's device");
    let (device, queue) = (host.device(), host.queue());
    let (stream, _) = listener.accept().unwrap();
    let mut reader: UnixStream = stream.try_clone().unwrap();
    let mut writer = stream.try_clone().unwrap();
    let import = |stream: &UnixStream| {
        let (hello, handles) = recv_hello(stream).unwrap();
        let slots: Vec<_> = handles
            .iter()
            .map(|h| import_slot(device, &hello, h).unwrap())
            .collect();
        (hello.width, slots)
    };
    let (mut width, mut slots) = import(&stream);
    let mut generation = 0;
    let mut n = 0;
    while n < frames {
        if n == resize_after {
            send(
                &mut writer,
                &Msg::Resize {
                    width: resize.0,
                    height: resize.1,
                },
            )
            .unwrap();
        }
        match recv(&mut reader).unwrap() {
            Msg::Ready {
                generation: g,
                slot,
                seq,
                ..
            } => {
                assert_eq!(g, generation, "a Ready of another generation");
                let pixels = read_rgba(device, queue, &slots[slot as usize]);
                // Released at once: this host holds nothing.
                send(
                    &mut writer,
                    &Msg::Release {
                        generation: g,
                        slot,
                    },
                )
                .unwrap();
                n += 1;
                let _ = seen.send(Seen {
                    generation: g,
                    slot,
                    seq,
                    width,
                    pixels,
                });
            }
            Msg::Rehello => {
                (width, slots) = import(&stream);
                generation += 1;
            }
            other => panic!("the engine sent {other:?}"),
        }
    }
}

#[test]
fn a_host_sees_each_frame_exactly_as_drawn_and_its_resize_reallocates_the_slots() {
    let Some(mut engine) = renderer(96, 64, Some(OUTPUT_FORMAT)) else {
        eprintln!("no adapter: skipped");
        return;
    };
    let path = socket("frames");
    let listener = UnixListener::bind(&path).unwrap();
    let (tx, seen) = mpsc::channel();
    let h = std::thread::spawn(move || host(listener, 2, (80, 48), 6, tx));
    let mut link = FrameLink::connect(&path, &engine);
    let scenes = [ball([0.9, 0.2, 0.1, 1.0]), ball([0.1, 0.3, 0.9, 1.0])];
    // What the engine drew for each frame it sent, by sequence.
    let mut drawn = std::collections::BTreeMap::new();
    let mut got = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut frame = 0usize;
    while got.len() < 6 && Instant::now() < deadline {
        if let Some((w, h)) = link.take_resize() {
            engine.resize(w, h);
        }
        let scene = &scenes[frame % 2];
        engine
            .render(scene, &og_render::RenderTarget::Offscreen)
            .unwrap();
        if let Presented::Slot { seq, .. } = link.present(&engine) {
            drawn.insert(
                seq,
                (engine.size(), engine.read_offscreen().unwrap().pixels),
            );
            frame += 1;
        }
        while let Ok(s) = seen.try_recv() {
            got.push(s);
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    h.join().unwrap();
    assert_eq!(got.len(), 6, "six frames reached the host");
    for s in &got {
        let ((w, _), pixels) = &drawn[&s.seq];
        assert_eq!(s.width, *w, "frame {} at its generation's size", s.seq);
        assert!(
            *pixels == s.pixels,
            "frame {} (slot {}) as drawn",
            s.seq,
            s.slot
        );
    }
    // The two scenes differ, so a slot holding another frame would fail.
    assert_ne!(got[0].pixels, got[1].pixels);
    assert_eq!(got[0].generation, 0);
    assert_eq!((got[0].width, got.last().unwrap().width), (96, 80));
    assert_eq!(
        got.last().unwrap().generation,
        1,
        "the resize made generation 1"
    );
    assert_eq!(engine.size(), (80, 48));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn a_host_that_keeps_every_slot_never_stalls_the_engine() {
    let Some(mut engine) = renderer(32, 32, Some(OUTPUT_FORMAT)) else {
        eprintln!("no adapter: skipped");
        return;
    };
    let path = socket("busy");
    let listener = UnixListener::bind(&path).unwrap();
    let (tx, rx) = mpsc::channel();
    let p = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (hello, _handles) = recv_hello(&stream).unwrap();
        tx.send(hello.slots).unwrap();
        // Holds the connection and every slot until the engine is done.
        let mut reader = stream;
        while recv(&mut reader).is_ok() {}
    });
    let mut link = FrameLink::connect(&path, &engine);
    let scene = ball([0.5, 0.5, 0.5, 1.0]);
    let mut outcomes = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(20);
    while outcomes.iter().filter(|o| **o == Presented::Busy).count() < 3
        && Instant::now() < deadline
    {
        engine
            .render(&scene, &og_render::RenderTarget::Offscreen)
            .unwrap();
        outcomes.push(link.present(&engine));
        std::thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(rx.recv().unwrap(), SLOTS);
    let shown = outcomes
        .iter()
        .filter(|o| matches!(o, Presented::Slot { .. }))
        .count();
    assert_eq!(
        shown, SLOTS as usize,
        "every slot once, then none: {outcomes:?}"
    );
    assert_eq!(outcomes.last(), Some(&Presented::Busy));
    drop(link);
    p.join().unwrap();
    let _ = std::fs::remove_file(&path);
}

#[test]
fn without_a_host_the_renderer_draws_on_and_links_once_one_listens() {
    let Some(engine) = renderer(32, 32, Some(OUTPUT_FORMAT)) else {
        eprintln!("no adapter: skipped");
        return;
    };
    let path = socket("later");
    let mut link = FrameLink::connect(&path, &engine);
    let scene = ball([0.5, 0.5, 0.5, 1.0]);
    let mut engine = engine;
    for _ in 0..5 {
        engine
            .render(&scene, &og_render::RenderTarget::Offscreen)
            .unwrap();
        assert_eq!(link.present(&engine), Presented::NoHost);
    }
    let listener = UnixListener::bind(&path).unwrap();
    let p = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let (hello, _handles) = recv_hello(&stream).unwrap();
        let mut reader = stream;
        let ready = recv(&mut reader).unwrap();
        (hello.width, ready)
    });
    let deadline = Instant::now() + Duration::from_secs(20);
    while !p.is_finished() && Instant::now() < deadline {
        engine
            .render(&scene, &og_render::RenderTarget::Offscreen)
            .unwrap();
        link.present(&engine);
        std::thread::sleep(Duration::from_millis(10));
    }
    let (width, ready) = p.join().unwrap();
    assert_eq!(width, 32);
    assert!(
        matches!(
            ready,
            Msg::Ready {
                generation: 0,
                seq: 1,
                ..
            }
        ),
        "{ready:?}"
    );
    let _ = std::fs::remove_file(&path);
}

/// The wire is Origin Desktop's `shared-frame`: serde's default JSON of the
/// same enum, a little-endian length before each body.
#[test]
fn messages_are_shared_frame_s_json_with_a_length() {
    let mut out = Vec::new();
    send(
        &mut out,
        &Msg::Release {
            generation: 2,
            slot: 3,
        },
    )
    .unwrap();
    send(&mut out, &Msg::Rehello).unwrap();
    let first = br#"{"Release":{"generation":2,"slot":3}}"#;
    assert_eq!(&out[..4], &(first.len() as u32).to_le_bytes());
    assert_eq!(&out[4..4 + first.len()], first);
    assert_eq!(&out[8 + first.len()..], br#""Rehello""#);
    let mut input = &out[..];
    assert_eq!(
        recv(&mut input).unwrap(),
        Msg::Release {
            generation: 2,
            slot: 3
        }
    );
    assert_eq!(recv(&mut input).unwrap(), Msg::Rehello);
}
