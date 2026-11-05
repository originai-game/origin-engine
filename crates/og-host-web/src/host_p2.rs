//! Phase 2's modules' trait implementations (TerrainApi, VideoApi, StreamApi, AudioApi, PathApi, TimelineApi, MotionApi): every member is
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
    fn og_terrain_stamp(&self, world: u32, desc: Option<String>) -> String {
        rt::og_terrain_stamp(world, desc)
    }
    fn og_terrain_unstamp(&self, world: u32, data: f64, stamp: u32) -> String {
        rt::og_terrain_unstamp(world, data, stamp)
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

impl VideoApi for WebHost {
    fn og_video_capabilities(&self, world: u32) -> String {
        rt::og_video_capabilities(world)
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

impl AudioApi for WebHost {
    fn og_audio_output_latency(&self, world: u32) -> f64 {
        rt::og_audio_output_latency(world)
    }
    fn og_audio_create_sound(&self, world: u32, desc: Option<String>) -> String {
        rt::og_audio_create_sound(world, desc)
    }
}

impl PathApi for WebHost {
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

impl TimelineApi for WebHost {
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

impl MotionApi for WebHost {
    fn og_motion_pose(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_motion_pose(world, entity, desc)
    }
    fn og_motion_place(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_motion_place(world, entity, desc)
    }
    fn og_motion_face(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_motion_face(world, entity, desc)
    }
    fn og_motion_walk_to(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_motion_walk_to(world, entity, desc)
    }
    fn og_motion_clear(&self, world: u32, entity: f64) -> String {
        rt::og_motion_clear(world, entity)
    }
    fn og_motion_look_at(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_motion_look_at(world, entity, desc)
    }
    fn og_motion_react(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_motion_react(world, entity, desc)
    }
    fn og_motion_status(&self, world: u32, entity: f64) -> String {
        rt::og_motion_status(world, entity)
    }
    fn og_motion_capabilities(&self, world: u32) -> String {
        rt::og_motion_capabilities(world)
    }
    fn og_motion_presentation(&self, world: u32, entity: f64) -> String {
        rt::og_motion_presentation(world, entity)
    }
    fn og_motion_poses(&self) -> String {
        rt::og_motion_poses()
    }
}

impl DestructionApi for WebHost {
    fn og_destruction_damage(&self, world: u32, entity: f64, desc: Option<String>) -> String {
        rt::og_destruction_damage(world, entity, desc)
    }
    fn og_destruction_explode(&self, world: u32, desc: Option<String>) -> String {
        rt::og_destruction_explode(world, desc)
    }
    fn og_destruction_fracture(&self, world: u32, entity: f64) -> String {
        rt::og_destruction_fracture(world, entity)
    }
    fn og_destruction_info(&self, world: u32, entity: f64) -> String {
        rt::og_destruction_info(world, entity)
    }
    fn og_destruction_material(&self, world: u32, kind: u32) -> String {
        rt::og_destruction_material(world, kind)
    }
    fn og_destruction_capabilities(&self, world: u32) -> String {
        rt::og_destruction_capabilities(world)
    }
}
