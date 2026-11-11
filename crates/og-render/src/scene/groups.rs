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

/// The overlay (its UI tree and canvases) and the UI's vector paths and
/// materials.
#[derive(Debug)]
pub struct SceneUi {
    /// Screen-space sprites and text over the finished frame (`None`: none).
    pub overlay: Option<Overlay>,
    /// Cooked vectors the overlay's UI draws ([`RenderScene::add_ui_vector`]).
    pub ui_vectors: Vec<crate::ui::UiVectorData>,
    /// Custom materials UI layers composite through
    /// ([`RenderScene::add_ui_material`]).
    pub ui_materials: Vec<crate::ui::UiMaterialShader>,
}

/// The air and the weather: fog, volumetric fog, clouds, aurora, the eye's
/// adaptation, the surface weather, the wind and what pushes the grass.
#[derive(Debug)]
pub struct SceneAtmosphere {
    /// Fog over the main pass (`None`: none).
    pub fog: Option<crate::Fog>,
    /// Fog lit as a participating medium (`volumetric.rs`; `None`: none).
    /// It replaces [`RenderScene::fog`] over the frame while it is set
    /// (and [`crate::RenderQuality::volumetric_fog`] lets it).
    pub volumetric_fog: Option<crate::VolumetricFog>,
    /// Clouds in the procedural sky (`clouds.rs`; `None`: none). Drawn
    /// only with [`crate::Background::Procedural`].
    pub clouds: Option<crate::Clouds>,
    /// An aurora in the procedural sky (`aurora.rs`; `None`: none). Drawn
    /// only with [`crate::Background::Procedural`].
    pub aurora: Option<crate::Aurora>,
    /// How much of the image-based light's strength
    /// ([`crate::Environment::intensity`]) is the eye's adaptation to shade
    /// rather than the sky's own light: the day-night cycle's twilight lift
    /// ([`crate::DayNightLook::adaptation`]). It lights surfaces, but a
    /// mirror of the sky (the sea's reflection) divides it out, so it shows
    /// the sky as bright as the sky is drawn. 1: none.
    pub adaptation: f32,
    /// Rain, puddles and snow on the lit surfaces (`None`: dry).
    pub surface_weather: Option<crate::SurfaceWeather>,
    /// The wind that sways the scattered copies (`None` or strength 0:
    /// still).
    pub wind: Option<crate::Wind>,
    /// Local pushes on the simulated wind field (og.WindMotor,
    /// `wind_field.rs`): the nearest [`crate::wind_field::MAX_MOTORS`] drive it.
    pub wind_motors: Vec<crate::WindMotor>,
    /// What presses the grass and scattered copies (og.GrassPusher,
    /// `trample.rs`): the nearest [`crate::trample::MAX_PUSHERS`] press.
    pub grass_pushers: Vec<crate::GrassPusher>,
}
