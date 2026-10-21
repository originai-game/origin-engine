//! The probe's `memory` (and the web Worker's `memory` call, which a publish
//! page answers without the probe's dev members): what the engine holds, in
//! one shape on both hosts.
//!
//! - `host`: `web` or `native`.
//! - `linear`: the wasm linear memory's size in bytes (what the page's
//!   engine takes from the browser, and never gives back); null natively.
//! - `resident`: the process's resident memory in bytes (Linux
//!   `/proc/self/status`, macOS `task_info`); null on the web, where a page
//!   cannot read its process's.
//! - `heap`: the engine's allocator ([`og_core::alloc::Counting`], which both
//!   hosts install): bytes `live`, their `peak` and the `allocations` made;
//!   null when the host counts nothing.
//! - `world`: `entities` alive.
//! - `stream`: the streaming budget (`og.stream.budget`): `budget`, `used`,
//!   `peak` in bytes.
//! - `audio`: the sound bank's decoded and resident samples, in bytes.

use super::engine::engine;
use og_ecs::World;
use serde_json::{Value, json};

/// The report for engine `id` over its `world` (the probe's hook passes the
/// world it holds).
pub(crate) fn report(id: u32, world: &World) -> Value {
    let (stream, audio) = match engine(id) {
        Ok(eng) => (
            super::stream::memory(&eng),
            og_audio::link(id).map(|l| l.bank().resident_bytes() as f64),
        ),
        Err(_) => (Value::Null, None),
    };
    json!({
        "host": if cfg!(target_arch = "wasm32") { "web" } else { "native" },
        "linear": linear(),
        "resident": resident(),
        "heap": og_core::alloc::heap().map(|h| json!({
            "live": h.live as f64,
            "peak": h.peak as f64,
            "allocations": h.allocations as f64,
        })),
        "world": { "entities": world.entity_count() },
        "stream": stream,
        "audio": audio,
    })
}

/// The report for engine `id`, outside a probe call (the web Worker's
/// `memory`, `og_web_memory`): null for an unknown engine, or while a system
/// runs.
pub fn memory_report(id: u32) -> Value {
    let Ok(eng) = engine(id) else {
        return Value::Null;
    };
    let Ok(world) = eng.world() else {
        return Value::Null;
    };
    report(id, world)
}

/// The wasm linear memory, in bytes.
fn linear() -> Option<f64> {
    #[cfg(target_arch = "wasm32")]
    {
        Some(core::arch::wasm32::memory_size(0) as f64 * 65536.0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// The process's resident memory, in bytes.
fn resident() -> Option<f64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        let line = status.lines().find(|l| l.starts_with("VmRSS:"))?;
        let kb: f64 = line.split_whitespace().nth(1)?.parse().ok()?;
        Some(kb * 1024.0)
    }
    #[cfg(target_os = "macos")]
    {
        mach::resident().map(|b| b as f64)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        None
    }
}

#[cfg(target_os = "macos")]
mod mach {
    /// `mach_task_basic_info` (mach/task_info.h, `#pragma pack(4)`).
    #[repr(C, packed(4))]
    #[derive(Default)]
    struct BasicInfo {
        virtual_size: u64,
        resident_size: u64,
        resident_size_max: u64,
        user_time: [i32; 2],
        system_time: [i32; 2],
        policy: i32,
        suspend_count: i32,
    }

    const MACH_TASK_BASIC_INFO: u32 = 20;

    unsafe extern "C" {
        static mach_task_self_: u32;
        fn task_info(task: u32, flavor: u32, info: *mut i32, count: *mut u32) -> i32;
    }

    /// This task's resident size, in bytes.
    pub(super) fn resident() -> Option<u64> {
        let mut info = BasicInfo::default();
        let mut count = (size_of::<BasicInfo>() / size_of::<i32>()) as u32;
        // SAFETY: `info` is a `mach_task_basic_info` of `count` words, as
        // MACH_TASK_BASIC_INFO asks; `mach_task_self_` is this task's port,
        // set by the system before main.
        let r = unsafe {
            task_info(
                mach_task_self_,
                MACH_TASK_BASIC_INFO,
                (&raw mut info).cast(),
                &mut count,
            )
        };
        let size = info.resident_size;
        (r == 0).then_some(size)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_native_process_reads_its_resident_memory() {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let r = super::resident().expect("resident memory");
            assert!(r > 1e6, "{r} bytes resident");
        }
        assert!(super::linear().is_none());
    }
}
