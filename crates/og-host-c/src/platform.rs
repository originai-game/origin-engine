//! Platform services for a store package's shell: the player's save
//! document in their data directory, and the store's provider library.
//!
//! A provider library (a Steam bridge over steam_api, say) exports one C
//! entry, `const char *og_provider_call(const char *method, const char
//! *json)`: a method name and its arguments as JSON, the answer as JSON
//! (NULL for none), valid until the next call. The game's bundled provider
//! (`packages/engine/src/hermes/platform.ts`) names the methods; the engine
//! never links the store.

use std::cell::RefCell;
use std::ffi::{CStr, CString, c_char};
use std::path::PathBuf;

type ProviderCall = unsafe extern "C" fn(*const c_char, *const c_char) -> *const c_char;

thread_local! {
    static PROVIDER: RefCell<Option<(libloading::Library, ProviderCall)>> = const { RefCell::new(None) };
    static ANSWER: RefCell<CString> = RefCell::new(CString::default());
}

fn answer(text: String) -> *const c_char {
    ANSWER.with(|a| {
        *a.borrow_mut() = CString::new(text).unwrap_or_default();
        a.borrow().as_ptr()
    })
}

/// SAFETY: `s` is a NUL-terminated string.
unsafe fn text(s: *const c_char) -> String {
    unsafe { CStr::from_ptr(s) }.to_string_lossy().into_owned()
}

/// The game's directory in the player's data directory: `OG_DATA_DIR`, else
/// Application Support (macOS), %APPDATA% (Windows) or the XDG data home.
fn data_dir(game: &str) -> Option<PathBuf> {
    let name: String = game
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || " -_.".contains(c) {
                c
            } else {
                '-'
            }
        })
        .collect();
    let name = name.trim_matches(|c| c == '.' || c == ' ');
    if name.is_empty() {
        return None;
    }
    let var = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    let base = var("OG_DATA_DIR").or_else(|| {
        if cfg!(windows) {
            var("APPDATA")
        } else if cfg!(target_os = "macos") {
            var("HOME").map(|h| h.join("Library/Application Support"))
        } else {
            var("XDG_DATA_HOME").or_else(|| var("HOME").map(|h| h.join(".local/share")))
        }
    })?;
    Some(base.join(name))
}

/// The game's saved platform document (`<data>/<game>/platform.json`), NULL
/// when there is none; valid until the next call.
///
/// # Safety
/// `game` is a NUL-terminated UTF-8 string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_c_data_load(game: *const c_char) -> *const c_char {
    // SAFETY: the caller's string.
    let game = unsafe { text(game) };
    match data_dir(&game).and_then(|d| std::fs::read_to_string(d.join("platform.json")).ok()) {
        Some(t) => answer(t),
        None => std::ptr::null(),
    }
}

/// Saves the game's platform document, through a temporary file so a crash
/// mid-write keeps the last one; false when it could not.
///
/// # Safety
/// `game` and `document` are NUL-terminated UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_c_data_save(game: *const c_char, document: *const c_char) -> bool {
    // SAFETY: the caller's strings.
    let (game, document) = unsafe { (text(game), text(document)) };
    let Some(dir) = data_dir(&game) else {
        return false;
    };
    let temp = dir.join("platform.json.tmp");
    std::fs::create_dir_all(&dir).is_ok()
        && std::fs::write(&temp, document).is_ok()
        && std::fs::rename(&temp, dir.join("platform.json")).is_ok()
}

/// Loads the provider library at `path`; answers `{"ok":true}` or
/// `{"ok":false,"reason":…}`, valid until the next call.
///
/// # Safety
/// `path` is a NUL-terminated UTF-8 string; the library's initialisers run.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_c_provider_open(path: *const c_char) -> *const c_char {
    // SAFETY: the caller's string.
    let path = unsafe { text(path) };
    // SAFETY: loading runs the library's initialisers; the package chose it.
    let opened = unsafe { libloading::Library::new(&path) }.and_then(|lib| {
        // SAFETY: the provider contract's signature.
        let call = unsafe { lib.get::<ProviderCall>(b"og_provider_call\0") }.map(|s| *s)?;
        Ok((lib, call))
    });
    let reply = match opened {
        Ok(p) => {
            PROVIDER.with(|s| *s.borrow_mut() = Some(p));
            serde_json::json!({ "ok": true })
        }
        Err(e) => serde_json::json!({ "ok": false, "reason": format!("{path}: {e}") }),
    };
    answer(reply.to_string())
}

/// Calls the open provider library's `og_provider_call`; its answer (NULL
/// without a library or an answer), valid until the next call.
///
/// # Safety
/// `method` and `json` are NUL-terminated UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn og_c_provider_call(
    method: *const c_char,
    json: *const c_char,
) -> *const c_char {
    PROVIDER.with(|s| match s.borrow().as_ref() {
        // SAFETY: the library is open; the strings are the caller's.
        Some((_, call)) => {
            let out = unsafe { call(method, json) };
            if out.is_null() {
                std::ptr::null()
            } else {
                answer(unsafe { text(out) })
            }
        }
        None => std::ptr::null(),
    })
}
