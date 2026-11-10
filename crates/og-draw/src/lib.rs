//! Origin Engine draw commands: what a frame asks og-render to draw over
//! the scene, as plain data. og-ui writes it and og-render draws it, so
//! the UI never links the renderer.
//!
//! - [`Overlay`]: screen-space sprites, text, glass panels and clip
//!   rectangles, and the UI pass's [`ui::UiList`].
//! - [`ui`]: the UI pass's shapes, paints, layers, filters, paths and mass
//!   elements.
//! - The resource handles ([`TextureHandle`], [`FontHandle`], ...) a scene
//!   gives what is added to it.
//! - [`ColorFilter`]: the colour vision filter over the presented frame.

mod color_vision;
mod handles;
mod sprites;
pub mod ui;

pub use color_vision::{ColorFilter, ColorVision, ColorVisionFilter};
pub use handles::*;
pub use sprites::*;
pub use ui::{
    UI_GRADIENT_STOPS, UI_MASS_KEYS, UI_MASS_QUADS, UI_MATERIAL_PARAMS, UI_QUAD_LAYER_MAX, UiAlign,
    UiBackground, UiBackgroundSize, UiBlend, UiBorderImage, UiBuiltin, UiCap, UiClip,
    UiColorMatrix, UiCorner, UiCssCorners, UiDrape, UiEdge, UiElevation, UiFillRule, UiFilter,
    UiFilterComposite, UiFilterInput, UiFilterOp, UiFilterPrimitive, UiFit, UiGeometry, UiGlass,
    UiGlassFallback, UiGlassShadow, UiGlassStyle, UiGlassWeight, UiGradient, UiGradientImage,
    UiGradientKind, UiGround, UiImage, UiJoin, UiLayer, UiLight, UiList, UiMask, UiMassBatch,
    UiMassInstance, UiMassKey, UiMassKind, UiMassSpace, UiMassTemplate, UiMaterial,
    UiMaterialHandle, UiMode, UiOccluder, UiOcclusion, UiPaint, UiParticleShape, UiPath, UiRepeat,
    UiShape, UiSpread, UiStop, UiStroke, UiTextEffects, UiTextOutline, UiTextShadow, UiTransfer,
    UiVectorData, UiVectorDraw, UiVectorHandle, UiVectorPaint, UiVectorStroke, UiVectorVertex,
    UiVerb, flatten_cubic, flatten_quad, format_number, oklab, oklab_to_linear, square_to_quad,
    ui_scrim,
};
