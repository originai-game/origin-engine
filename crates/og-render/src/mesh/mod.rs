//! Mesh forms and vertex-stage deformation: the compact encoding, skinning, lines, points, glTF import.

pub(crate) mod compact;
#[cfg(feature = "gltf")]
pub mod gltf;
pub(crate) mod lines;
pub(crate) mod points;
pub(crate) mod skin;
