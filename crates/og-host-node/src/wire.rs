//! The napi wire types: what crosses between JavaScript and the engine
//! (bytes, the dispatcher, the shared buffer, column views) and the
//! call's env.

// napi registers the exports outside `cfg(test)` only.
#![cfg_attr(test, allow(dead_code))]

use napi::bindgen_prelude::{FromNapiValue, ToNapiValue, TypeName, ValidateNapiValue, ValueType};
use napi::{Env, sys};
use napi_derive::napi;
use og_runtime::ColumnPtr;
use std::cell::{Cell, RefCell};
use std::ptr;

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
    pub(crate) static LIVE: RefCell<Vec<Kept>> = const { RefCell::new(Vec::new()) };
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

pub(crate) fn env() -> sys::napi_env {
    ENV.with(Cell::get)
}

pub(crate) fn ok(status: sys::napi_status) -> bool {
    status == sys::Status::napi_ok
}

/// A JavaScript function the engine keeps (a strong reference).
pub struct NodeFunction(pub(crate) sys::napi_ref);

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

/// `og_host_shared`: the shared buffer as an `ArrayBuffer` of JavaScript's
/// own heap (no runtime refuses that, as some do external memory), made
/// once per thread, kept by a strong reference and installed as the
/// runtime's (`og_runtime::shared`).
pub struct NodeShared;

thread_local! {
    static SHARED: Cell<sys::napi_ref> = const { Cell::new(ptr::null_mut()) };
}

impl ToNapiValue for NodeShared {
    unsafe fn to_napi_value(env: sys::napi_env, _: Self) -> napi::Result<sys::napi_value> {
        unsafe {
            let mut buffer = ptr::null_mut();
            let kept = SHARED.with(Cell::get);
            if !kept.is_null() {
                sys::napi_get_reference_value(env, kept, &mut buffer);
                if !buffer.is_null() {
                    return Ok(buffer);
                }
            }
            let mut data = ptr::null_mut();
            let mut kept = ptr::null_mut();
            if !ok(sys::napi_create_arraybuffer(
                env,
                og_runtime::shared::SHARED_BYTES,
                &mut data,
                &mut buffer,
            )) || data.is_null()
                || !ok(sys::napi_create_reference(env, buffer, 1, &mut kept))
            {
                return Err(napi::Error::from_reason(
                    "cannot allocate the shared buffer",
                ));
            }
            // SAFETY: the reference keeps the buffer, and so its memory, for
            // the thread's life; only this thread's JavaScript touches it.
            og_runtime::shared::install(data.cast());
            SHARED.with(|s| s.set(kept));
            Ok(buffer)
        }
    }
}

/// One ArrayBuffer over (or copied from) a column, kept while the column
/// stays where it is, so a system's views cost no allocation tick after tick.
pub(crate) struct Kept {
    pub(crate) buffer: sys::napi_ref,
    /// The column it shows.
    pub(crate) column: *mut u8,
    pub(crate) len: usize,
    /// The JavaScript copy's memory, in copy mode.
    pub(crate) copy: Option<*mut u8>,
    /// Handed out since the last retirement, writable: copy back then.
    pub(crate) write_back: bool,
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

/// Whether column views alias engine memory (`"alias"`) or are copied in
/// and out (`"copy"`), as decided so far.
#[napi(js_name = "og_node_views_mode")]
pub fn og_node_views_mode() -> String {
    if copy_mode() { "copy" } else { "alias" }.to_string()
}
