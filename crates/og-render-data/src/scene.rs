//! The scene's resources as data: geometry, textures, materials and the
//! handles a scene gives them.

mod resources;
mod sprites;
pub use resources::*;
pub use sprites::*;

use crate::texture::TextureUpload;
use glam::Vec3;
pub use og_draw::{
    EnvironmentMapHandle, FontHandle, LutHandle, MaterialHandle, MeshHandle, PaletteHandle,
    ShaderHandle, SpriteArray, TerrainHandle, TextureArrayHandle, TextureHandle,
};
