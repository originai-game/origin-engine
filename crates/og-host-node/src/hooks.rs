//! The runtime's hooks on this host: the JavaScript dispatcher, the
//! retirement of handed-out column buffers, files and textures.

use crate::resident;
use crate::wire::{LIVE, NodeFunction, env, ok};
use napi::sys;
use og_runtime::{Dispatch, Fetch, Hooks};
use std::ptr;

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
    fn pump_platform(&self, engine: u32) -> Result<(), String> {
        self.dispatch(&Dispatch::platform_pump(engine)).map(|_| ())
    }

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
                Dispatch::answer(js_string(env, answer))
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
        match std::fs::read(self.asset_file(path)) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(e.to_string()),
        }
    }

    fn asset_file(&self, path: &str) -> String {
        resident::asset_file(path)
    }
}
