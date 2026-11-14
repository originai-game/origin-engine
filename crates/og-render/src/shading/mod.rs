//! How a surface is lit: materials, lights and clusters, shadows, indirect light, reflections, transparency.

pub(crate) mod area;
pub(crate) mod clusters;
pub(crate) mod decals;
pub(crate) mod environment;
pub(crate) mod gi_probes;
pub(crate) mod gi_scene;
pub(crate) mod ibl;
pub(crate) mod lightmap;
pub(crate) mod lit_prelude;
pub(crate) mod material;
pub(crate) mod oit;
pub(crate) mod physical;
pub(crate) mod probe_content;
pub(crate) mod probe_grid;
pub(crate) mod probe_select;
pub(crate) mod probes;
pub(crate) mod shadow_cache;
pub(crate) mod shadow_fit;
pub(crate) mod shadow_keep;
pub(crate) mod shadow_target;
pub(crate) mod shadows;
pub(crate) mod ssgi;
pub(crate) mod ssr;
