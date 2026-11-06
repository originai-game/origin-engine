//! Adapter-aware libtest registration. A GPU test uses `#[ignore = "requires GPU"]`;
//! the runner enables it only after wgpu finds the requested adapter. The
//! descriptor stays ignored without one, including discovery by nextest.
extern crate test;

use crate::gpu_wgpu as wgpu;
use test::{RunIgnored, TestDescAndFn, TestList, TestListOrder};

pub fn run(tests: &[&TestDescAndFn]) {
    let args: Vec<_> = std::env::args().collect();
    let Some(Ok(mut opts)) = test::test::parse_opts(&args) else {
        std::process::exit(
            if test::test_main(&args, tests) == std::process::ExitCode::SUCCESS {
                0
            } else {
                1
            },
        );
    };
    // Apply name filters before probing: an individual CPU test never opens wgpu.
    let selected = |t: &&TestDescAndFn| {
        let name = t.desc.name.as_slice();
        let matches = |filter: &String| {
            if opts.filter_exact {
                name == filter
            } else {
                name.contains(filter)
            }
        };
        (opts.filters.is_empty() || opts.filters.iter().any(matches))
            && !opts.skip.iter().any(matches)
    };
    let gpu = |t: &TestDescAndFn| {
        t.desc
            .ignore_message
            .is_some_and(|m| m.starts_with("requires GPU"))
    };
    let error = if tests.iter().copied().filter(selected).any(gpu) {
        adapter_error()
    } else {
        None
    };
    let reason = error.map(|e| &*Box::leak(e.into_boxed_str()));
    let mut owned: Vec<_> = tests
        .iter()
        .map(|t| {
            let mut t = (*t).clone();
            if gpu(&t) {
                if let Some(reason) = reason {
                    t.desc.ignore = true;
                    t.desc.ignore_message = Some(reason);
                } else if t.desc.ignore_message == Some("requires GPU") {
                    t.desc.ignore = false;
                    t.desc.ignore_message = None;
                }
            }
            t
        })
        .collect();
    let refs: Vec<_> = owned.iter().collect();
    if let Some(reason) = reason {
        let unexpected = if disabled() { "" } else { "unexpected " };
        let skipped = refs
            .iter()
            .copied()
            .filter(selected)
            .filter(|t| t.desc.ignore_message == Some(reason))
            .count();
        let package = env!("CARGO_PKG_NAME");
        let warning = format!(
            "WARNING {unexpected}missing GPU: {reason}; {skipped} GPU tests skipped ({package}), never passed\n"
        );
        if opts.list {
            eprint!("{warning}");
        }
        // nextest captures successful discovery's stderr. The gate prints
        // these diagnostics and never caches an unchecked GPU run as green.
        if let Some(dir) = std::env::var_os("OG_GPU_REPORT_DIR") {
            let path = std::path::PathBuf::from(dir).join(format!(
                "{}-{}.txt",
                env!("CARGO_PKG_NAME"),
                std::process::id()
            ));
            std::fs::write(path, warning).expect("write GPU discovery report");
        }
    }
    if opts.list {
        // Keep libtest's discovery protocol, including --ignored, intact.
        std::process::exit(
            if test::test_main(&args, &refs) == std::process::ExitCode::SUCCESS {
                0
            } else {
                1
            },
        );
    }
    owned = test::filter_tests(&opts, TestList::new(&refs, TestListOrder::Unsorted));
    let mut skipped = 0;
    if let Some(reason) = reason {
        for t in &mut owned {
            if t.desc.ignore_message == Some(reason) {
                // --ignored/--include-ignored may select unavailable tests, but
                // must not turn an unchecked body into a successful test.
                t.desc.ignore = true;
                skipped += 1;
            }
        }
        let unexpected = if disabled() { "" } else { "unexpected " };
        eprintln!(
            "WARNING {unexpected}missing GPU: {reason}; {skipped} GPU tests skipped, never passed"
        );
        if skipped > 0 && std::env::var_os("NEXTEST").is_some() {
            eprintln!(
                "unsupported nextest invocation: do not force unavailable ignored GPU tests with --run-ignored; nextest counts a runtime skip as passed. Use cargo test -- --ignored instead, which reports these tests as ignored."
            );
            std::process::exit(2);
        }
    }
    opts.run_ignored = RunIgnored::No;
    let refs: Vec<_> = owned.iter().collect();
    match test::run_tests_console(&opts, TestList::new(&refs, TestListOrder::Unsorted)) {
        Ok(true) => {}
        Ok(false) => std::process::exit(1),
        Err(e) => {
            eprintln!("test runner: {e}");
            std::process::exit(1);
        }
    }
}

fn disabled() -> bool {
    std::env::var("OG_RENDERER").is_ok_and(|v| v.trim().eq_ignore_ascii_case("none"))
}

fn adapter_error() -> Option<String> {
    if disabled() {
        return Some("no GPU adapter: OG_RENDERER=none (adapter explicitly disabled)".into());
    }
    let backends = if cfg!(windows) {
        wgpu::Backends::DX12
    } else {
        wgpu::Backends::PRIMARY
    };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        ..Default::default()
    }))
    .err()
    .map(|e| format!("no GPU adapter: {backends:?}: {e}"))
}
