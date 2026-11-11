//! RenderScene's frame data by group: plain data, no GPU types, so a
//! format crate can take each group whole.

use super::*;

/// The frame's look: its post effects and stylise, the custom post passes
/// and their parameters, and the colour filter.
#[derive(Debug)]
pub struct SceneLook {
    pub post: PostEffects,
    /// Stylisation after (and, for depth bands, with) the post stack: off
    /// by default ([`crate::Stylise`]).
    pub stylise: crate::Stylise,
    /// Custom post passes ([`RenderScene::add_post`], `post_custom.rs`) and
    /// their parameters, by handle.
    pub posts: Vec<crate::PostShader>,
    pub post_params: Vec<[[f32; 4]; crate::MAX_SHADER_PARAMS]>,
    /// The custom post passes the frame runs, in order (at most
    /// [`crate::MAX_CUSTOM_POSTS`]), after the built-in effects and before
    /// exposure, bloom and the tone curve (`PostEffects.custom` in the API).
    pub custom_post: Vec<crate::PostHandle>,
    /// The player's colour vision filter over the finished frame, scene and
    /// overlay alike (`None`: none; crate::ColorFilter).
    pub color_filter: Option<crate::ColorFilter>,
}
