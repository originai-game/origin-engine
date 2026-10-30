//! The renderer's frames in another process's window, through shared GPU
//! memory (`docs/design.md` §2.6, Shared frames): what Origin Desktop's
//! Preview draws while the built-in runner runs a game's engine.
//!
//! A host that draws frames listens on a socket and names it to the engine
//! process in [`SOCKET_ENV`]. [`FrameLink::connect`] connects to it (and
//! again whenever the host goes away and comes back), allocates [`SLOTS`]
//! shareable textures of the renderer's size on its device, sends their
//! memory with one `Hello`, and from then on [`FrameLink::present`] copies
//! each frame of the offscreen target into a free slot on the GPU and
//! announces it with [`Msg::Ready`] once the copy has finished. The host
//! samples the newest ready slot and gives older ones back with
//! [`Msg::Release`]; [`Msg::Resize`] asks for another size, which the host
//! applies with [`Renderer::resize`] before its next frame
//! ([`FrameLink::take_resize`]): the link then sends [`Msg::Rehello`] and a
//! new generation's `Hello`. No pixel crosses the CPU. The engine never
//! waits for the window: a frame with no free slot is not shown there, and
//! the handshake runs on the link's own thread.
//!
//! The memory is platform-specific, each in its module:
//! - macOS (`iosurface.rs`): BGRA8 IOSurfaces, their Mach ports sent through
//!   a one-shot bootstrap rendezvous the `Hello` names, to the socket's peer
//!   process only (the renderer must output [`OUTPUT_FORMAT`], BGRA, so the
//!   copy is a plain texture copy);
//! - Linux (`vulkan_fd.rs`): Vulkan images exported as opaque fds, passed
//!   with `SCM_RIGHTS` with the `Hello`;
//! - Windows (`d3d12.rs`): D3D12 committed resources shared as NT handles,
//!   duplicated into the socket's peer process; the socket is AF_UNIX.
//!
//! The wire is the contract of Origin Desktop's `shared-frame` crate
//! (origin-agent `desktop/crates/shared-frame`): every message a `u32`
//! little-endian length and a JSON body, [`Msg`] and each platform's
//! `Hello` serialised as serde does by default. The engine links no
//! platform code, so this is its own implementation of it; the
//! tests hold the consumer side ([`recv_hello`], [`import_slot`]) to it.

#[cfg(target_os = "linux")]
mod vulkan_fd;
#[cfg(target_os = "linux")]
pub use vulkan_fd::{Hello, SharedHandle, export_slots, import_slot, recv_hello, send_hello};

#[cfg(target_os = "macos")]
mod iosurface;
#[cfg(target_os = "macos")]
pub use iosurface::{Hello, SharedHandle, export_slots, import_slot, recv_hello, send_hello};

#[cfg(windows)]
mod d3d12;
#[cfg(windows)]
pub use d3d12::{Hello, SharedHandle, export_slots, import_slot, recv_hello, send_hello};

#[cfg(unix)]
pub use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(windows)]
pub use uds_windows::{UnixListener, UnixStream};

use crate::Renderer;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The environment variable naming the socket of a host that draws this
/// process's frames (Origin Runner sets it on the warm engine process).
pub const SOCKET_ENV: &str = "ORIGIN_FRAMES_SOCKET";

/// The slots' memory: 8-bit colour holding sRGB-encoded bytes, sampled as
/// UNORM by the host. RGBA where Vulkan and D3D12 share it, BGRA on macOS,
/// the IOSurface format Metal and the compositor prefer.
#[cfg(not(target_os = "macos"))]
pub const SLOT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
#[cfg(target_os = "macos")]
pub const SLOT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

/// The renderer output ([`crate::RenderDesc::output_format`]) a linked
/// renderer draws in: the slots' format seen as sRGB, so a frame copies
/// into a slot byte for byte. The probe's captures read either order as
/// RGBA.
#[cfg(not(target_os = "macos"))]
pub const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
#[cfg(target_os = "macos")]
pub const OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

/// The slots a generation has. The window holds up to four (the one shown,
/// two waiting out its CPU fence, the newest ready one); a fifth keeps the
/// engine from running out (Origin Desktop's measurements: with three, a
/// 60 fps engine delivered ~37 fps).
pub const SLOTS: u32 = 5;
/// The most slots a generation may have (the host keys its textures by
/// `generation * MAX_SLOTS + slot`).
pub const MAX_SLOTS: u32 = 16;
/// The sizes a host may ask for, each side (smaller and larger are clamped).
pub const MIN_SIDE: u32 = 16;
pub const MAX_SIZE: (u32, u32) = (7680, 4320);

/// How long the link waits before it tries a socket nobody listens on again.
const RETRY: Duration = Duration::from_millis(500);

/// Every message after a `Hello`, either way. Slots belong to a generation:
/// a connection's first `Hello` starts generation 0 and each `Rehello` the
/// next, so a release that crosses a reallocation is recognised as stale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    /// Engine: `slot` of `generation` holds frame `seq`, its copy finished
    /// `render_ms` after it was submitted, sent at `sent_us` (µs since the
    /// Unix epoch).
    Ready {
        generation: u32,
        slot: u32,
        seq: u64,
        render_ms: f32,
        sent_us: u64,
    },
    /// Host: `slot` of `generation` may be drawn into again.
    Release { generation: u32, slot: u32 },
    /// Host: draw at this size from now on (device pixels).
    Resize { width: u32, height: u32 },
    /// Engine: the next thing on the socket is a `Hello` starting the next
    /// generation; the previous generation's slots are no longer drawn into.
    Rehello,
}

/// One message on `stream`.
pub fn send(stream: &mut impl Write, msg: &Msg) -> std::io::Result<()> {
    write_frame(
        stream,
        &serde_json::to_vec(msg).map_err(std::io::Error::other)?,
    )
}

/// The next message on `stream`.
pub fn recv(stream: &mut impl Read) -> std::io::Result<Msg> {
    serde_json::from_slice(&read_frame(stream)?).map_err(std::io::Error::other)
}

fn write_frame(stream: &mut impl Write, body: &[u8]) -> std::io::Result<()> {
    let mut frame = Vec::with_capacity(4 + body.len());
    frame.extend_from_slice(&(body.len() as u32).to_le_bytes());
    frame.extend_from_slice(body);
    stream.write_all(&frame)
}

// The Hello's own read on Linux comes with its fds (`vulkan_fd::recv_hello`).
#[cfg_attr(target_os = "linux", allow(dead_code))]
fn read_frame(stream: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut len = [0u8; 4];
    stream.read_exact(&mut len)?;
    let len = u32::from_le_bytes(len) as usize;
    if len > 1 << 20 {
        return Err(std::io::Error::other(format!("a {len}-byte message")));
    }
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body)?;
    Ok(body)
}

/// The descriptor of a slot texture on either side: renderable through the
/// sRGB view, sampled and copied as UNORM.
fn slot_desc(width: u32, height: u32) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        label: Some("shared frame"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: SLOT_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[OUTPUT_FORMAT],
    }
}

fn now_us() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_micros() as u64)
}

/// What [`FrameLink::present`] did with a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presented {
    /// Copied into `slot` of `generation` as frame `seq`; its `Ready`
    /// follows when the copy has finished.
    Slot {
        generation: u32,
        slot: u32,
        seq: u64,
    },
    /// No host is connected (or its slots are still being made).
    NoHost,
    /// Every slot is held by the host: this frame is not shown there.
    Busy,
}

/// Engine → link thread.
enum Request {
    /// Make generation `generation` of connection `connection` at this size.
    Export {
        connection: u64,
        generation: u32,
        width: u32,
        height: u32,
    },
}

/// Link threads → engine.
enum Event {
    Connected {
        connection: u64,
        writer: Arc<Mutex<UnixStream>>,
    },
    Slots {
        connection: u64,
        generation: u32,
        size: (u32, u32),
        textures: Vec<wgpu::Texture>,
    },
    Msg {
        connection: u64,
        msg: Msg,
    },
    Closed {
        connection: u64,
    },
}

/// One generation of slots on the engine's side.
struct Generation {
    generation: u32,
    size: (u32, u32),
    textures: Vec<wgpu::Texture>,
    free: Vec<u32>,
}

/// The connected host.
struct Host {
    connection: u64,
    writer: Arc<Mutex<UnixStream>>,
    /// The generation drawn into, once its Hello has gone.
    current: Option<Generation>,
    /// The generation asked of the link thread and its size, until its
    /// slots arrive.
    pending: Option<(u32, (u32, u32))>,
    next_generation: u32,
}

/// A renderer's link to a host that draws its frames: see the module.
pub struct FrameLink {
    path: PathBuf,
    requests: Sender<Request>,
    events: Receiver<Event>,
    host: Option<Host>,
    resize: Option<(u32, u32)>,
    seq: u64,
}

impl FrameLink {
    /// Links `renderer` to the host listening on `path`: connects now and
    /// whenever the host comes back, on a thread of its own. Never blocks
    /// and never fails; a host that is not there leaves the renderer
    /// drawing offscreen as before. The renderer should draw in
    /// [`OUTPUT_FORMAT`] (a frame in another format is not shown).
    pub fn connect(path: impl Into<PathBuf>, renderer: &Renderer) -> Self {
        let path = path.into();
        let (requests, requests_rx) = mpsc::channel();
        let (events_tx, events) = mpsc::channel();
        let device = renderer.device().clone();
        let p = path.clone();
        let _ = std::thread::Builder::new()
            .name("og-frames".into())
            .spawn(move || link_thread(p, device, requests_rx, events_tx));
        Self {
            path,
            requests,
            events,
            host: None,
            resize: None,
            seq: 0,
        }
    }

    /// The socket this link connects to.
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Whether a host is connected and has this renderer's slots.
    pub fn connected(&self) -> bool {
        self.host.as_ref().is_some_and(|h| h.current.is_some())
    }

    /// The size the host asked for since the last call, taken once: the
    /// host resizes the renderer to it before its next frame.
    pub fn take_resize(&mut self) -> Option<(u32, u32)> {
        self.drain();
        self.resize.take()
    }

    /// Shows the frame `renderer` drew last offscreen in the host's window:
    /// copied into a free slot on the GPU, announced when the copy has
    /// finished. Call it once per frame of the loop, after
    /// [`Renderer::render`] to [`crate::RenderTarget::Offscreen`].
    pub fn present(&mut self, renderer: &Renderer) -> Presented {
        // Finished copies send their Ready from here (on_submitted_work_done).
        let _ = renderer.device().poll(wgpu::PollType::Poll);
        self.drain();
        let size = renderer.size();
        let Some(host) = self.host.as_mut() else {
            return Presented::NoHost;
        };
        let stale = host.current.as_ref().is_none_or(|g| g.size != size);
        if stale && host.pending.is_none_or(|(_, s)| s != size) {
            // Every Ready of the old generation goes before its Rehello.
            let _ = renderer.device().poll(wgpu::PollType::wait_indefinitely());
            host.current = None;
            let generation = host.next_generation;
            host.next_generation += 1;
            host.pending = Some((generation, size));
            let _ = self.requests.send(Request::Export {
                connection: host.connection,
                generation,
                width: size.0,
                height: size.1,
            });
        }
        let Some(current) = host.current.as_mut().filter(|g| g.size == size) else {
            return Presented::NoHost;
        };
        if renderer.output_format().remove_srgb_suffix() != SLOT_FORMAT {
            return Presented::NoHost;
        }
        let Some(slot) = current.free.pop() else {
            return Presented::Busy;
        };
        let mut encoder =
            renderer
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("shared frame copy"),
                });
        encoder.copy_texture_to_texture(
            renderer.persistent.offscreen.as_image_copy(),
            current.textures[slot as usize].as_image_copy(),
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );
        renderer.queue().submit([encoder.finish()]);
        self.seq += 1;
        let (generation, seq) = (current.generation, self.seq);
        let writer = host.writer.clone();
        let submitted = Instant::now();
        renderer.queue().on_submitted_work_done(move || {
            let msg = Msg::Ready {
                generation,
                slot,
                seq,
                render_ms: submitted.elapsed().as_secs_f32() * 1000.0,
                sent_us: now_us(),
            };
            if let Ok(mut w) = writer.lock() {
                let _ = send(&mut *w, &msg);
            }
        });
        Presented::Slot {
            generation,
            slot,
            seq,
        }
    }

    fn drain(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Connected { connection, writer } => {
                    if let Some(old) = self.host.replace(Host {
                        connection,
                        writer,
                        current: None,
                        pending: None,
                        next_generation: 0,
                    }) {
                        shutdown(&old.writer);
                    }
                }
                Event::Slots {
                    connection,
                    generation,
                    size,
                    textures,
                } => {
                    if let Some(host) = self.host.as_mut().filter(|h| {
                        h.connection == connection
                            && h.pending.is_some_and(|(g, _)| g == generation)
                    }) {
                        host.pending = None;
                        host.current = Some(Generation {
                            generation,
                            size,
                            free: (0..textures.len() as u32).rev().collect(),
                            textures,
                        });
                    }
                }
                Event::Msg { connection, msg } => {
                    let Some(host) = self.host.as_mut().filter(|h| h.connection == connection)
                    else {
                        continue;
                    };
                    match msg {
                        Msg::Release { generation, slot } => {
                            if let Some(g) = host.current.as_mut().filter(|g| {
                                g.generation == generation
                                    && (slot as usize) < g.textures.len()
                                    && !g.free.contains(&slot)
                            }) {
                                g.free.push(slot);
                            }
                        }
                        Msg::Resize { width, height } => {
                            self.resize = Some((
                                width.clamp(MIN_SIDE, MAX_SIZE.0),
                                height.clamp(MIN_SIDE, MAX_SIZE.1),
                            ));
                        }
                        Msg::Ready { .. } | Msg::Rehello => {}
                    }
                }
                Event::Closed { connection } => {
                    if self
                        .host
                        .as_ref()
                        .is_some_and(|h| h.connection == connection)
                    {
                        self.host = None;
                    }
                }
            }
        }
    }
}

impl Drop for FrameLink {
    fn drop(&mut self) {
        if let Some(host) = self.host.take() {
            shutdown(&host.writer);
        }
    }
}

fn shutdown(writer: &Mutex<UnixStream>) {
    if let Ok(w) = writer.lock() {
        let _ = w.shutdown(std::net::Shutdown::Both);
    }
}

/// Connects to `path` (again after each disconnection), reads the host's
/// messages on a second thread and makes the generations the engine asks
/// for. Ends when the [`FrameLink`] is dropped.
fn link_thread(
    path: PathBuf,
    device: wgpu::Device,
    requests: Receiver<Request>,
    events: Sender<Event>,
) {
    let mut connection = 0u64;
    let mut said_failure = false;
    loop {
        let stream = match UnixStream::connect(&path) {
            Ok(s) => s,
            Err(_) => match requests.recv_timeout(RETRY) {
                Err(RecvTimeoutError::Disconnected) => return,
                _ => continue,
            },
        };
        connection += 1;
        let (Ok(reader), Ok(writer)) = (stream.try_clone(), stream.try_clone()) else {
            continue;
        };
        let writer = Arc::new(Mutex::new(writer));
        let open = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let (tx, o) = (events.clone(), open.clone());
        let _ = std::thread::Builder::new()
            .name("og-frames-reader".into())
            .spawn(move || {
                let mut reader = reader;
                while let Ok(msg) = recv(&mut reader) {
                    if tx.send(Event::Msg { connection, msg }).is_err() {
                        break;
                    }
                }
                o.store(false, std::sync::atomic::Ordering::Relaxed);
                let _ = tx.send(Event::Closed { connection });
            });
        if events
            .send(Event::Connected {
                connection,
                writer: writer.clone(),
            })
            .is_err()
        {
            return;
        }
        while open.load(std::sync::atomic::Ordering::Relaxed) {
            let request = match requests.recv_timeout(Duration::from_millis(100)) {
                Ok(r) => r,
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    shutdown(&writer);
                    return;
                }
            };
            let Request::Export {
                connection: c,
                generation,
                width,
                height,
            } = request;
            if c != connection {
                continue;
            }
            match export(&device, &writer, generation, width, height) {
                Ok((adapter, textures)) => {
                    if generation == 0 {
                        eprintln!(
                            "og: frames shared with {} ({SLOTS} slots of {width}x{height} on {adapter})",
                            path.display()
                        );
                    }
                    let _ = events.send(Event::Slots {
                        connection,
                        generation,
                        size: (width, height),
                        textures,
                    });
                }
                Err(e) => {
                    // A device whose memory cannot be shared stays offscreen.
                    if !said_failure {
                        eprintln!(
                            "og: frames not shared with {}: {e}; drawing offscreen only",
                            path.display()
                        );
                        said_failure = true;
                    }
                    shutdown(&writer);
                    break;
                }
            }
        }
        // The host went away (or its slots could not be made): wait for it
        // to listen again.
        let wait = if said_failure { RETRY * 10 } else { RETRY };
        if let Err(RecvTimeoutError::Disconnected) = requests.recv_timeout(wait) {
            return;
        }
    }
}

/// Makes a generation's slots and sends their memory (after a `Rehello`
/// past the first). The adapter's name and the slots.
fn export(
    device: &wgpu::Device,
    writer: &Mutex<UnixStream>,
    generation: u32,
    width: u32,
    height: u32,
) -> Result<(String, Vec<wgpu::Texture>), String> {
    let (hello, textures, handles) = export_slots(device, width, height, SLOTS)?;
    let mut w = writer.lock().map_err(|_| "the socket's writer panicked")?;
    if generation > 0 {
        send(&mut *w, &Msg::Rehello).map_err(|e| e.to_string())?;
    }
    send_hello(&w, &hello, &handles)?;
    Ok((hello.adapter.clone(), textures))
}
