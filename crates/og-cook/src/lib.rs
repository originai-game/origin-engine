//! Origin Engine `og-cook`: the cook (`assets/` → `cooked/`, `docs/design.md`
//! §2.5, `README.md`). The runtime loaders that read what it writes are
//! og-bundle's, re-exported here under the same paths (`og_cook::runtime`,
//! `og_cook::format`, ...) for the cook and the tools.
//!
//! | module | feature | what it owns |
//! |---|---|---|
//! | `cook` (looks) | `cook` | look resolution and validation against the IDL's records |
//! | `cook` (shaders) | `cook` | naga compilation of material shaders, post passes and UI materials for every back end |
//! | `cook` (skins) | `cook` | UI skins (`.ogt` of `type skin`), validated with og-ui's parser |
//! | [`cook`] | `cook` | [`cook::cook_project`], options, the report, the importers |
//!
//! `transcode` forwards to og-bundle's native Basis Universal transcoding;
//! `basis` encodes textures with basis-universal; `cook` (the default) adds
//! the importers and encoders.

#![cfg_attr(windows, feature(windows_by_handle, windows_change_time))]

pub use og_bundle::{
    anim_codec, effect, format, fx, ktx2, lightmap, look, meshopt_codec, opus, qoa, runtime,
    scene_text, shader, vfx,
};

#[cfg(feature = "cook")]
pub mod cook;

#[cfg(feature = "cook")]
pub use cook::{COOK_TIMES, CookError, CookOptions, CookTimes, Report, cook_project};
