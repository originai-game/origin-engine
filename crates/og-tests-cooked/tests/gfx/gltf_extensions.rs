//! The glTF extension scenes (roadmap G1-3, `og_tests_cooked::gltf_scenes`):
//! each cooks, carries what its extension says into the cooked model and
//! the scene, and matches its reference in og-tests' references/. The
//! references were not recorded with the scenes (they were written on an
//! orb with no GPU): until macrunner records them (`OG_GFX_CAPTURE=<scene>`)
//! each test fails at its reference, after its cooked checks pass.
//!
//! Before G1-3 the cook dropped every one of these extensions (and refused
//! `EXT_mesh_gpu_instancing`, which the instancing file requires): no
//! transform, no lights, one box instead of seven, no variants.
