//! Compiles the vendored xatlas (third_party/xatlas) with its C API,
//! single-threaded so the charts and their packing do not depend on the
//! machine (third_party/xatlas/README.md).

fn main() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../third_party/xatlas");
    println!(
        "cargo:rerun-if-changed={}",
        dir.join("xatlas.cpp").display()
    );
    println!("cargo:rerun-if-changed={}", dir.join("xatlas.h").display());
    println!(
        "cargo:rerun-if-changed={}",
        dir.join("xatlas_c.h").display()
    );
    cc::Build::new()
        .cpp(true)
        .std("c++14")
        .file(dir.join("xatlas.cpp"))
        .include(&dir)
        .define("XATLAS_C_API", "1")
        .define("XA_MULTITHREADED", "0")
        // The library's own debug checks and prints stay off in every profile.
        .define("NDEBUG", None)
        .warnings(false)
        .flag_if_supported("-fno-exceptions")
        .compile("xatlas");
}
