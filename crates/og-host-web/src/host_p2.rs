//! Phase 2's modules' trait implementations (TerrainApi, StreamApi): every member is
//! the host-neutral runtime's.

use crate::WebHost;
use og_api::host::*;
use og_runtime as rt;

impl TerrainApi for WebHost {
    fn og_terrain_load(&self, world: u32, path: String) -> String {
        rt::og_terrain_load(world, path)
    }
    fn og_terrain_sample(&self, world: u32, desc: Option<String>) -> String {
        rt::og_terrain_sample(world, desc)
    }
    fn og_terrain_bounds(
        &self,
        world: u32,
        data: f64,
        origin: Option<String>,
        quaternion: Option<String>,
        scale: Option<String>,
    ) -> String {
        rt::og_terrain_bounds(world, data, origin, quaternion, scale)
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

impl StreamApi for WebHost {
    fn og_stream_status(&self, world: u32, group: String) -> String {
        rt::og_stream_status(world, group)
    }
    fn og_stream_request(&self, world: u32, group: String) -> String {
        rt::og_stream_request(world, group)
    }
    fn og_stream_release(&self, world: u32, group: String) -> String {
        rt::og_stream_release(world, group)
    }
    fn og_stream_budget(&self, world: u32) -> String {
        rt::og_stream_budget(world)
    }
    fn og_stream_set_budget(&self, world: u32, bytes: f64) -> String {
        rt::og_stream_set_budget(world, bytes)
    }
}
