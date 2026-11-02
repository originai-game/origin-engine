//! Origin Engine `og-bundle`: the cooked bundle's blob formats and the
//! runtime loaders that read what og-cook writes (`docs/design.md` §2.5,
//! `README.md`).
//!
//! | module | what it owns |
//! |---|---|
//! | [`anim_codec`] | how animation channels hold their samples: f32, constant, quantised keys with keyframe reduction |
//! | [`format`] | the asset blobs: reflected root and row structs, type hashes, section names |
//! | [`runtime`] | [`runtime::Bundle`] (the manifest as an index), loaders into og-render types and [`runtime::SceneDesc`] |
//! | [`ktx2`] | KTX2 parsing, lossless RGBA8 textures; `.basis` ↔ KTX2 with `basis`/`transcode` |
//! | [`meshopt_codec`] | pure-Rust meshoptimizer vertex and index decoders |
//! | [`qoa`] | the QOA audio codec |
//! | [`opus`] | Opus sounds: Opus packets in self-contained blocks (music and long sounds) |
//! | [`lightmap`] | a scene's baked lighting (`<scene>.lightmap.ogt` and its KTX2 pages) |
//! | [`fx`] | particle emitter files (`.ogt` of `type particles`) and `.cube` colour tables |
//! | [`look`] | looks (`.ogt` of `type look`): the text, merging over a base, hex colours, `.hex`/`.gpl` palettes |
//! | [`scene_text`] | scene write-back: a scene's `.ogt` rewritten line by line from current values, comments and order kept |
//! | [`shader`] | material shaders (`.ogt` of `type shader`): parsing and the cooked text (og-cook compiles them with naga) |
//!
//! Without features it builds for `wasm32-unknown-unknown` (no C).
//! `transcode` adds native Basis Universal transcoding to RGBA8; `basis`
//! the `.basis` → KTX2 wrapping og-cook's encoder uses.

pub mod anim_codec;
pub mod format;
pub mod fx;
pub mod ktx2;
pub mod lightmap;
pub mod look;
pub mod meshopt_codec;
pub mod opus;
pub mod qoa;
pub mod runtime;
pub mod scene_text;
pub mod shader;
