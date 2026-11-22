//! Origin Engine's modeling kernel (og-shape): solids built from programs
//! and the tools that finish meshes from elsewhere, deterministic in f64
//! on every host (`math` states the rules).

pub mod bevel;
pub mod boolean;
pub mod edit;
pub mod generators;
pub mod math;
pub mod mesh;
pub mod model;
pub mod profile;
pub mod skin;
pub mod tools;
