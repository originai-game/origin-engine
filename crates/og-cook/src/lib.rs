//! Origin Engine `og-cook`: the cook (`assets/` → `cooked/`) and the runtime
//! loaders that read what it writes (`docs/design.md` §2.5, `README.md`).
//!
//! | module | feature | what it owns |
//! |---|---|---|
//! | [`anim_codec`] | `runtime` | how animation channels hold their samples: f32, constant, quantised keys with keyframe reduction |
//! | [`format`] | always | the asset blobs: reflected root and row structs, type hashes, section names |
//! | [`runtime`] | `runtime` | [`runtime::Bundle`] (the manifest as an index), loaders into og-render types and [`runtime::SceneDesc`] |
//! | [`ktx2`] | `runtime` | KTX2 parsing, lossless RGBA8 textures; `.basis` ↔ KTX2 with `basis`/`transcode` |
//! | [`meshopt_codec`] | `runtime` | pure-Rust meshoptimizer vertex and index decoders |
//! | [`qoa`] | `runtime` | the QOA audio codec |
//! | [`fx`] | `runtime` | particle emitter files (`.ogt` of `type particles`) and `.cube` colour tables |
//! | [`look`] | `runtime` (resolution and validation: `cook`) | looks (`.ogt` of `type look`): the text, merging over a base, hex colours, `.hex`/`.gpl` palettes |
//! | [`shader`] | `runtime` (validation: `cook`) | material shaders (`.ogt` of `type shader`): parsing, the cooked text, naga compilation for every back end |
//! | `cook` (skins) | `cook` | UI skins (`.ogt` of `type skin`), validated with og-ui's parser |
//! | [`cook`] | `cook` | [`cook::cook_project`], options, the report, the importers |
//!
//! `runtime` alone builds for `wasm32-unknown-unknown` (no C, no cook
//! dependencies). `transcode` adds native Basis Universal transcoding to
//! RGBA8; `cook` (the default) adds the importers and encoders.

#[cfg(feature = "runtime")]
pub mod anim_codec;
pub mod format;
#[cfg(feature = "runtime")]
pub mod fx;
#[cfg(feature = "runtime")]
pub mod ktx2;
#[cfg(feature = "runtime")]
pub mod look;
#[cfg(feature = "runtime")]
pub mod meshopt_codec;
#[cfg(feature = "runtime")]
pub mod qoa;
#[cfg(feature = "runtime")]
pub mod runtime;
#[cfg(feature = "runtime")]
pub mod shader;

#[cfg(feature = "cook")]
pub mod cook;

#[cfg(feature = "cook")]
pub use cook::{COOK_TIMES, CookError, CookOptions, CookTimes, Report, cook_project};
