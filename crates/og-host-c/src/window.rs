//! The desktop window a store shell draws in (feature `window`): one winit
//! window per process, presenting the engine's frames on its surface and
//! handing its keys, pointer and wheel to the engine as protocol 5's
//! `InjectedInput`, the events the web shell sends (`og_runtime_probe_input`).
//!
//! The shell owns the loop: `og_c_window_open` once, after the engine
//! exists and before its first tick, then `og_c_window_pump` before every
//! display frame (winit's `pump_app_events`, so the loop is the shell's on
//! macOS, Windows and Linux alike), each handed to the engine as a
//! presenting host's (`og_present_frame`), which keeps the frame cap and
//! answers when the next drawn frame is due: the shell waits until then
//! (winit's `ControlFlow::WaitUntil`). The surface presents with vsync: a
//! drawn frame waits for the display. The window tells the engine its
//! monitor's refresh rate (`og_present_refresh`) and whether it is
//! fullscreen (`og_present_fullscreen`), and goes fullscreen or leaves it
//! when the player's settings ask (`og_c_window_fullscreen`); the player's
//! OS locale is `og_c_locale`'s. The engine's audio plays on the default
//! output device (og_audio::device, the napi host's `device` sink); a
//! machine without one runs silent. Gamepads (gilrs-core) are polled each
//! pump and sent raw and whole when they change, as the web shell's
//! `gamepad` events: their buttons, axes and hat in the driver's order with
//! their SDL GUID, which og-input maps. A pad keeps its slot (0 to 7) while
//! it is connected: another one going does not renumber it.

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
use winit::event::{ElementState, Ime, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::platform::pump_events::{EventLoopExtPumpEvents, PumpStatus};
use winit::window::{CursorIcon, Window, WindowId};

struct Shell {
    events: EventLoop<()>,
    /// The monitor's refresh (mHz) and whether the window was fullscreen,
    /// as the engine was last told.
    refresh: Option<u32>,
    fullscreen: bool,
    app: App,
    /// The default output device playing the engine's audio, if there is one.
    audio: Option<og_audio::device::DeviceSink>,
    /// The gamepads, if the platform's input library opened.
    pads: Option<gilrs_core::Gilrs>,
    /// The pad in each slot (0 to 7).
    slots: [Option<PadSlot>; og_input::MAX_GAMEPADS],
}

/// The order this platform's driver lists a pad's inputs in.
const PAD_LAYOUT: og_input::PadLayout = if cfg!(target_os = "linux") {
    og_input::PadLayout::Linux
} else if cfg!(target_os = "macos") {
    og_input::PadLayout::Mac
} else {
    og_input::PadLayout::XInput
};

/// A connected pad in its slot: its driver's id, its inputs' codes in the
/// driver's order (hat 0 apart from the axes), and its raw state.
struct PadSlot {
    id: usize,
    buttons: Vec<u32>,
    /// Each axis's code and range.
    axes: Vec<(u32, i32, i32)>,
    raw: og_input::RawPad,
    sent: Option<og_input::RawPad>,
}

impl PadSlot {
    fn new(slot: usize, id: usize, pad: &gilrs_core::Gamepad) -> PadSlot {
        use gilrs_core::native_ev_codes::{AXIS_DPADX, AXIS_DPADY};
        let axes = pad
            .axes()
            .iter()
            .filter(|c| **c != AXIS_DPADX && **c != AXIS_DPADY)
            .map(|c| {
                let (min, max) = pad
                    .axis_info(*c)
                    .map_or((-32768, 32767), |i| (i.min, i.max));
                (c.into_u32(), min, max)
            })
            .collect();
        PadSlot {
            id,
            buttons: pad.buttons().iter().map(|c| c.into_u32()).collect(),
            axes,
            raw: og_input::RawPad {
                pad: slot as u32,
                connected: true,
                layout: PAD_LAYOUT,
                guid: pad.uuid(),
                ..og_input::RawPad::default()
            },
            sent: None,
        }
    }

    /// One driver event into the raw state.
    fn take(&mut self, event: gilrs_core::EventType) {
        use gilrs_core::EventType::*;
        use gilrs_core::native_ev_codes::{AXIS_DPADX, AXIS_DPADY};
        match event {
            ButtonPressed(c) | ButtonReleased(c) => {
                let on = matches!(event, ButtonPressed(_));
                if let Some(i) = self.buttons.iter().position(|b| *b == c.into_u32())
                    && let Some(v) = self.raw.buttons.get_mut(i)
                {
                    *v = f32::from(u8::from(on));
                }
            }
            AxisValueChanged(v, c) if c == AXIS_DPADX || c == AXIS_DPADY => {
                let (neg, pos) = if c == AXIS_DPADX { (8, 2) } else { (1, 4) };
                self.raw.hat &= !(neg | pos);
                self.raw.hat |= match v.signum() {
                    -1 => neg,
                    1 => pos,
                    _ => 0,
                };
            }
            AxisValueChanged(v, c) => {
                if let Some(i) = self.axes.iter().position(|a| a.0 == c.into_u32())
                    && let Some(a) = self.raw.axes.get_mut(i)
                {
                    let (_, min, max) = self.axes[i];
                    let span = (max as f32 - min as f32).max(1.0);
                    *a = ((v as f32 - min as f32) / span * 2.0 - 1.0).clamp(-1.0, 1.0);
                }
            }
            _ => {}
        }
    }
}

impl Shell {
    /// The pads' events since the last pump: a pad's raw state when it
    /// changed, and a disconnect for a slot whose pad went.
    fn poll_pads(&mut self) -> Vec<Value> {
        let Some(g) = self.pads.as_mut() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        while let Some(e) = g.next_event() {
            let at = self
                .slots
                .iter()
                .position(|s| s.as_ref().is_some_and(|s| s.id == e.id));
            match (e.event, at) {
                (gilrs_core::EventType::Connected, None) => {
                    let (Some(free), Some(pad)) =
                        (self.slots.iter().position(Option::is_none), g.gamepad(e.id))
                    else {
                        continue;
                    };
                    eprintln!("og-host-c: gamepad {free} connected ({})", pad.name());
                    self.slots[free] = Some(PadSlot::new(free, e.id, pad));
                }
                (gilrs_core::EventType::Disconnected, Some(at)) => {
                    eprintln!("og-host-c: gamepad {at} disconnected");
                    self.slots[at] = None;
                    let gone = og_input::RawPad {
                        pad: at as u32,
                        connected: false,
                        ..og_input::RawPad::default()
                    };
                    out.push(og_input::InjectedInput::Gamepad(gone).to_json());
                }
                (event, Some(at)) => {
                    if let Some(s) = self.slots[at].as_mut() {
                        s.take(event);
                    }
                }
                _ => {}
            }
        }
        for s in self.slots.iter_mut().flatten() {
            if s.sent != Some(s.raw) {
                s.sent = Some(s.raw);
                out.push(og_input::InjectedInput::Gamepad(s.raw).to_json());
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
    /// An input method shows a preedit: its keys are its own.
    composing: bool,
}

thread_local! {
    static SHELL: RefCell<Option<Shell>> = const { RefCell::new(None) };
    /// The engine the window draws.
    static ENGINE: Cell<u32> = const { Cell::new(0) };
    /// The size the window asked for since the last frame, taken by it.
    static RESIZE: Cell<Option<(u32, u32)>> = const { Cell::new(None) };
    /// The last answer of `og_c_window_open`.
    static ANSWER: RefCell<CString> = RefCell::new(CString::default());
    /// The UI has a text field focused (the frame answer's `text`): the
    /// paste shortcut reads the clipboard into it.
    static TEXT_FOCUS: Cell<bool> = const { Cell::new(false) };
    /// The OS clipboard, opened on its first use.
    static CLIPBOARD: RefCell<Option<arboard::Clipboard>> = const { RefCell::new(None) };
}

/// `f` on the OS clipboard, if it opens.
fn clipboard<R>(f: impl FnOnce(&mut arboard::Clipboard) -> R) -> Option<R> {
    CLIPBOARD.with_borrow_mut(|c| {
        if c.is_none() {
            *c = arboard::Clipboard::new()
                .map_err(|e| eprintln!("og-host-c: no clipboard: {e}"))
                .ok();
        }
        c.as_mut().map(f)
    })
}

/// The paste shortcut's modifier is held: Command on macOS, else Control.
fn paste_held(held: &BTreeSet<String>) -> bool {
    let [l, r] = if cfg!(target_os = "macos") {
        ["MetaLeft", "MetaRight"]
    } else {
        ["ControlLeft", "ControlRight"]
    };
    held.contains(l) || held.contains(r)
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
        // The paste shortcut into a focused text field: the clipboard's
        // text, as the page's paste event gives it.
        if down
            && key == "KeyV"
            && TEXT_FOCUS.get()
            && paste_held(&self.held)
            && let Some(text) = clipboard(|c| c.get_text().ok()).flatten()
            && !text.is_empty()
        {
            self.input.push(json!({ "kind": "text", "text": text }));
        }
        // Typed text for the UI's text fields; control characters are keys,
        // and an input method's keys are its composition's.
        if down
            && !self.composing
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
            // An input method's composition and commit, as the page's
            // compositionupdate and compositionend; one switched off
            // mid-word ends its composition uncommitted.
            WindowEvent::Ime(ime) => match ime {
                Ime::Preedit(text, _) => {
                    if self.composing || !text.is_empty() {
                        self.composing = !text.is_empty();
                        self.input
                            .push(json!({ "kind": "text", "text": text, "composing": true }));
                    }
                }
                Ime::Commit(text) => {
                    self.composing = false;
                    self.input.push(json!({ "kind": "text", "text": text }));
                }
                Ime::Disabled if self.composing => {
                    self.composing = false;
                    self.input
                        .push(json!({ "kind": "text", "text": "", "composing": true }));
                }
                Ime::Enabled | Ime::Disabled => {}
            },
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
    /// The input method is allowed (a text field is focused), and the
    /// caret it was last told, in CSS px.
    ime: bool,
    ime_at: [f64; 4],
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
    /// A focused text field allows the platform's input method (and the
    /// soft keyboard where there is one) with its candidates at the caret;
    /// a copy goes on the OS clipboard.
    fn text_entry(&mut self, target: &Value, copy: Option<&str>) {
        let caret = target["caret"].as_array().map(|c| {
            let v = |i: usize| c.get(i).and_then(Value::as_f64).unwrap_or(0.0);
            [v(0), v(1), v(2), v(3)]
        });
        TEXT_FOCUS.set(caret.is_some());
        if self.ime != caret.is_some() {
            self.ime = caret.is_some();
            self.window.set_ime_allowed(self.ime);
            self.ime_at = [f64::NAN; 4];
        }
        if let Some(at) = caret
            && at != self.ime_at
        {
            self.ime_at = at;
            self.window.set_ime_cursor_area(
                winit::dpi::LogicalPosition::new(at[0], at[1]),
                LogicalSize::new(at[2].max(1.0), at[3].max(1.0)),
            );
        }
        if let Some(text) = copy
            && let Some(Err(e)) = clipboard(|c| c.set_text(text))
        {
            eprintln!("og-host-c: the copy did not reach the clipboard: {e}");
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
    fn cull_record(&mut self) -> Option<og_render::CullRecord> {
        self.renderer.cull_record().cloned()
    }
    fn cull_asked(&self) -> Option<u64> {
        self.renderer.cull_asked()
    }
    fn read_counts(&mut self) {
        self.renderer.finish_stats(&mut self.stats);
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
        // og-render's `capture_frame`, waited for.
        self.renderer
            .capture_frame(scene, width, height)
            .and_then(|(_, req)| req.wait(&self.renderer, og_render::capture_timeouts().0))
            .ok()
    }
    /// The last presented frame, kept offscreen once the shell asked
    /// (`OG_SHELL_CAPTURE`).
    fn read_frame(&mut self) -> Option<og_render::RgbaImage> {
        self.renderer
            .keeps_frame()
            .then(|| self.renderer.read_offscreen().ok())
            .flatten()
    }
    fn warm(&mut self) -> bool {
        self.renderer.warm()
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
        composing: false,
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
            ime: false,
            ime_at: [f64::NAN; 4],
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
    let pads = match gilrs_core::Gilrs::new() {
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
            refresh: None,
            fullscreen: false,
            app,
            audio,
            pads,
            slots: Default::default(),
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
        let engine = ENGINE.get();
        if let Some(w) = shell.app.window.as_ref() {
            // The monitor the window is on now: its refresh is the display's interval.
            let refresh = w
                .current_monitor()
                .and_then(|m| m.refresh_rate_millihertz());
            if refresh != shell.refresh {
                shell.refresh = refresh;
                og_runtime::og_present_refresh(
                    engine,
                    refresh.map_or(0.0, |mhz| f64::from(mhz) / 1000.0),
                );
            }
            let fullscreen = w.fullscreen().is_some();
            if fullscreen != shell.fullscreen {
                shell.fullscreen = fullscreen;
                og_runtime::og_present_fullscreen(engine, fullscreen);
            }
        }
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

/// Puts the window fullscreen (borderless, on its monitor) or takes it out,
/// as the frame answer's `fullscreen` asks; the next pump tells the engine
/// what the window did.
#[unsafe(no_mangle)]
pub extern "C" fn og_c_window_fullscreen(on: bool) {
    SHELL.with(|s| {
        if let Some(w) = s.borrow().as_ref().and_then(|s| s.app.window.as_ref()) {
            w.set_fullscreen(on.then_some(winit::window::Fullscreen::Borderless(None)));
        }
    });
}

/// The player's locale as the OS names it (a BCP 47 tag, `en-US`), or
/// null where it names none; valid until the next call. The shell gives
/// it to the engine's creation (`locale`).
#[unsafe(no_mangle)]
pub extern "C" fn og_c_locale() -> *const c_char {
    let Some(tag) = sys_locale::get_locale() else {
        return std::ptr::null();
    };
    ANSWER.with(|a| {
        *a.borrow_mut() = CString::new(tag).unwrap_or_default();
        a.borrow().as_ptr()
    })
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
            composing: false,
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
}
