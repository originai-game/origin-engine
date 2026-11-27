//! Native effect material resource bindings.

use og_render_data::{ShaderHandle, ShaderParam, TextureHandle};

/// A linked material shader and the parameters it declares.
pub type ShaderLink = (ShaderHandle, Vec<ShaderParam>);

/// Resolve native material resources to host handles. Capturing checked input
/// uses these same callbacks to construct the cooked resource table.
pub struct EffectLinks<'a> {
    pub texture: &'a mut dyn FnMut(&str) -> Result<Option<TextureHandle>, String>,
    pub shader: &'a mut dyn FnMut(&str) -> Result<Option<ShaderLink>, String>,
}
