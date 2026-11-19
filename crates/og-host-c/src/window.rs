//! The desktop window a store shell draws in (feature `window`): one winit
//! window per process, presenting the engine's frames on its surface and
//! handing its keys, pointer and wheel to the engine as protocol 5's
//! `InjectedInput`, the events the web shell sends (`og_runtime_probe_input`).
//!
//! The shell owns the loop: `og_c_window_open` once, after the engine
//! exists and before its first tick, then `og_c_window_pump` before every
//! frame (winit's `pump_app_events`, so the loop is the shell's on macOS,
//! Windows and Linux alike). The surface presents with vsync: the frame
//! call waits for the display. The engine's audio plays on the default
//! output device (og_audio::device, the napi host's `device` sink); a
//! machine without one runs silent. Gamepads (gilrs) are polled each pump
//! and sent whole when they change, as the web shell's `gamepad` events
//! (pads 0 to 7, the W3C standard layout, the family from the vendor).

use og_render::{RenderDesc, RenderError, RenderScene, RenderTarget, Renderer};
use og_runtime::RenderBackend;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::ffi::{CStr, CString, c_char};
use std::sync::Arc;
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{CursorIcon, Window, WindowId};

struct Shell {
    events: EventLoop<()>,
    app: App,
    /// The default output device playing the engine's audio, if there is one.
    audio: Option<og_audio::device::DeviceSink>,
    /// The gamepads, if the platform's input library opened.
    pads: Option<gilrs::Gilrs>,
    /// Each pad's last event sent, by its slot (0 to 7).
    sent: [Option<Value>; og_input::MAX_GAMEPADS],
}

/// A pad's family from its USB vendor and name, as the web shell reads it
/// from the browser's id (`padDevice` in packages/engine/src/web/gamepad.ts).
fn pad_device(vendor: Option<u16>, name: &str) -> &'static str {
    match vendor {
        Some(0x045e) => return "xbox",
        Some(0x054c) => return "playstation",
        Some(0x057e) => return "switch",
        _ => {}
    }
    let name = name.to_lowercase();
    if ["xbox", "xinput", "x-box"].iter().any(|n| name.contains(n)) {
        "xbox"
    } else if [
        "playstation",
        "dualshock",
        "dualsense",
        "wireless controller",
    ]
    .iter()
    .any(|n| name.contains(n))
    {
        "playstation"
    } else if ["switch", "pro controller", "joy-con", "nintendo"]
        .iter()
        .any(|n| name.contains(n))
    {
        "switch"
    } else {
        "gamepad"
    }
}

/// gilrs's buttons in the W3C standard layout's order (og-input's GAMEPAD_BUTTONS).
const PAD_BUTTONS: [gilrs::Button; 17] = {
    use gilrs::Button::*;
    [
        South,
        East,
        West,
        North,
        LeftTrigger,
        RightTrigger,
        LeftTrigger2,
        RightTrigger2,
        Select,
        Start,
        LeftThumb,
        RightThumb,
        DPadUp,
        DPadDown,
        DPadLeft,
        DPadRight,
        Mode,
    ]
};

/// One pad's state as the web shell's `gamepad` event: held buttons by
/// name, sticks with y down, analog triggers 0 to 1.
fn pad_event(
    slot: usize,
    connected: bool,
    device: &str,
    held: &[bool; 17],
    sticks: [f32; 4],
    triggers: [f32; 2],
) -> Value {
    let buttons: Vec<&str> = og_input::GAMEPAD_BUTTONS
        .iter()
        .zip(held)
        .filter(|(_, h)| **h)
        .map(|(n, _)| *n)
        .collect();
    json!({
        "kind": "gamepad",
        "pad": slot,
        "connected": connected,
        "buttons": buttons,
        "axes": [sticks[0], -sticks[1], sticks[2], -sticks[3]],
        "triggers": triggers,
        "device": device,
    })
}

impl Shell {
    /// The pads' events since the last pump: each connected pad's state when
    /// it changed, and a disconnect for a slot whose pad went.
    fn poll_pads(&mut self) -> Vec<Value> {
        let Some(g) = self.pads.as_mut() else {
            return Vec::new();
        };
        while g.next_event().is_some() {}
        let mut now: [Option<Value>; og_input::MAX_GAMEPADS] = Default::default();
        for (slot, (_, pad)) in g
            .gamepads()
            .filter(|(_, p)| p.is_connected())
            .take(og_input::MAX_GAMEPADS)
            .enumerate()
        {
            let held = PAD_BUTTONS.map(|b| pad.is_pressed(b));
            let value = |a| pad.value(a);
            let trigger = |b| pad.button_data(b).map_or(0.0, |d| d.value());
            use gilrs::Axis::*;
            now[slot] = Some(pad_event(
                slot,
                true,
                pad_device(pad.vendor_id(), pad.name()),
                &held,
                [
                    value(LeftStickX),
                    value(LeftStickY),
                    value(RightStickX),
                    value(RightStickY),
                ],
                [
                    trigger(gilrs::Button::LeftTrigger2),
                    trigger(gilrs::Button::RightTrigger2),
                ],
            ));
        }
        let mut out = Vec::new();
        for (slot, (was, is)) in self.sent.iter_mut().zip(now).enumerate() {
            match (&*was, is) {
                (_, Some(v)) if was.as_ref() != Some(&v) => {
                    if was.is_none() {
                        eprintln!("og-host-c: gamepad {slot} connected ({})", v["device"]);
                    }
                    out.push(v.clone());
                    *was = Some(v);
                }
                (Some(_), None) => {
                    eprintln!("og-host-c: gamepad {slot} disconnected");
                    out.push(json!({ "kind": "gamepad", "pad": slot, "connected": false, "buttons": [], "axes": [0, 0, 0, 0], "triggers": [0, 0] }));
                    *was = None;
                }
                _ => {}
            }
        }
        out
    }
}

/// The window's side of winit: what happened since the last pump.
struct App {
    title: String,
    size: LogicalSize<f64>,
    window: Option<Arc<Window>>,
    error: Option<String>,
    closed: bool,
    /// `InjectedInput`s, oldest first, handed to the engine after the pump.
    input: Vec<Value>,
    /// The surface's new size in device pixels and its scale.
    resized: Option<(PhysicalSize<u32>, f64)>,
    scale: f64,
    /// The pointer in CSS pixels, for buttons and the wheel.
    pointer: (f64, f64),
    /// Key codes sent down and not yet up, released when focus goes.
    held: BTreeSet<String>,
}

thread_local! {
    static SHELL: RefCell<Option<Shell>> = const { RefCell::new(None) };
    /// The engine the window draws.
    static ENGINE: Cell<u32> = const { Cell::new(0) };
    /// The size the window asked for since the last frame, taken by it.
    static RESIZE: Cell<Option<(u32, u32)>> = const { Cell::new(None) };
    /// The last answer of `og_c_window_open`.
    static ANSWER: RefCell<CString> = RefCell::new(CString::default());
}

/// A W3C `KeyboardEvent.code` from winit's: the same names but for the
/// platform keys (`SuperLeft` is `MetaLeft`).
fn code(key: PhysicalKey) -> Option<String> {
    let PhysicalKey::Code(c) = key else {
        return None;
    };
    let name = format!("{c:?}");
    Some(match name.as_str() {
        "SuperLeft" => "MetaLeft".into(),
        "SuperRight" => "MetaRight".into(),
        _ => name,
    })
}

fn button(b: MouseButton) -> &'static str {
    match b {
        MouseButton::Left => "left",
        MouseButton::Middle => "middle",
        MouseButton::Right => "right",
        _ => "none",
    }
}

impl App {
    fn key(&mut self, event: &KeyEvent) {
        let Some(key) = code(event.physical_key) else {
            return;
        };
        let down = event.state == ElementState::Pressed;
        if down && event.repeat {
            // Held, as the page's repeats are to the engine.
        } else {
            if down {
                self.held.insert(key.clone());
            } else {
                self.held.remove(&key);
            }
            self.input.push(
                json!({ "kind": "key", "key": key, "action": if down { "down" } else { "up" } }),
            );
        }
        // Typed text for the UI's text fields; control characters are keys.
        if down
            && let Some(text) = event
                .text
                .as_ref()
                .filter(|t| !t.chars().any(char::is_control))
        {
            self.input
                .push(json!({ "kind": "text", "text": text.as_str() }));
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        match event_loop.create_window(
            Window::default_attributes()
                .with_title(self.title.clone())
                .with_inner_size(self.size),
        ) {
            Ok(w) => {
                self.scale = w.scale_factor();
                self.window = Some(Arc::new(w));
            }
            Err(e) => self.error = Some(format!("window: {e}")),
        }
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        self.translate(event);
    }
}

impl App {
    /// One window event as the engine hears it: its input, a resize, a close.
    fn translate(&mut self, event: WindowEvent) {
        let s = self.scale.max(f64::MIN_POSITIVE);
        match event {
            WindowEvent::CloseRequested | WindowEvent::Destroyed => self.closed = true,
            WindowEvent::Resized(size) => self.resized = Some((size, self.scale)),
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale = scale_factor;
                if let Some(w) = &self.window {
                    self.resized = Some((w.inner_size(), scale_factor));
                }
            }
            WindowEvent::Focused(false) => {
                for key in std::mem::take(&mut self.held) {
                    self.input
                        .push(json!({ "kind": "key", "key": key, "action": "up" }));
                }
            }
            WindowEvent::KeyboardInput { event, .. } => self.key(&event),
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x / s, position.y / s);
                let (dx, dy) = (x - self.pointer.0, y - self.pointer.1);
                self.pointer = (x, y);
                self.input.push(json!({
                    "kind": "pointer", "action": "move", "x": x, "y": y, "button": "none",
                    "movementX": dx, "movementY": dy,
                }));
            }
            WindowEvent::MouseInput {
                state, button: b, ..
            } => {
                let (x, y) = self.pointer;
                let action = if state == ElementState::Pressed {
                    "down"
                } else {
                    "up"
                };
                self.input.push(json!({
                    "kind": "pointer", "action": action, "x": x, "y": y, "button": button(b),
                }));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                // The page's wheel: CSS pixels, positive down and right.
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        (-f64::from(x) * 40.0, -f64::from(y) * 40.0)
                    }
                    MouseScrollDelta::PixelDelta(p) => (-p.x / s, -p.y / s),
                };
                let (x, y) = self.pointer;
                self.input
                    .push(json!({ "kind": "wheel", "x": x, "y": y, "deltaX": dx, "deltaY": dy }));
            }
            _ => {}
        }
    }
}

/// The play surface in CSS pixels for a window of `size` device pixels.
fn viewport(size: PhysicalSize<u32>, scale: f64) -> Value {
    json!({
        "width": f64::from(size.width.max(1)) / scale,
        "height": f64::from(size.height.max(1)) / scale,
        "dpr": scale,
        "macKeys": cfg!(target_os = "macos"),
    })
}

/// The engine's renderer on the window's surface.
struct WindowBackend {
    renderer: Renderer,
    /// What the surface was made with, for a new one after a loss.
    instance: og_render::wgpu::Instance,
    tier: String,
    window: Arc<Window>,
    stats: og_render::FrameStats,
    losses: u32,
    cursor: Option<(&'static str, u64)>,
}

impl WindowBackend {
    /// A lost device is replaced by a renderer on a new device, presenting
    /// to a new surface of the same window.
    fn recreate(&mut self) -> Result<(), String> {
        let Some(why) = self.renderer.lost() else {
            return Ok(());
        };
        eprintln!("og-host-c: the GPU device was {why}; making the renderer again");
        let successor = self.renderer.successor();
        let surface = self
            .instance
            .create_surface(self.window.clone())
            .map_err(|e| format!("surface: {e}"))?;
        self.renderer = pollster::block_on(successor.make(Some((&self.instance, surface))))
            .map_err(|e| format!("the GPU device was {why}, and a new one failed: {e}"))?;
        self.losses += 1;
        Ok(())
    }
}

impl RenderBackend for WindowBackend {
    fn tier(&self) -> String {
        self.tier.clone()
    }
    fn size(&self) -> (u32, u32) {
        self.renderer.size()
    }
    fn antialias(&self) -> String {
        og_runtime::antialias_name(self.renderer.info().antialias).into()
    }
    fn history(&self) -> Option<(u64, u64)> {
        Some(self.renderer.history())
    }
    fn info(&self) -> Option<og_render::RenderInfo> {
        Some(self.renderer.info().clone())
    }
    fn texture_formats(&self) -> Option<og_render::TextureFormats> {
        Some(self.renderer.info().texture_formats.clone())
    }
    fn compute(&self) -> Option<(og_render::wgpu::Device, og_render::wgpu::Queue)> {
        Some((
            self.renderer.device().clone(),
            self.renderer.queue().clone(),
        ))
    }
    fn texture_uploaded(&self, scene: &RenderScene, texture: og_render::TextureHandle) -> bool {
        self.renderer.texture_uploaded(scene, texture)
    }
    /// The frame at the size the window asked for since the last one,
    /// presented; an occluded window draws nothing while the game goes on.
    fn render(&mut self, scene: &RenderScene) -> Result<(), String> {
        self.recreate()?;
        if let Some((w, h)) = RESIZE.take() {
            self.renderer.resize(w, h);
        }
        self.renderer.request_culling_counts();
        self.window.pre_present_notify();
        match self.renderer.render(scene, &RenderTarget::Surface) {
            Ok(s) => self.stats = s,
            Err(RenderError::SurfaceOccluded) | Err(RenderError::DeviceLost(_)) => {}
            Err(e) => return Err(e.to_string()),
        }
        Ok(())
    }
    fn cursor(&mut self, shape: &og_runtime::ui_cursor::CursorShape) {
        if self.cursor != Some(shape.id()) {
            self.cursor = Some(shape.id());
            self.window
                .set_cursor(shape.keyword.parse::<CursorIcon>().unwrap_or_default());
        }
    }
    fn set_clock(&mut self, clock: Option<fn() -> f64>) {
        self.renderer.set_clock(clock);
    }
    fn pass_timings(&self) -> Vec<og_render::PassTiming> {
        self.renderer.pass_timings().to_vec()
    }
    fn gpu_timings(&mut self, wait: bool) -> Vec<og_render::GpuFrame> {
        self.renderer.gpu_timings(wait)
    }
    fn gpu_timing_absent(&self) -> Option<&'static str> {
        self.renderer.gpu_timing_absent()
    }
    fn frame_stats(&self) -> og_render::FrameStats {
        self.stats
    }
    fn capture(
        &mut self,
        scene: &RenderScene,
        width: u32,
        height: u32,
    ) -> Option<og_render::RgbaImage> {
        let target = self.renderer.size();
        let size = if width == 0 || height == 0 {
            target
        } else {
            (width, height)
        };
        if size != target {
            self.renderer.resize(size.0, size.1);
        }
        let image = self
            .renderer
            .render_capture(scene, &RenderTarget::Offscreen)
            .ok()
            .and_then(|_| self.renderer.read_offscreen().ok());
        if size != target {
            self.renderer.resize(target.0, target.1);
        }
        image
    }
    /// The last presented frame, kept offscreen once the shell asked
    /// (`OG_SHELL_CAPTURE`).
    fn read_frame(&mut self) -> Option<og_render::RgbaImage> {
        self.renderer
            .keeps_frame()
            .then(|| self.renderer.read_offscreen().ok())
            .flatten()
    }
    fn presents(&self) -> bool {
        true
    }
    fn set_render_scale(&mut self, scale: f32) {
        self.renderer.set_render_scale(scale);
    }
    fn scene_size(&self) -> (u32, u32) {
        self.renderer.scene_size_drawn()
    }
    fn gpu_memory(&self) -> Option<og_render::GpuMemory> {
        Some(self.renderer.gpu_memory())
    }
    fn recover(&mut self) -> Result<bool, String> {
        self.recreate().map(|()| false)
    }
    fn device_losses(&self) -> u32 {
        self.losses
    }
    fn lose_device(&mut self) -> bool {
        self.renderer.lose_device();
        true
    }
}

fn open(engine: u32, width: u32, height: u32, title: String, keep: bool) -> Result<String, String> {
    if SHELL.with(|s| s.borrow().is_some()) {
        return Err("the window is already open: a shell has one".into());
    }
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder
            .with_activation_policy(ActivationPolicy::Regular)
            .with_activate_ignoring_other_apps(true);
    }
    let mut events = builder.build().map_err(|e| e.to_string())?;
    let mut app = App {
        title,
        size: LogicalSize::new(f64::from(width.max(1)), f64::from(height.max(1))),
        window: None,
        error: None,
        closed: false,
        input: Vec::new(),
        resized: None,
        scale: 1.0,
        pointer: (0.0, 0.0),
        held: BTreeSet::new(),
    };
    // The window is made when the loop first resumes.
    for _ in 0..100 {
        if app.window.is_some() || app.error.is_some() {
            break;
        }
        if let PumpStatus::Exit(code) =
            events.pump_app_events(Some(Duration::from_millis(10)), &mut app)
        {
            return Err(format!("the window's event loop exited ({code})"));
        }
    }
    if let Some(e) = app.error.take() {
        return Err(e);
    }
    let window = app.window.clone().ok_or("no window was made")?;
    let size = window.inner_size();
    let desc = og_runtime::render_desc(
        engine,
        RenderDesc {
            width: size.width.max(1),
            height: size.height.max(1),
            ..RenderDesc::default()
        },
    );
    // Compiled pipelines kept across runs where the backend can (Vulkan),
    // as the napi host keeps them: OG_PIPELINE_CACHE names the directory,
    // `off` keeps none.
    og_render::set_pipeline_cache_dir(match std::env::var("OG_PIPELINE_CACHE") {
        Ok(v) if v == "off" => None,
        Ok(v) if !v.is_empty() => Some(v.into()),
        _ => std::env::var_os(if cfg!(windows) {
            "LOCALAPPDATA"
        } else {
            "HOME"
        })
        .map(|h| std::path::PathBuf::from(h).join(".cache/origin-engine/pipelines")),
    });
    let instance = desc.backend.instance();
    let surface = instance
        .create_surface(window.clone())
        .map_err(|e| format!("surface: {e}"))?;
    let mut renderer = pollster::block_on(Renderer::with_surface(&desc, &instance, surface))
        .map_err(|e| e.to_string())?;
    if keep {
        renderer.set_keep_frame(true).map_err(|e| e.to_string())?;
    }
    let tier = renderer.info().backend_tier();
    og_runtime::set_render_backend(
        engine,
        Some(Box::new(WindowBackend {
            renderer,
            instance,
            tier: tier.clone(),
            window: window.clone(),
            stats: Default::default(),
            losses: 0,
            cursor: None,
        })),
    )?;
    og_runtime::set_viewport(engine, viewport(size, app.scale))?;
    app.resized = None;
    let audio = match og_audio::link(engine).map(|l| og_audio::device::DeviceSink::open(&l)) {
        Some(Ok(sink)) => Some(sink),
        Some(Err(e)) => {
            eprintln!("og-host-c: no audio: {e}");
            None
        }
        None => None,
    };
    let pads = match gilrs::Gilrs::new() {
        Ok(g) => Some(g),
        Err(e) => {
            eprintln!("og-host-c: no gamepads: {e}");
            None
        }
    };
    ENGINE.set(engine);
    SHELL.with(|s| {
        *s.borrow_mut() = Some(Shell {
            events,
            app,
            audio,
            pads,
            sent: Default::default(),
        })
    });
    Ok(tier)
}

/// Opens the window for `engine` at `width`×`height` CSS pixels, its
/// renderer on the window's surface. Answers wire JSON (`{ok, value:
/// tier}` or `{ok: false, reason}`), valid until the next call; with
/// `keep` the presented frames are also kept offscreen for
/// `og_c_window_capture`.
///
/// # Safety
/// `title` is a NUL-terminated UTF-8 string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_c_window_open(
    engine: u32,
    width: u32,
    height: u32,
    title: *const c_char,
    keep: bool,
) -> *const c_char {
    let title = if title.is_null() {
        "Origin".to_string()
    } else {
        // SAFETY: the caller's string.
        unsafe { CStr::from_ptr(title) }
            .to_string_lossy()
            .into_owned()
    };
    let answer = match open(engine, width, height, title, keep) {
        Ok(tier) => json!({ "ok": true, "value": tier }),
        Err(reason) => json!({ "ok": false, "reason": reason }),
    };
    ANSWER.with(|a| {
        *a.borrow_mut() = CString::new(answer.to_string()).unwrap_or_default();
        a.borrow().as_ptr()
    })
}

/// Handles what happened to the window since the last call: input goes to
/// the engine, a resize to its surface and viewport. False once the
/// window was closed (or none is open).
#[unsafe(no_mangle)]
pub extern "C" fn og_c_window_pump() -> bool {
    let pumped = SHELL.with(|s| {
        let mut s = s.borrow_mut();
        let shell = s.as_mut()?;
        let status = shell
            .events
            .pump_app_events(Some(Duration::ZERO), &mut shell.app);
        let latency = shell.audio.as_ref().map(|a| a.latency());
        let pads = shell.poll_pads();
        let app = &mut shell.app;
        let closed = app.closed || matches!(status, PumpStatus::Exit(_));
        let mut input = std::mem::take(&mut app.input);
        input.extend(pads);
        Some((input, app.resized.take(), closed, latency))
    });
    let Some((input, resized, closed, latency)) = pumped else {
        return false;
    };
    let engine = ENGINE.get();
    // The device's latency reaches og.Audio.outputLatency between ticks, as host input.
    if let Some(seconds) = latency {
        let _ = og_runtime::set_audio_latency(engine, seconds);
    }
    if let Some((size, scale)) = resized
        && size.width > 0
        && size.height > 0
    {
        RESIZE.set(Some((size.width, size.height)));
        if let Err(e) = og_runtime::set_viewport(engine, viewport(size, scale)) {
            eprintln!("og-host-c: the window's viewport: {e}");
        }
    }
    for event in input {
        og_runtime::og_runtime_probe_input(engine, event.to_string());
    }
    !closed
}

/// What the audio device played, as JSON (`{device, sampleRate, frames,
/// voicesPeak, late, dropped}`), or `null` without one; valid until the
/// next call.
#[unsafe(no_mangle)]
pub extern "C" fn og_c_window_audio() -> *const c_char {
    let answer = SHELL.with(|s| {
        s.borrow().as_ref().and_then(|s| s.audio.as_ref()).map(|a| {
            let m = a.stats();
            json!({
                "device": a.name(),
                "sampleRate": a.sample_rate(),
                "frames": m.frames,
                "voicesPeak": m.voices_peak,
                "late": m.late,
                "dropped": a.link().dropped(),
            })
        })
    });
    ANSWER.with(|a| {
        *a.borrow_mut() =
            CString::new(answer.unwrap_or(Value::Null).to_string()).unwrap_or_default();
        a.borrow().as_ptr()
    })
}

/// Writes the last presented frame to `path` as a PNG (the window was
/// opened with `keep`); false when there is none.
///
/// # Safety
/// `path` is a NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_c_window_capture(path: *const c_char) -> bool {
    // SAFETY: the caller's string.
    let path = unsafe { CStr::from_ptr(path) }
        .to_string_lossy()
        .into_owned();
    let Ok(frame) = og_runtime::render_frame(ENGINE.get()) else {
        return false;
    };
    let image = og_render::RgbaImage {
        width: frame.width,
        height: frame.height,
        pixels: frame.rgba,
    };
    std::fs::write(path, og_render::png(&image)).is_ok()
}

/// Closes the window: the engine's renderer goes first (its surface is the
/// window's), then the window and its event loop. The shell calls it after
/// the game's engine was destroyed, before the process ends.
#[unsafe(no_mangle)]
pub extern "C" fn og_c_window_close() {
    let _ = og_runtime::set_render_backend(ENGINE.get(), None);
    SHELL.with(|s| s.borrow_mut().take());
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::dpi::PhysicalPosition;
    use winit::event::{DeviceId, TouchPhase};
    use winit::keyboard::KeyCode;

    fn app(scale: f64) -> App {
        App {
            title: String::new(),
            size: LogicalSize::new(1.0, 1.0),
            window: None,
            error: None,
            closed: false,
            input: Vec::new(),
            resized: None,
            scale,
            pointer: (0.0, 0.0),
            held: BTreeSet::new(),
        }
    }

    /// The window's input reaches the engine as the web shell's does: W3C
    /// key codes, CSS pixels, the wheel positive down, buttons at the pointer.
    #[test]
    fn window_input_is_the_web_shells() {
        assert_eq!(
            code(PhysicalKey::Code(KeyCode::KeyW)).as_deref(),
            Some("KeyW")
        );
        assert_eq!(
            code(PhysicalKey::Code(KeyCode::SuperLeft)).as_deref(),
            Some("MetaLeft")
        );
        assert_eq!(
            code(PhysicalKey::Code(KeyCode::ArrowUp)).as_deref(),
            Some("ArrowUp")
        );

        let device_id = DeviceId::dummy();
        let mut a = app(2.0);
        a.translate(WindowEvent::CursorMoved {
            device_id,
            position: PhysicalPosition::new(300.0, 100.0),
        });
        a.translate(WindowEvent::MouseInput {
            device_id,
            state: ElementState::Pressed,
            button: MouseButton::Right,
        });
        // One line up on the wheel scrolls the page up: deltaY negative.
        a.translate(WindowEvent::MouseWheel {
            device_id,
            delta: MouseScrollDelta::LineDelta(0.0, 1.0),
            phase: TouchPhase::Moved,
        });
        a.translate(WindowEvent::MouseWheel {
            device_id,
            delta: MouseScrollDelta::PixelDelta(PhysicalPosition::new(0.0, -30.0)),
            phase: TouchPhase::Moved,
        });
        assert_eq!(
            a.input,
            vec![
                json!({ "kind": "pointer", "action": "move", "x": 150.0, "y": 50.0, "button": "none", "movementX": 150.0, "movementY": 50.0 }),
                json!({ "kind": "pointer", "action": "down", "x": 150.0, "y": 50.0, "button": "right" }),
                json!({ "kind": "wheel", "x": 150.0, "y": 50.0, "deltaX": -0.0, "deltaY": -40.0 }),
                json!({ "kind": "wheel", "x": 150.0, "y": 50.0, "deltaX": -0.0, "deltaY": 15.0 }),
            ]
        );

        // Losing focus releases what is held; a close is reported.
        a.input.clear();
        a.held.insert("KeyW".into());
        a.translate(WindowEvent::Focused(false));
        a.translate(WindowEvent::CloseRequested);
        assert_eq!(
            a.input,
            vec![json!({ "kind": "key", "key": "KeyW", "action": "up" })]
        );
        assert!(a.held.is_empty() && a.closed);
    }

    /// A pad reaches the engine as the web shell's `gamepad` event: the
    /// standard layout's buttons, sticks with y down, the family from the
    /// vendor, read back by og-input as the state it was.
    #[test]
    fn pads_are_the_web_shells() {
        assert_eq!(
            pad_device(Some(0x054c), "Wireless Controller"),
            "playstation"
        );
        assert_eq!(pad_device(None, "Xbox Wireless Controller"), "xbox");
        assert_eq!(pad_device(Some(0x1234), "8BitDo SN30"), "gamepad");

        let mut held = [false; 17];
        held[1] = true; // East: `b`
        held[12] = true; // DPadUp: `up`
        // gilrs's sticks are y up: pushed up and to the right.
        let event = pad_event(
            3,
            true,
            "playstation",
            &held,
            [0.5, 0.75, 0.0, -1.0],
            [0.25, 0.0],
        );
        let Some(og_input::InjectedInput::Gamepad(state)) =
            og_input::InjectedInput::from_json(&event)
        else {
            panic!("og-input refused {event}");
        };
        assert_eq!(state.pad, 3);
        assert!(state.connected);
        assert_eq!(state.buttons, (1 << 1) | (1 << 12));
        assert_eq!(state.axes, [0.5, -0.75, 0.0, 1.0]);
        assert_eq!(state.triggers, [0.25, 0.0]);
        assert_eq!(state.device, Some(og_input::Device::PlayStation));
    }
}
