//! Origin Engine draw commands: what a frame asks og-render to draw over
//! the scene, as plain data. og-ui writes it and og-render draws it, so
//! the UI never links the renderer.
//!
//! - [`Overlay`]: screen-space sprites, text, glass panels and clip
//!   rectangles, and the UI pass's [`ui::UiList`].
//! - [`paint::Paint`]: og-ui's paint arena, drawn by the paint pass.
//! - [`ui`]: the UI pass's shapes, paints, layers, filters, paths and mass
//!   elements.
//! - The resource handles ([`TextureHandle`], [`FontHandle`], ...) a scene
//!   gives what is added to it.
//! - [`ColorFilter`]: the colour vision filter over the presented frame.
//! - [`RangeLimit`]: how far over SDR white the presented frame may go.

mod color_vision;
mod handles;
pub mod paint;
mod range_limit;
mod sprites;
pub mod ui;

pub use color_vision::{ColorFilter, ColorVision, ColorVisionFilter};
pub use handles::*;
/// The path model ([`UiPath`]'s `BezPath`) and its geometry.
pub use kurbo;
pub use range_limit::RangeLimit;
pub use sprites::*;
pub use ui::{
    UI_GRADIENT_STOPS, UI_MASS_KEYS, UI_MASS_QUADS, UI_MATERIAL_PARAMS, UI_QUAD_LAYER_MAX, UiAlign,
    UiBackground, UiBackgroundSize, UiBlend, UiBorderImage, UiBuiltin, UiCap, UiClip,
    UiColorMatrix, UiCorner, UiCssCorners, UiDrape, UiEdge, UiElevation, UiFillRule, UiFilter,
    UiFilterComposite, UiFilterInput, UiFilterOp, UiFilterPrimitive, UiFit, UiGeometry, UiGlass,
    UiGlassFallback, UiGlassShadow, UiGlassStyle, UiGlassWeight, UiGradient, UiGradientImage,
    UiGradientKind, UiGround, UiImage, UiJoin, UiLayer, UiLight, UiList, UiMask, UiMassBarStyle,
    UiMassBatch, UiMassInstance, UiMassKey, UiMassKind, UiMassSpace, UiMassTemplate, UiMaterial,
    UiMaterialHandle, UiMode, UiOccluder, UiOcclusion, UiOrientation, UiPaint, UiParticleShape,
    UiPath, UiRepeat, UiShape, UiSpread, UiStop, UiStroke, UiTextEffects, UiTextOutline,
    UiTextShadow, UiTransfer, UiVectorData, UiVectorDraw, UiVectorHandle, UiVectorPaint,
    UiVectorStroke, UiVectorVertex, flatten_cubic, format_number, glam_vec2, kurbo_point, oklab,
    oklab_to_linear, parse_svg_path, polylines, square_to_quad, ui_scrim,
};
