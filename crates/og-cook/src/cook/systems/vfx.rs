//! Composed effects share the native FX writer with materials and emitters.
use super::KindDef;

pub(crate) static VFX: KindDef = KindDef {
    name: "vfx",
    extensions: &["efkproj", "efkefc"],
    types: &[],
    by_content: false,
    streams: false,
    plan: super::effect::plan,
};
pub(crate) static KINDS: &[&KindDef] = &[&VFX];
