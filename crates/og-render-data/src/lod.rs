/// The most a drawn level's error may cover on the screen, in pixels of
/// the render target's height, at bias 1.
pub const LOD_PIXEL_ERROR: f32 = 1.0;

/// How far past its threshold (a fraction of it) the projected error must
/// go before the level changes.
pub const LOD_HYSTERESIS: f32 = 0.15;

/// The most levels a chain keeps (the cook writes at most eight).
pub const MAX_LOD_LEVELS: usize = 8;
