//! The sky pass's module.

use crate::sky_physical::sky_scatter_wgsl;
use og_render_data::shader_source::FRAME_WGSL;

/// The clouds' shape, then the sky.
pub(crate) const SKY_WGSL: &str = concat!(wgsl!("clouds_shape.wgsl"), wgsl!("sky.wgsl"));
/// The sky pass's module: the frame, the physical sky, the sky pass.
pub(crate) fn sky_source() -> String {
    format!(
        "{FRAME_WGSL}{}{SKY_WGSL}",
        sky_scatter_wgsl("frame.sky_scatter")
    )
}
