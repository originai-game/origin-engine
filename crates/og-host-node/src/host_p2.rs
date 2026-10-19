//! Phase 2's modules' trait implementations (TerrainApi, AudioApi): every member is
//! the host-neutral runtime's.

use crate::NodeHost;
use og_api::host::*;
use og_runtime as rt;

impl TerrainApi for NodeHost {
    fn og_terrain_load(&self, world: u32, path: String) -> String {
        rt::og_terrain_load(world, path)
    }
    fn og_terrain_sample(&self, world: u32, desc: Option<String>) -> String {
        rt::og_terrain_sample(world, desc)
    }
    fn og_terrain_bounds(&self, world: u32, data: f64, origin: Option<String>) -> String {
        rt::og_terrain_bounds(world, data, origin)
    }
    fn og_terrain_edit(&self, world: u32, data: f64, brush: Option<String>) -> String {
        rt::og_terrain_edit(world, data, brush)
    }
    fn og_terrain_layer(&self, world: u32, data: f64, index: u32) -> String {
        rt::og_terrain_layer(world, data, index)
    }
    fn og_terrain_set_layer(
        &self,
        world: u32,
        data: f64,
        index: u32,
        layer: Option<String>,
    ) -> String {
        rt::og_terrain_set_layer(world, data, index, layer)
    }
}

impl AudioApi for NodeHost {
    fn og_audio_output_latency(&self, world: u32) -> f64 {
        rt::og_audio_output_latency(world)
    }
}
