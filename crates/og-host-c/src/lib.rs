//! Origin Engine C ABI host: every binding symbol of
//! `crates/og-api/api/<module>.ogt` as an `extern "C"` entry, for a native
//! shell that embeds its own JavaScript engine (Hermes on a store package;
//! `hermes/` is the prototype shell). The third host of `docs/design.md`
//! §2.1: it adds a wire, not a copy; every binding is
//! `og_runtime::host_impl`'s.
//!
//! - `src/generated/glue.rs` (`og api gen`) holds one entry per symbol,
//!   `fn(args: *const OgValue, out: *mut OgValue)`, and `SYMBOLS`, the table
//!   [`og_c_symbols`] hands the shell.
//! - An answer's memory (a string, bytes, an array, views) is this host's
//!   and stays valid until the next entry is called on the thread: the
//!   shell copies it into its engine's values first.
//! - Column views alias engine memory, one `OgColumn` per field; before a
//!   column moves the host calls the shell's `invalidate`, which detaches
//!   the buffers it made over them.
//! - The shared buffer is this host's memory, installed once per thread.

#[doc(hidden)]
#[rustfmt::skip]
#[path = "generated/glue.rs"]
pub mod glue;

use og_api::host::Wire;
use og_runtime as rt;
use og_runtime::host_impl::HostWire;
use og_runtime::{ColumnPtr, Dispatch, Fetch, Hooks};
use std::cell::RefCell;
use std::ffi::{CStr, c_void};
use std::rc::Rc;

/// One argument or answer.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct OgValue {
    pub tag: u32,
    /// Elements behind `ptr` (bytes, words or columns).
    pub len: u32,
    /// A bool (0 or 1), a number or a handle.
    pub num: f64,
    pub ptr: *const c_void,
}

pub const OG_UNDEFINED: u32 = 0;
pub const OG_BOOL: u32 = 1;
pub const OG_NUMBER: u32 = 2;
/// UTF-8 text, `len` bytes.
pub const OG_STRING: u32 = 3;
/// A `Uint8Array`, `len` bytes.
pub const OG_BYTES: u32 = 4;
/// A `Uint32Array`, `len` words.
pub const OG_U32_ARRAY: u32 = 5;
/// Column views: `len` `OgColumn`s.
pub const OG_VIEWS: u32 = 6;
/// The dispatcher: `ptr` is an `OgDispatcher` the shell keeps alive.
pub const OG_FUNCTION: u32 = 7;
/// The shared buffer: `len` is its size in bytes.
pub const OG_SHARED: u32 = 8;

impl OgValue {
    const UNDEFINED: OgValue = OgValue {
        tag: OG_UNDEFINED,
        len: 0,
        num: 0.0,
        ptr: std::ptr::null(),
    };
    fn number(num: f64) -> Self {
        OgValue {
            tag: OG_NUMBER,
            num,
            ..Self::UNDEFINED
        }
    }
    fn text(&self) -> String {
        if self.tag != OG_STRING || self.ptr.is_null() {
            return String::new();
        }
        // SAFETY: the shell's string is `len` bytes at `ptr` for the call.
        let bytes = unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len as usize) };
        String::from_utf8_lossy(bytes).into_owned()
    }
}

/// One column handed to the shell: `len` bytes at `ptr`.
#[repr(C)]
pub struct OgColumn {
    pub ptr: *mut u8,
    pub len: usize,
    pub writable: bool,
}

/// The shell's dispatcher (`og_host_set_dispatcher`).
#[repr(C)]
pub struct OgDispatcher {
    pub ctx: *mut c_void,
    /// Runs game code; writes its answer (`""`, `"=..."` or an error) as
    /// UTF-8 to `*answer`/`*answer_len`, valid until the next call.
    pub call: unsafe extern "C" fn(
        ctx: *mut c_void,
        kind: u32,
        engine: u32,
        id: u32,
        query: u32,
        tick: f64,
        dt: f64,
        elapsed: f64,
        payload: *const u8,
        payload_len: usize,
        answer: *mut *const u8,
        answer_len: *mut usize,
    ),
    /// The column views handed out are about to move (`moving`) or only to
    /// be retired: the shell detaches the buffers it made over them.
    pub invalidate: unsafe extern "C" fn(ctx: *mut c_void, moving: bool),
}

/// An entry of the symbol table.
#[repr(C)]
pub struct Symbol {
    pub name: &'static CStr,
    pub arity: usize,
    pub call: unsafe extern "C" fn(args: *const OgValue, out: *mut OgValue),
}

/// The C table entry as C sees it (a `const char *` name).
#[repr(C)]
pub struct OgSymbol {
    pub name: *const std::ffi::c_char,
    pub arity: usize,
    pub call: unsafe extern "C" fn(args: *const OgValue, out: *mut OgValue),
}

/// The engine's heap, counted for the probe's `memory`.
#[global_allocator]
static HEAP: og_runtime::CountingAlloc<std::alloc::System> =
    og_runtime::CountingAlloc(std::alloc::System);

pub struct CHost;
pub struct CArray(pub Vec<u32>);
pub struct CViews(pub Vec<ColumnPtr>);
pub struct CFunction(pub *const OgDispatcher);
pub struct CBytes(pub Vec<u8>);
pub struct CShared;

impl std::ops::Deref for CBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl Wire for CHost {
    type Array = CArray;
    type Views = CViews;
    type Function = CFunction;
    type Bytes = CBytes;
    type Shared = CShared;
}

impl HostWire for CHost {
    fn install(dispatcher: CFunction) {
        rt::set_hooks(Rc::new(CHooks {
            dispatcher: dispatcher.0,
        }));
    }
    fn array(values: Vec<u32>) -> CArray {
        CArray(values)
    }
    fn views(columns: Result<Vec<ColumnPtr>, String>) -> CViews {
        CViews(columns.unwrap_or_default())
    }
    fn shared() -> CShared {
        CShared
    }
    fn bytes_out(bytes: Vec<u8>) -> CBytes {
        CBytes(bytes)
    }
    fn bytes_in(bytes: CBytes) -> Vec<u8> {
        bytes.0
    }
    fn destroyed(_engine: u32) {}
}

og_runtime::host_bindings!(CHost);

struct CHooks {
    dispatcher: *const OgDispatcher,
}

impl Hooks for CHooks {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        // SAFETY: the shell keeps its dispatcher alive while engines run.
        let d = unsafe { &*self.dispatcher };
        let mut answer = std::ptr::null();
        let mut len = 0usize;
        unsafe {
            (d.call)(
                d.ctx,
                call.kind,
                call.engine,
                call.id,
                call.query,
                call.tick,
                call.dt,
                call.elapsed,
                call.payload.as_ptr(),
                call.payload.len(),
                &mut answer,
                &mut len,
            );
        }
        let text = if answer.is_null() || len == 0 {
            String::new()
        } else {
            // SAFETY: the shell's answer is `len` bytes, valid until its next call.
            String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(answer, len) }).into_owned()
        };
        Dispatch::answer(text)
    }

    fn pump_platform(&self, engine: u32) -> Result<(), String> {
        self.dispatch(&Dispatch::platform_pump(engine)).map(|_| ())
    }

    fn invalidate_views(&self, moving: bool) {
        let d = unsafe { &*self.dispatcher };
        unsafe { (d.invalidate)(d.ctx, moving) };
    }

    fn now_ms(&self) -> f64 {
        thread_local! { static START: std::time::Instant = std::time::Instant::now(); }
        START.with(|s| s.elapsed().as_secs_f64() * 1000.0)
    }

    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }

    fn fetching(&self, _path: &str) -> bool {
        false
    }
}

// ── arguments ─────────────────────────────────────────────────────────────

/// A wire type read from an `OgValue`.
pub trait FromC: Sized {
    fn from_c(v: &OgValue) -> Self;
}

impl FromC for u32 {
    fn from_c(v: &OgValue) -> Self {
        v.num as u32
    }
}
impl FromC for i32 {
    fn from_c(v: &OgValue) -> Self {
        v.num as i32
    }
}
impl FromC for f64 {
    fn from_c(v: &OgValue) -> Self {
        v.num
    }
}
impl FromC for bool {
    fn from_c(v: &OgValue) -> Self {
        v.num != 0.0
    }
}
impl FromC for String {
    fn from_c(v: &OgValue) -> Self {
        v.text()
    }
}
impl FromC for CBytes {
    fn from_c(v: &OgValue) -> Self {
        if v.ptr.is_null() || v.len == 0 {
            return CBytes(Vec::new());
        }
        // SAFETY: the shell's bytes are `len` long for the call.
        CBytes(unsafe { std::slice::from_raw_parts(v.ptr.cast::<u8>(), v.len as usize) }.to_vec())
    }
}
impl FromC for CFunction {
    fn from_c(v: &OgValue) -> Self {
        CFunction(v.ptr.cast())
    }
}
impl<T: FromC> FromC for Option<T> {
    fn from_c(v: &OgValue) -> Self {
        (v.tag != OG_UNDEFINED).then(|| T::from_c(v))
    }
}

/// The `i`th argument.
///
/// # Safety
/// `args` holds at least `i + 1` values.
pub unsafe fn arg<T: FromC>(args: *const OgValue, i: usize) -> T {
    T::from_c(unsafe { &*args.add(i) })
}

// ── answers ───────────────────────────────────────────────────────────────

/// What the last answer points into, kept until the next entry.
enum Kept {
    None,
    Text(String),
    Bytes(Vec<u8>),
    Words(Vec<u32>),
    Columns(Vec<OgColumn>),
}

thread_local! {
    static ANSWER: RefCell<Kept> = const { RefCell::new(Kept::None) };
}

/// A wire type written to an `OgValue`.
pub trait IntoC {
    /// # Safety
    /// `out` is valid for one write.
    unsafe fn into_c(self, out: *mut OgValue);
}

fn keep(k: Kept, tag: u32, len: usize, out: *mut OgValue) {
    ANSWER.with(|a| {
        let mut a = a.borrow_mut();
        *a = k;
        let ptr: *const c_void = match &*a {
            Kept::Text(s) => s.as_ptr().cast(),
            Kept::Bytes(b) => b.as_ptr().cast(),
            Kept::Words(w) => w.as_ptr().cast(),
            Kept::Columns(c) => c.as_ptr().cast(),
            Kept::None => std::ptr::null(),
        };
        // SAFETY: the caller's `out`.
        unsafe {
            *out = OgValue {
                tag,
                len: len as u32,
                num: 0.0,
                ptr,
            }
        };
    });
}

impl IntoC for () {
    unsafe fn into_c(self, out: *mut OgValue) {
        unsafe { *out = OgValue::UNDEFINED };
    }
}
impl IntoC for bool {
    unsafe fn into_c(self, out: *mut OgValue) {
        unsafe {
            *out = OgValue {
                tag: OG_BOOL,
                num: f64::from(u8::from(self)),
                ..OgValue::UNDEFINED
            }
        };
    }
}
impl IntoC for u32 {
    unsafe fn into_c(self, out: *mut OgValue) {
        unsafe { *out = OgValue::number(f64::from(self)) };
    }
}
impl IntoC for i32 {
    unsafe fn into_c(self, out: *mut OgValue) {
        unsafe { *out = OgValue::number(f64::from(self)) };
    }
}
impl IntoC for f64 {
    unsafe fn into_c(self, out: *mut OgValue) {
        unsafe { *out = OgValue::number(self) };
    }
}
impl IntoC for String {
    unsafe fn into_c(self, out: *mut OgValue) {
        let len = self.len();
        keep(Kept::Text(self), OG_STRING, len, out);
    }
}
impl IntoC for CBytes {
    unsafe fn into_c(self, out: *mut OgValue) {
        let len = self.0.len();
        keep(Kept::Bytes(self.0), OG_BYTES, len, out);
    }
}
impl IntoC for CArray {
    unsafe fn into_c(self, out: *mut OgValue) {
        let len = self.0.len();
        keep(Kept::Words(self.0), OG_U32_ARRAY, len, out);
    }
}
impl IntoC for CViews {
    unsafe fn into_c(self, out: *mut OgValue) {
        let cols: Vec<OgColumn> = self
            .0
            .iter()
            .map(|c| OgColumn {
                ptr: c.ptr,
                len: c.len,
                writable: c.writable,
            })
            .collect();
        let len = cols.len();
        keep(Kept::Columns(cols), OG_VIEWS, len, out);
    }
}
impl IntoC for CShared {
    unsafe fn into_c(self, out: *mut OgValue) {
        let ptr = rt::shared::address();
        unsafe {
            *out = OgValue {
                tag: OG_SHARED,
                len: rt::shared::SHARED_BYTES as u32,
                num: 0.0,
                ptr: ptr.cast(),
            }
        };
    }
}

// ── the table ─────────────────────────────────────────────────────────────

/// The symbol table: `*count` entries, valid for the process's life.
///
/// # Safety
/// `count` is valid for one write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_c_symbols(count: *mut usize) -> *const OgSymbol {
    thread_local! {
        static TABLE: Vec<OgSymbol> = glue::SYMBOLS
            .iter()
            .map(|s| OgSymbol { name: s.name.as_ptr(), arity: s.arity, call: s.call })
            .collect();
    }
    TABLE.with(|t| {
        unsafe { *count = t.len() };
        t.as_ptr()
    })
}
