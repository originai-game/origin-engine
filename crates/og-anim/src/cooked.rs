//! The cooked animation assets: what og-cook writes for a glTF's `#anim`
//! blob, a flipbook sheet, a 2D rig and a humanoid rig, decoded into this
//! crate's assets. The blob schemas are og-bundle's (`og_bundle::format`);
//! the loaders live here because they build og-anim's types.
//!
//! | item | what it is |
//! |---|---|
//! | [`BundleAnim`] | the loaders, as methods of `og_bundle::runtime::Bundle` |
//! | [`CookedAnimation`], [`SkinStream`], [`MorphTargets`] | a decoded animation blob and its per-vertex skin and morph data |
//! | [`anim_codec`] | how animation channels hold their samples: f32, constant, quantised keys with keyframe reduction |

use og_bundle::format;
#[cfg(not(target_arch = "wasm32"))]
use og_bundle::runtime::anim_id;
use og_bundle::runtime::{Bundle, LoadError, asset_id, invalid, one};

pub mod anim_codec;
mod animation;

pub use animation::*;

/// The animation loaders of a cooked bundle: bring it into scope to call
/// them on a [`Bundle`].
pub trait BundleAnim {
    /// Decodes an animation blob (`<model>#anim`): the skeleton, the clips
    /// and the per-vertex skin and morph data, as og-anim's assets.
    fn animation(&self, id: &str, bytes: &[u8]) -> Result<CookedAnimation, LoadError>;

    /// Reads and decodes a glTF's animation blob (native); `id` is the model's
    /// id with or without `#anim`.
    #[cfg(not(target_arch = "wasm32"))]
    fn load_animation(&self, id: &str) -> Result<CookedAnimation, LoadError>;

    /// Decodes a flipbook sheet blob (`og.Flipbook`) into its sheet.
    fn flipbook(&self, id: &str, bytes: &[u8]) -> Result<crate::FlipbookSheet, LoadError>;

    /// Decodes a 2D rig blob (`og.Rig2D`) into its rig.
    fn rig2d(&self, id: &str, bytes: &[u8]) -> Result<crate::Rig2dAsset, LoadError>;

    /// Reads and decodes a 2D rig (native).
    #[cfg(not(target_arch = "wasm32"))]
    fn load_rig2d(&self, id: &str) -> Result<crate::Rig2dAsset, LoadError>;

    /// Reads and decodes a flipbook sheet (native).
    #[cfg(not(target_arch = "wasm32"))]
    fn load_flipbook(&self, id: &str) -> Result<crate::FlipbookSheet, LoadError>;

    /// The humanoid map of a rig blob (`og.Rig`).
    fn rig(&self, id: &str, bytes: &[u8]) -> Result<crate::RigAsset, LoadError>;
}

impl BundleAnim for Bundle {
    fn animation(&self, id: &str, bytes: &[u8]) -> Result<CookedAnimation, LoadError> {
        let blob = self.blob::<format::Animation>(id, bytes)?;
        CookedAnimation::decode(asset_id(id), &blob)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_animation(&self, id: &str) -> Result<CookedAnimation, LoadError> {
        let id = anim_id(id);
        self.animation(&id, &self.read(&id)?)
    }

    fn flipbook(&self, id: &str, bytes: &[u8]) -> Result<crate::FlipbookSheet, LoadError> {
        let blob = self.blob::<format::Flipbook>(id, bytes)?;
        let meta = one::<format::FlipbookMeta>(&blob)?;
        let text = blob.raw("text")?;
        let text = text
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the sheet text is cut short or not UTF-8"))?;
        crate::FlipbookSheet::parse(text).map_err(|e| invalid(&blob, e.reason()))
    }

    fn rig2d(&self, id: &str, bytes: &[u8]) -> Result<crate::Rig2dAsset, LoadError> {
        let blob = self.blob::<format::Rig2D>(id, bytes)?;
        let meta = one::<format::Rig2DMeta>(&blob)?;
        let text = blob.raw("text")?;
        let text = text
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the rig text is cut short or not UTF-8"))?;
        crate::Rig2dAsset::parse(text).map_err(|e| invalid(&blob, e.reason()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_rig2d(&self, id: &str) -> Result<crate::Rig2dAsset, LoadError> {
        self.rig2d(id, &self.read(id)?)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn load_flipbook(&self, id: &str) -> Result<crate::FlipbookSheet, LoadError> {
        self.flipbook(id, &self.read(id)?)
    }

    fn rig(&self, id: &str, bytes: &[u8]) -> Result<crate::RigAsset, LoadError> {
        let blob = self.blob::<format::Rig>(id, bytes)?;
        let meta = one::<format::RigMeta>(&blob)?;
        let text = blob
            .raw("text")?
            .get(..meta.text_bytes as usize)
            .and_then(|t| std::str::from_utf8(t).ok())
            .ok_or_else(|| invalid(&blob, "the rig text is cut short or not UTF-8"))?;
        crate::RigAsset::parse(text).map_err(|e| invalid(&blob, e.reason().to_string()))
    }
}
