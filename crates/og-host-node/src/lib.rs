//! Origin Engine native host (`docs/design.md` §2.1, §5): the binding
//! symbols of `crates/og-api/api.ogt` as a napi addon (`origin-engine.node`),
//! headless, for `og run` and the executors. `packages/engine/src/node/`
//! loads it under bun or node.
//!
//! - `src/generated/glue.rs` (`og api gen`) exports every symbol under its
//!   snake_case name and calls the `og_api::host` trait method of the same
//!   name; `src/host.rs` implements the traits, so a member missing here is a
//!   build error.
//! - Column views alias engine memory: each field is an external
//!   ArrayBuffer over the column. The buffer is kept while the column stays
//!   where it is, and detached before anything can move it (a structural
//!   change, a restore), so a kept typed array reads as empty and never
//!   reaches moved memory. Where the runtime refuses external buffers (a V8
//!   sandbox), or with `OG_NODE_VIEWS=copy`, each field is copied into an
//!   ArrayBuffer (kept the same way) as the view is made and writable ones
//!   are copied back when the system call ends.
//! - `og_node_attach_offscreen` gives an engine an offscreen renderer
//!   (`GameDesc.render` picks the antialiasing, `OG_GFX_CULLING=cpu` forces
//!   the CPU cull); the probe's `capture` and
//!   the `screenshot` command render offscreen at the asked size and read it
//!   back blocking (`og_render::Renderer::read_offscreen`);
//!   `og_node_wait_gpu` waits for its GPU work, for benches that time a frame.
//! - og-probe is mounted by the runtime (`og_runtime`); this host adds
//!   its loopback WebSocket (`og_node_probe_listen`, `og_node_probe_serve`,
//!   og-probe's `ProbeServer` at `127.0.0.1:<port>/probe`), cooked files read
//!   from disk (`Hooks::read_asset`) and og-cook's Basis transcoder for
//!   textures (the `Hooks::texture` default with og-cook's `transcode`).

/// The exports, one per binding symbol (`og api gen`).
#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;
mod audio;
mod host;
mod host_depth;
mod host_e6;
mod host_p2;
#[cfg(not(feature = "publish"))]
mod render;
mod room;

use napi::bindgen_prelude::{FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue, ValueType};
use napi::{Env, sys};
use napi_derive::napi;
use og_runtime::{ColumnPtr, Dispatch, Fetch, Hooks, RenderBackend};
use std::cell::{Cell, RefCell};
use std::ptr;

/// The engine's heap, counted for the probe's `memory` (og-core's `alloc`).
#[global_allocator]
static HEAP: og_runtime::CountingAlloc<std::alloc::System> =
    og_runtime::CountingAlloc(std::alloc::System);

/// The native host: a unit type, since the engines live in the runtime's
/// thread-local tables.
pub struct NodeHost;

/// `ArchetypeView.entities`: a fresh `Uint32Array` of (index, generation).
pub type NodeArray = napi::bindgen_prelude::Uint32Array;

/// A resource's packed bytes, either way (`og_engine_read_resource`,
/// `og_engine_write_resource_bytes`): copied out of the `Uint8Array` the
/// runtime passes, and into a fresh one it is handed. A record is a few
/// hundred bytes, so the copies cost less than napi's typed array, which
/// keeps a reference to the JavaScript array until a later collection.
pub struct NodeBytes(pub Vec<u8>);

impl std::ops::Deref for NodeBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl TypeName for NodeBytes {
    fn type_name() -> &'static str {
        "Uint8Array"
    }
    fn value_type() -> ValueType {
        ValueType::Object
    }
}

impl ValidateNapiValue for NodeBytes {}

impl FromNapiValue for NodeBytes {
    unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
        unsafe {
            let mut ty = 0;
            let mut len = 0;
            let mut data = ptr::null_mut();
            let mut buffer = ptr::null_mut();
            let mut offset = 0;
            if !ok(sys::napi_get_typedarray_info(
                env,
                value,
                &mut ty,
                &mut len,
                &mut data,
                &mut buffer,
                &mut offset,
            )) || ty != sys::TypedarrayType::uint8_array
            {
                return Err(napi::Error::from_reason("expected a Uint8Array"));
            }
            if len == 0 || data.is_null() {
                return Ok(NodeBytes(Vec::new()));
            }
            Ok(NodeBytes(
                std::slice::from_raw_parts(data.cast::<u8>(), len).to_vec(),
            ))
        }
    }
}

impl ToNapiValue for NodeBytes {
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
        unsafe {
            let mut data = ptr::null_mut();
            let mut buffer = ptr::null_mut();
            if !ok(sys::napi_create_arraybuffer(
                env,
                val.0.len(),
                &mut data,
                &mut buffer,
            )) {
                return Err(napi::Error::from_reason("cannot allocate a Uint8Array"));
            }
            if !val.0.is_empty() {
                ptr::copy_nonoverlapping(val.0.as_ptr(), data.cast::<u8>(), val.0.len());
            }
            let mut array = ptr::null_mut();
            if !ok(sys::napi_create_typedarray(
                env,
                sys::TypedarrayType::uint8_array,
                val.0.len(),
                buffer,
                0,
                &mut array,
            )) {
                return Err(napi::Error::from_reason("cannot make a Uint8Array"));
            }
            Ok(array)
        }
    }
}

thread_local! {
    static ENV: Cell<sys::napi_env> = const { Cell::new(ptr::null_mut()) };
    static LIVE: RefCell<Vec<Kept>> = const { RefCell::new(Vec::new()) };
    static COPY: Cell<Option<bool>> = const { Cell::new(None) };
}

/// The env of the napi call in progress, for the runtime's call-backs.
pub struct EnvScope(sys::napi_env);

impl EnvScope {
    pub fn enter(env: &Env) -> EnvScope {
        EnvScope(ENV.with(|e| e.replace(env.raw())))
    }
}

impl Drop for EnvScope {
    fn drop(&mut self) {
        ENV.with(|e| e.set(self.0));
    }
}

fn env() -> sys::napi_env {
    ENV.with(Cell::get)
}

fn ok(status: sys::napi_status) -> bool {
    status == sys::Status::napi_ok
}

/// A JavaScript function the engine keeps (a strong reference).
pub struct NodeFunction(sys::napi_ref);

impl TypeName for NodeFunction {
    fn type_name() -> &'static str {
        "Function"
    }
    fn value_type() -> ValueType {
        ValueType::Function
    }
}

impl ValidateNapiValue for NodeFunction {}

impl FromNapiValue for NodeFunction {
    unsafe fn from_napi_value(env: sys::napi_env, value: sys::napi_value) -> napi::Result<Self> {
        unsafe {
            let mut r = ptr::null_mut();
            if !ok(sys::napi_create_reference(env, value, 1, &mut r)) {
                return Err(napi::Error::from_reason("cannot keep the dispatcher"));
            }
            Ok(NodeFunction(r))
        }
    }
}

/// One ArrayBuffer over (or copied from) a column, kept while the column
/// stays where it is, so a system's views cost no allocation tick after tick.
struct Kept {
    buffer: sys::napi_ref,
    /// The column it shows.
    column: *mut u8,
    len: usize,
    /// The JavaScript copy's memory, in copy mode.
    copy: Option<*mut u8>,
    /// Handed out since the last retirement, writable: copy back then.
    write_back: bool,
}

fn copy_mode() -> bool {
    COPY.with(|c| match c.get() {
        Some(v) => v,
        None => {
            let v = std::env::var("OG_NODE_VIEWS").is_ok_and(|v| v == "copy");
            c.set(Some(v));
            v
        }
    })
}

/// A buffer showing `col`: a kept one when the column has not moved, else a
/// new external ArrayBuffer over it (or a copy, where external memory is
/// refused or `OG_NODE_VIEWS=copy`).
unsafe fn buffer_for(env: sys::napi_env, col: &ColumnPtr) -> napi::Result<sys::napi_value> {
    unsafe {
        let found = LIVE.with(|l| {
            let mut l = l.borrow_mut();
            l.iter_mut()
                .find(|k| k.column == col.ptr && k.len == col.len)
                .map(|k| {
                    if let Some(js) = k.copy {
                        ptr::copy_nonoverlapping(col.ptr, js, col.len);
                        k.write_back |= col.writable;
                    }
                    k.buffer
                })
        });
        if let Some(r) = found {
            let mut buffer = ptr::null_mut();
            sys::napi_get_reference_value(env, r, &mut buffer);
            if !buffer.is_null() {
                return Ok(buffer);
            }
        }
        let mut buffer = ptr::null_mut();
        let mut copy = None;
        let mut aliased = false;
        if !copy_mode() && col.len > 0 {
            let status = sys::napi_create_external_arraybuffer(
                env,
                col.ptr.cast(),
                col.len,
                None,
                ptr::null_mut(),
                &mut buffer,
            );
            if ok(status) {
                aliased = true;
            } else {
                // The runtime refuses external memory: copy from now on.
                COPY.with(|c| c.set(Some(true)));
                let mut pending = false;
                sys::napi_is_exception_pending(env, &mut pending);
                if pending {
                    let mut e = ptr::null_mut();
                    sys::napi_get_and_clear_last_exception(env, &mut e);
                }
            }
        }
        if !aliased {
            let mut data = ptr::null_mut();
            if !ok(sys::napi_create_arraybuffer(
                env,
                col.len,
                &mut data,
                &mut buffer,
            )) {
                return Err(napi::Error::from_reason("cannot allocate a column buffer"));
            }
            if col.len > 0 {
                ptr::copy_nonoverlapping(col.ptr, data.cast::<u8>(), col.len);
                copy = Some(data.cast::<u8>());
            }
        }
        let mut r = ptr::null_mut();
        sys::napi_create_reference(env, buffer, 1, &mut r);
        LIVE.with(|l| {
            l.borrow_mut().push(Kept {
                buffer: r,
                column: col.ptr,
                len: col.len,
                copy,
                write_back: copy.is_some() && col.writable,
            })
        });
        Ok(buffer)
    }
}

/// `ArchetypeView.column`: one ArrayBuffer per field, in field order.
pub struct NodeViews(pub Vec<ColumnPtr>);

impl ToNapiValue for NodeViews {
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
        unsafe {
            let mut array = ptr::null_mut();
            sys::napi_create_array_with_length(env, val.0.len(), &mut array);
            for (i, col) in val.0.iter().enumerate() {
                let buffer = buffer_for(env, col)?;
                sys::napi_set_element(env, array, i as u32, buffer);
            }
            Ok(array)
        }
    }
}

/// Calls the installed JavaScript dispatcher and retires handed-out buffers.
pub(crate) struct NodeHooks {
    pub dispatcher: NodeFunction,
}

unsafe fn js_string(env: sys::napi_env, value: sys::napi_value) -> String {
    unsafe {
        let mut s = ptr::null_mut();
        if !ok(sys::napi_coerce_to_string(env, value, &mut s)) {
            return String::new();
        }
        let mut len = 0;
        sys::napi_get_value_string_utf8(env, s, ptr::null_mut(), 0, &mut len);
        let mut buf = vec![0u8; len + 1];
        sys::napi_get_value_string_utf8(env, s, buf.as_mut_ptr().cast(), len + 1, &mut len);
        buf.truncate(len);
        String::from_utf8_lossy(&buf).into_owned()
    }
}

impl Hooks for NodeHooks {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        let env = env();
        if env.is_null() {
            return Err("the dispatcher was called outside a napi call".into());
        }
        // SAFETY: `env` is the env of the napi call on this thread's stack.
        unsafe {
            let mut scope = ptr::null_mut();
            sys::napi_open_handle_scope(env, &mut scope);
            let result = (|| {
                let mut f = ptr::null_mut();
                sys::napi_get_reference_value(env, self.dispatcher.0, &mut f);
                let mut args = [ptr::null_mut(); 8];
                for (slot, v) in args.iter_mut().zip([
                    f64::from(call.kind),
                    f64::from(call.engine),
                    f64::from(call.id),
                    f64::from(call.query),
                    call.tick,
                    call.dt,
                    call.elapsed,
                ]) {
                    sys::napi_create_double(env, v, slot);
                }
                sys::napi_create_string_utf8(
                    env,
                    call.payload.as_ptr().cast(),
                    call.payload.len() as isize,
                    &mut args[7],
                );
                let mut recv = ptr::null_mut();
                sys::napi_get_undefined(env, &mut recv);
                let mut answer = ptr::null_mut();
                let status = sys::napi_call_function(env, recv, f, 8, args.as_ptr(), &mut answer);
                if !ok(status) {
                    let mut e = ptr::null_mut();
                    sys::napi_get_and_clear_last_exception(env, &mut e);
                    return Err(if e.is_null() {
                        "the dispatcher failed".to_string()
                    } else {
                        js_string(env, e)
                    });
                }
                let text = js_string(env, answer);
                match text.strip_prefix('=') {
                    Some(payload) => Ok(payload.to_string()),
                    None if text.is_empty() => Ok(String::new()),
                    None => Err(text),
                }
            })();
            sys::napi_close_handle_scope(env, scope);
            result
        }
    }

    fn invalidate_views(&self, moving: bool) {
        let env = env();
        LIVE.with(|l| {
            let mut l = l.borrow_mut();
            for k in l.iter_mut() {
                if let (Some(js), true) = (k.copy, k.write_back) {
                    // SAFETY: the column has not moved yet (the runtime calls
                    // this before anything moves it), and the reference keeps
                    // the JavaScript buffer alive.
                    unsafe { ptr::copy_nonoverlapping(js, k.column, k.len) };
                }
                k.write_back = false;
            }
            if !moving || env.is_null() {
                return;
            }
            for k in l.drain(..) {
                // SAFETY: `env` is the env of the napi call on this stack.
                unsafe {
                    if k.copy.is_none() {
                        // A kept typed array now reads as empty instead of
                        // reaching memory that is about to move.
                        let mut buffer = ptr::null_mut();
                        sys::napi_get_reference_value(env, k.buffer, &mut buffer);
                        if !buffer.is_null() {
                            sys::napi_detach_arraybuffer(env, buffer);
                        }
                    }
                    sys::napi_delete_reference(env, k.buffer);
                }
            }
        });
    }

    fn now_ms(&self) -> f64 {
        thread_local! {
            static START: std::time::Instant = std::time::Instant::now();
        }
        START.with(|s| s.elapsed().as_secs_f64() * 1000.0)
    }

    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }
}

/// `og run --trace`: turns the engine's trace on or off (og-runtime's
/// `trace.rs`).
#[napi(js_name = "og_node_trace")]
pub fn og_node_trace(engine: u32, on: bool) {
    og_runtime::trace_set(engine, on);
}

/// The trace's spans since the last take, as JSON (`{ now, events }`, the
/// events Chrome trace events on the engine's clock); `last` waits for the
/// GPU's frames in flight first.
#[napi(js_name = "og_node_trace_take")]
pub fn og_node_trace_take(engine: u32, last: bool) -> String {
    og_runtime::trace_take(engine, last)
}

/// Whether column views alias engine memory (`"alias"`) or are copied in
/// and out (`"copy"`), as decided so far.
#[napi(js_name = "og_node_views_mode")]
pub fn og_node_views_mode() -> String {
    if copy_mode() { "copy" } else { "alias" }.to_string()
}

struct OffscreenBackend {
    renderer: og_render::Renderer,
    tier: String,
    /// The last frame's `FrameStats::missing_glyphs`.
    missing: u32,
    /// The last frame's `FrameStats::triangles`.
    triangles: u64,
}

impl RenderBackend for OffscreenBackend {
    fn tier(&self) -> String {
        self.tier.clone()
    }
    fn texture_formats(&self) -> Option<og_render::TextureFormats> {
        Some(self.renderer.info().texture_formats.clone())
    }
    fn size(&self) -> (u32, u32) {
        self.renderer.size()
    }
    fn antialias(&self) -> String {
        og_runtime::antialias_name(self.renderer.info().antialias).into()
    }
    fn info(&self) -> Option<og_render::RenderInfo> {
        Some(self.renderer.info().clone())
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        self.renderer
            .render(scene, &og_render::RenderTarget::Offscreen)
            .map(|s| {
                self.missing = s.missing_glyphs;
                self.triangles = s.triangles;
            })
            .map_err(|e| e.to_string())
    }
    fn missing_glyphs(&self) -> u32 {
        self.missing
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
    fn triangles(&self) -> u64 {
        self.triangles
    }
    fn read_reflection_probe(&self, id: u64) -> Option<Result<og_render::ProbeCapture, String>> {
        self.renderer.read_reflection_probe(id)
    }
    fn post_errors(&self, scene: &og_render::RenderScene) -> Vec<(u32, String)> {
        self.renderer
            .post_errors(scene)
            .into_iter()
            .map(|(h, e)| (h.0, e))
            .collect()
    }
    fn capture(
        &mut self,
        scene: &og_render::RenderScene,
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
            .render(scene)
            .ok()
            .and_then(|_| self.renderer.read_offscreen().ok());
        if size != target {
            self.renderer.resize(target.0, target.1);
        }
        image
    }
    fn read_frame(&mut self) -> Option<og_render::RgbaImage> {
        self.renderer.read_offscreen().ok()
    }
    fn save_state(&mut self) -> u64 {
        self.renderer.save_state().id()
    }
    fn restore_state(&mut self, state: u64) {
        self.renderer
            .restore_state(og_render::RendererState::from_id(state));
    }
}

/// Give an engine a headless renderer of `width`×`height` (Metal or Vulkan),
/// created with the game's RenderOptions. A Result of the tier.
///
/// Without a renderer (no usable adapter, as in the platform's `engine` and
/// `probe` sandboxes, or `OG_RENDERER=none`) the answer is a refusal naming
/// why, and the engine runs on without one: its play surface is still
/// `width`×`height` (the viewport a renderer of that size would give, so
/// the UI lays out and the simulation runs exactly as with one), a
/// `screenshot` step notes its frame unchecked, and `capture` answers null.
#[napi(js_name = "og_node_attach_offscreen")]
pub fn og_node_attach_offscreen(env: Env, engine: u32, width: u32, height: u32) -> String {
    let _env = EnvScope::enter(&env);
    let desc = og_runtime::render_desc(
        engine,
        og_render::RenderDesc {
            width: width.max(1),
            height: height.max(1),
            // OG_GFX_CULLING=cpu culls on the CPU, as og-gfx does: lanes.test.ts
            // needs the CPU cull, which GPU culling skips.
            culling: if std::env::var("OG_GFX_CULLING").is_ok_and(|v| v == "cpu") {
                og_render::Culling::Cpu
            } else {
                og_render::Culling::Auto
            },
            ..og_render::RenderDesc::default()
        },
    );
    // Compiled pipelines kept across runs where the backend can (Vulkan):
    // OG_PIPELINE_CACHE names the directory, `off` keeps none; by default
    // ~/.cache/origin-engine/pipelines.
    og_render::set_pipeline_cache_dir(match std::env::var("OG_PIPELINE_CACHE") {
        Ok(v) if v == "off" => None,
        Ok(v) if !v.is_empty() => Some(v.into()),
        _ => std::env::var_os("HOME")
            .map(|h| std::path::PathBuf::from(h).join(".cache/origin-engine/pipelines")),
    });
    let answer = og_render::Renderer::new_blocking(&desc)
        .map_err(|e| e.to_string())
        .and_then(|renderer| {
            let tier = renderer.info().backend_tier();
            let device = renderer.device().clone();
            DEVICES.with(|d| d.borrow_mut().insert(engine, device));
            og_runtime::set_render_backend(
                engine,
                Some(Box::new(OffscreenBackend {
                    renderer,
                    tier: tier.clone(),
                    missing: 0,
                    triangles: 0,
                })),
            )?;
            Ok(tier)
        });
    match answer {
        Ok(tier) => format!(r#"{{"ok":true,"value":"{tier}"}}"#),
        Err(e) => {
            // The surface a renderer would have sized: rendering never
            // feeds back into simulation, so neither may its absence.
            let surface = serde_json::json!({ "width": width.max(1), "height": height.max(1) });
            let e = match og_runtime::set_viewport(engine, surface) {
                Ok(()) => e,
                Err(v) => format!("{e}; and the play surface: {v}"),
            };
            format!(r#"{{"ok":false,"reason":{}}}"#, napi_json_string(&e))
        }
    }
}

thread_local! {
    /// Each offscreen renderer's device, for og_node_wait_gpu.
    static DEVICES: RefCell<std::collections::BTreeMap<u32, og_render::wgpu::Device>> =
        const { RefCell::new(std::collections::BTreeMap::new()) };
}

/// Block until the GPU has finished everything an engine's offscreen
/// renderer submitted, so a frame's time includes its GPU work (benches).
/// A Result of null; refused for an engine without og_node_attach_offscreen.
#[napi(js_name = "og_node_wait_gpu")]
pub fn og_node_wait_gpu(engine: u32) -> String {
    let device = DEVICES.with(|d| d.borrow().get(&engine).cloned());
    let answer = match device {
        None => Err(format!(
            "engine {engine} has no offscreen renderer; call og_node_attach_offscreen first"
        )),
        Some(device) => device
            .poll(og_render::wgpu::PollType::wait_indefinitely())
            .map(|_| ())
            .map_err(|e| e.to_string()),
    };
    match answer {
        Ok(()) => r#"{"ok":true,"value":null}"#.to_string(),
        Err(e) => format!(r#"{{"ok":false,"reason":{}}}"#, napi_json_string(&e)),
    }
}

thread_local! {
    static SERVERS: RefCell<std::collections::BTreeMap<u32, og_probe::server::ProbeServer>> =
        const { RefCell::new(std::collections::BTreeMap::new()) };
}

/// Serve an engine's probe on `127.0.0.1:<port>/probe` (JSON-RPC 2.0 over
/// WebSocket; port 0 picks a free one). A Result of the port.
#[napi(js_name = "og_node_probe_listen")]
pub fn og_node_probe_listen(engine: u32, port: u32) -> String {
    let addr = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, port as u16));
    match og_probe::server::ProbeServer::bind(addr) {
        Ok(server) => {
            let port = server.port();
            SERVERS.with(|s| s.borrow_mut().insert(engine, server));
            format!(r#"{{"ok":true,"value":{port}}}"#)
        }
        Err(e) => format!(
            r#"{{"ok":false,"reason":{}}}"#,
            napi_json_string(&format!(
                "cannot listen on 127.0.0.1:{port}: {e}; pick another --port or stop the other og run"
            ))
        ),
    }
}

/// Answer every call queued on the engine's socket, between ticks. Returns
/// how many were answered (0 without a server).
#[napi(js_name = "og_node_probe_serve")]
pub fn og_node_probe_serve(env: Env, engine: u32) -> u32 {
    let _env = EnvScope::enter(&env);
    let mut answered = 0;
    loop {
        let pending = SERVERS.with(|s| s.borrow().get(&engine).and_then(|srv| srv.poll()));
        let Some(pending) = pending else { break };
        let frame = og_runtime::answer_call(engine, &pending.call);
        pending.answer(frame);
        answered += 1;
    }
    answered
}

/// Stop serving an engine's probe.
#[napi(js_name = "og_node_probe_close")]
pub fn og_node_probe_close(engine: u32) -> bool {
    SERVERS.with(|s| s.borrow_mut().remove(&engine).is_some())
}

fn napi_json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
