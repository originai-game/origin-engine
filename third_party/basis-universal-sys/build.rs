// args from the basis cmake file
fn build_with_common_settings() -> cc::Build {
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .flag_if_supported("-fvisibility=hidden")
        .flag_if_supported("-fno-strict-aliasing")
        // Vendored (a path dependency, whose warnings cargo prints on
        // every build): basisu's own warnings are upstream's to fix.
        .warnings(false)
        .cargo_warnings(false);

    // A multiply and add must round separately on every host. ARM64's
    // default contraction otherwise changes UASTC candidate/RDO choices
    // relative to x86-64, despite identical source/settings/cache keys.
    if build.get_compiler().is_like_msvc() {
        build.flag("/fp:precise");
    } else {
        build.flag("-ffp-contract=off");
    }

    build
}

fn main() {
    // This is a patched path dependency: native source/header edits must
    // invalidate the archive as well as changes to this build script.
    println!("cargo:rerun-if-changed=vendor");
    // x86-64: basisu's SSE 4.1 kernels (ETC1S selector and colour-distance
    // searches), chosen at run time by its CPU check; only their file is
    // compiled for SSE 4.1. They compute the scalar paths' integers, so the
    // bytes match every other host.
    let sse = std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("x86_64");
    let mut main = build_with_common_settings();
    if sse {
        main.define("BASISU_SUPPORT_SSE", "1");
    }
    main.define("BASISD_SUPPORT_KTX2_ZSTD", "0")
        .flag_if_supported("--std=c++11")
        .file("vendor/basis_universal/encoder/pvpngreader.cpp")
        .file("vendor/basis_universal/encoder/jpgd.cpp")
        .file("vendor/basis_universal/encoder/basisu_uastc_enc.cpp")
        .file("vendor/basis_universal/encoder/basisu_ssim.cpp")
        .file("vendor/basis_universal/encoder/basisu_resampler.cpp")
        .file("vendor/basis_universal/encoder/basisu_resample_filters.cpp")
        .file("vendor/basis_universal/encoder/basisu_pvrtc1_4.cpp")
        .file("vendor/basis_universal/encoder/basisu_opencl.cpp")
        .file("vendor/basis_universal/encoder/basisu_gpu_texture.cpp")
        .file("vendor/basis_universal/encoder/basisu_frontend.cpp")
        .file("vendor/basis_universal/encoder/basisu_etc.cpp")
        .file("vendor/basis_universal/encoder/basisu_enc.cpp")
        .file("vendor/basis_universal/encoder/basisu_comp.cpp")
        .file("vendor/basis_universal/encoder/basisu_bc7enc.cpp")
        .file("vendor/basis_universal/encoder/basisu_basis_file.cpp")
        .file("vendor/basis_universal/encoder/basisu_backend.cpp")
        .file("vendor/basis_universal/transcoder/basisu_transcoder.cpp")
        .file("vendor/transcoding_wrapper.cpp")
        .file("vendor/encoding_wrapper.cpp")
        .compile("basisuniversal");
    // After the library, which calls into it (static archives link in order).
    let mut kernels = build_with_common_settings();
    kernels
        .flag_if_supported("--std=c++11")
        .file("vendor/basis_universal/encoder/basisu_kernels_sse.cpp");
    if sse {
        kernels.define("BASISU_SUPPORT_SSE", "1");
        if !kernels.get_compiler().is_like_msvc() {
            kernels.flag("-msse4.1");
        }
    }
    kernels.compile("basisuniversal_kernels");

    // We regenerate binding code and check it in. (See generate_bindings.sh)
}
