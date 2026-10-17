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
//! - `og_node_attach_offscreen` gives an engine an offscreen renderer; the
//!   probe's `capture` reads it back (`og_render::Renderer::read_offscreen`).

/// The exports, one per binding symbol (`og api gen`).
#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;
mod host;

use napi::bindgen_prelude::{FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue, ValueType};
use napi::{sys, Env};
use napi_derive::napi;
use og_api::runtime::{ColumnPtr, Dispatch, Hooks, RenderBackend};
use std::cell::{Cell, RefCell};
use std::ptr;

/// The native host: a unit type, since the engines live in the runtime's
/// thread-local tables.
pub struct NodeHost;

/// `ArchetypeView.entities`: a fresh `Uint32Array` of (index, generation).
pub type NodeArray = napi::bindgen_prelude::Uint32Array;

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
        let mut r = ptr::null_mut();
        if !ok(sys::napi_create_reference(env, value, 1, &mut r)) {
            return Err(napi::Error::from_reason("cannot keep the dispatcher"));
        }
        Ok(NodeFunction(r))
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

/// `ArchetypeView.column`: one ArrayBuffer per field, in field order.
pub struct NodeViews(pub Vec<ColumnPtr>);

impl ToNapiValue for NodeViews {
    unsafe fn to_napi_value(env: sys::napi_env, val: Self) -> napi::Result<sys::napi_value> {
        let mut array = ptr::null_mut();
        sys::napi_create_array_with_length(env, val.0.len(), &mut array);
        for (i, col) in val.0.iter().enumerate() {
            let buffer = buffer_for(env, col)?;
            sys::napi_set_element(env, array, i as u32, buffer);
        }
        Ok(array)
    }
}

/// Calls the installed JavaScript dispatcher and retires handed-out buffers.
pub(crate) struct NodeHooks {
    pub dispatcher: NodeFunction,
}

unsafe fn js_string(env: sys::napi_env, value: sys::napi_value) -> String {
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
}

impl RenderBackend for OffscreenBackend {
    fn tier(&self) -> String {
        self.tier.clone()
    }
    fn size(&self) -> (u32, u32) {
        self.renderer.size()
    }
    fn render(&mut self, scene: &og_render::RenderScene) -> Result<(), String> {
        self.renderer
            .render(scene, &og_render::RenderTarget::Offscreen)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
    fn capture(&mut self, scene: &og_render::RenderScene) -> Option<og_render::RgbaImage> {
        self.render(scene).ok()?;
        self.renderer.read_offscreen().ok()
    }
}

/// Give an engine a headless renderer of `width`×`height` (Metal or Vulkan,
/// lavapipe in executors). A Result of the tier.
#[napi(js_name = "og_node_attach_offscreen")]
pub fn og_node_attach_offscreen(env: Env, engine: u32, width: u32, height: u32) -> String {
    let _env = EnvScope::enter(&env);
    let desc = og_render::RenderDesc {
        width: width.max(1),
        height: height.max(1),
        ..og_render::RenderDesc::default()
    };
    let answer = og_render::Renderer::new_blocking(&desc)
        .map_err(|e| e.to_string())
        .and_then(|renderer| {
            let tier = renderer.info().backend_tier();
            og_api::runtime::set_render_backend(
                engine,
                Some(Box::new(OffscreenBackend {
                    renderer,
                    tier: tier.clone(),
                })),
            )?;
            Ok(tier)
        });
    match answer {
        Ok(tier) => format!(r#"{{"ok":true,"value":"{tier}"}}"#),
        Err(e) => format!(r#"{{"ok":false,"reason":{}}}"#, napi_json_string(&e)),
    }
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
