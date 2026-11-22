//! og-shape's integration tests, by the part of the kernel each exercises.

mod bevel {
    mod morphs;
    mod uvs;
}
mod boolean {
    mod contracts;
    mod edge_on_edge;
    mod self_union;
    mod slivers;
    mod touching;
    mod ulps_apart;
    mod uncross;
}
mod edit {
    mod holes;
    mod morphs;
}
mod generators {
    mod sweep_ends;
    mod textured;
}
mod model {
    mod built_parts;
    mod end_to_end;
    mod params;
    mod phase6_b;
    mod pose;
    mod rest;
    mod surface;
    mod uvs;
}
mod skin {
    mod phase5b;
}
mod tools {
    mod colors;
    mod export;
    mod meshtools;
    mod sand3d_gallery;
    mod uvs;
}
