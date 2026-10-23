//! Phase 2's modules' trait implementations (TerrainApi, StreamApi, AudioApi, PathApi, TimelineApi): every member is
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

impl StreamApi for NodeHost {
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

impl AudioApi for NodeHost {
    fn og_audio_output_latency(&self, world: u32) -> f64 {
        rt::og_audio_output_latency(world)
    }
}

impl PathApi for NodeHost {
    fn og_path_create(&self, world: u32, desc: Option<String>) -> String {
        rt::og_path_create(world, desc)
    }
    fn og_path_find(&self, world: u32, name: String) -> String {
        rt::og_path_find(world, name)
    }
    fn og_path_sample(&self, world: u32, path: f64, t: f64) -> String {
        rt::og_path_sample(world, path, t)
    }
    fn og_path_length(&self, world: u32, path: f64) -> String {
        rt::og_path_length(world, path)
    }
    fn og_path_nearest(&self, world: u32, path: f64, point: String) -> String {
        rt::og_path_nearest(world, path, point)
    }
}

impl TimelineApi for NodeHost {
    fn og_timeline_info(&self, world: u32, asset: f64) -> String {
        rt::og_timeline_info(world, asset)
    }
    fn og_timeline_play(&self, world: u32, entity: f64) -> String {
        rt::og_timeline_play(world, entity)
    }
    fn og_timeline_pause(&self, world: u32, entity: f64) -> String {
        rt::og_timeline_pause(world, entity)
    }
    fn og_timeline_seek(&self, world: u32, entity: f64, time: f64) -> String {
        rt::og_timeline_seek(world, entity, time)
    }
    fn og_timeline_stop(&self, world: u32, entity: f64) -> String {
        rt::og_timeline_stop(world, entity)
    }
}
