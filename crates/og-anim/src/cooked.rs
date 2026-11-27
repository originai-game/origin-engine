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

    /// The validated graph source and its model clip bindings.
    fn animgraph(&self, id: &str, bytes: &[u8]) -> Result<crate::graph::GraphSource, LoadError>;
}

impl BundleAnim for Bundle {
    fn animgraph(&self, id: &str, bytes: &[u8]) -> Result<crate::graph::GraphSource, LoadError> {
        let blob = self.blob::<format::AnimGraph>(id, bytes)?;
        let meta = one::<format::AnimGraphMeta>(&blob)?;
        let data = blob.raw("data")?;
        if data.len() != meta.data_bytes as usize {
            return Err(invalid(&blob, "animation graph data length mismatch"));
        }
        crate::graph::GraphSource::from_bytes(data).map_err(|e| invalid(&blob, e.reason()))
    }

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
        let data = blob.raw("sheet")?;
        if data.len() != meta.data_bytes as usize {
            return Err(invalid(&blob, "the native sheet length does not match"));
        }
        crate::FlipbookSheet::decode(data).map_err(|e| invalid(&blob, e.reason()))
    }

    fn rig2d(&self, id: &str, bytes: &[u8]) -> Result<crate::Rig2dAsset, LoadError> {
        let blob = self.blob::<format::Rig2D>(id, bytes)?;
        let meta = one::<format::Rig2DMeta>(&blob)?;
        let data = blob.raw("data")?;
        if data.len() != meta.data_bytes as usize {
            return Err(invalid(&blob, "the native rig length does not match"));
        }
        crate::Rig2dAsset::from_bytes(data).map_err(|e| invalid(&blob, e.reason()))
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
        let strings = blob.raw("strings")?;
        let text = |offset: u32, len: u32| {
            (offset as usize)
                .checked_add(len as usize)
                .and_then(|end| strings.get(offset as usize..end))
                .and_then(|s| std::str::from_utf8(s).ok())
                .ok_or_else(|| invalid(&blob, "a rig string range is invalid or not UTF-8"))
        };
        let scheme = text(meta.scheme_offset, meta.scheme_len)?;
        let mut rig = crate::RigAsset {
            scheme: (!scheme.is_empty()).then(|| scheme.to_owned()),
            foot_height: match meta.foot_height {
                0 => None,
                1 => Some([meta.left_height, meta.right_height]),
                _ => return Err(invalid(&blob, "invalid rig foot-height flag")),
            },
            bones: Vec::new(),
        };
        for row in blob
            .table::<format::RigBoneRow>("bones", format::row_hash::<format::RigBoneRow>())?
            .iter()
        {
            let node = match row.mapped {
                0 => None,
                1 => Some(text(row.node_offset, row.node_len)?.to_owned()),
                _ => return Err(invalid(&blob, "invalid rig bone mapping flag")),
            };
            rig.bones.push((row.bone as usize, node, 0));
        }
        rig.validate().map_err(|e| invalid(&blob, e.reason()))?;
        Ok(rig)
    }
}
