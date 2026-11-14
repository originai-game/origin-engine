//! Mesh forms and vertex-stage deformation: the compact encoding, skinning, foliage in the wind, lines, points, glTF import.

pub(crate) mod compact;
pub mod foliage;
#[cfg(feature = "gltf")]
pub mod gltf;
pub(crate) mod lines;
pub(crate) mod points;
pub(crate) mod skin;
