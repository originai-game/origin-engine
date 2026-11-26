//! One binding per domain: the IDL module of each domain crate over the
//! runtime's engines.
//! Domain crates do not depend on each other; where two meet (AI hearing
//! audio, cloth posed by a skin, the sea's floor from a terrain) the wiring
//! is here.

pub(crate) mod agents;
pub(crate) mod ai;
pub(crate) mod anim;
pub(crate) mod destruction;
pub(crate) mod destruction2d;
pub(crate) mod flow;
pub(crate) mod forces;
pub(crate) mod garment;
pub(crate) mod i18n;
pub(crate) mod modules;
pub(crate) mod motion;
pub(crate) mod nav;
pub(crate) mod net;
pub(crate) mod particles;
pub(crate) mod physics_ext;
pub(crate) mod play;
pub(crate) use og_generation as procgen;
pub(crate) mod seabed;
pub(crate) mod season;
pub(crate) mod shared;
pub(crate) mod sight;
pub(crate) mod skeleton2d;
pub(crate) mod story;
pub(crate) mod table;
pub(crate) mod terrain;
pub(crate) mod terrain_stream;
pub(crate) mod timeline;
pub(crate) mod vfx;
pub(crate) mod video;
pub(crate) mod weather;
pub(crate) mod wind;

/// What the hosts call of the bindings (`og_runtime::*`).
pub mod exports {
    pub use super::ai::{
        og_ai_describe, og_ai_finish, og_ai_get, og_ai_load, og_ai_noise, og_ai_parse,
        og_ai_restart, og_ai_set,
    };
    pub use super::anim::{
        connect_animation_physics, og_anim_get_node_matrix, og_anim_graph, og_anim_humanoid,
        og_anim_mirror, og_anim_retarget, og_load_animation, og_load_animgraph, og_load_rig,
    };
    pub use super::destruction::{
        og_destruction_capabilities, og_destruction_damage, og_destruction_explode,
        og_destruction_fracture, og_destruction_info, og_destruction_material,
    };
    pub use super::destruction2d::{
        og_destruction_carve2d, og_destruction_cell2d, og_destruction_explode2d,
        og_destruction_fill2d, og_destruction_fracture2d,
    };
    pub use super::flow::{
        og_flow_advance, og_flow_chart, og_flow_checkpoint, og_flow_choose, og_flow_current,
        og_flow_gesture, og_flow_history, og_flow_inspect, og_flow_jump, og_flow_load,
        og_flow_pause, og_flow_play, og_flow_seek, og_flow_set_music, og_flow_set_speed,
        og_flow_skip, og_flow_start, og_flow_stop, og_flow_view,
    };
    pub use super::i18n::{
        fonts_pending, og_i18n_format_compact, og_i18n_format_currency, og_i18n_format_date,
        og_i18n_format_duration, og_i18n_format_number, og_i18n_format_time, og_i18n_has,
        og_i18n_load, og_i18n_locale, og_i18n_set_locale, og_i18n_t,
    };
    pub use super::modules::{
        og_load_style, og_physics_mass_properties, og_physics_overlap, og_physics_overlap_point,
        og_physics_project_point, og_physics_raycast, og_physics_raycast_all, og_physics_set_mesh,
        og_physics_shape_cast, og_physics_shape_cast_all, og_physics_velocity_at_point,
        og_ui_configure, og_ui_feedback, og_ui_frame_cursor, og_ui_frame_direction,
        og_ui_frame_pointer, og_ui_frame_theme, og_ui_frame_viewport, og_ui_has_icon,
        og_ui_preview_viewport, og_ui_seconds, og_ui_set_haptics, og_ui_set_virtual_cursor,
        og_ui_theme, og_ui_tick, og_ui_viewport, og_ui_virtual_cursor, set_viewport, ui_surface,
    };
    pub use super::motion::{
        og_motion_capabilities, og_motion_clear, og_motion_face, og_motion_jump, og_motion_look_at,
        og_motion_place, og_motion_pose, og_motion_poses, og_motion_presentation, og_motion_react,
        og_motion_set_backend, og_motion_status, og_motion_walk_to,
    };
    pub use super::nav::{
        og_nav_build, og_nav_carve, og_nav_cell_at, og_nav_find_path, og_nav_finish_link,
        og_nav_flow_at, og_nav_flow_field, og_nav_grid, og_nav_grid_from_tilemap, og_nav_load,
        og_nav_nearest, og_nav_path_of, og_nav_set_area, og_nav_set_cost, og_nav_set_flow_targets,
        og_nav_set_link, og_nav_set_link_open, og_nav_set_obstacle, world_handle_names,
    };
    pub use super::net::{
        og_net_command, og_net_commands, og_net_final_hash, og_net_hash, og_net_resimulating,
        og_net_shape, og_net_status, og_net_submit, og_net_submit_input, og_platform_quick_play,
        og_platform_room_create, og_platform_room_info, og_platform_room_join,
        og_platform_room_leave, og_platform_room_message, og_platform_room_peers,
        og_platform_room_send, og_room_deliver, og_room_set_host, og_room_take, world_block_forms,
        world_snapshot_bytes,
    };
    pub use super::physics_ext::{
        og_physics_apply_force, og_physics_apply_impulse, og_physics_apply_ragdoll_impulse,
        og_physics_apply_torque, og_physics_apply_torque_impulse, og_physics_cloth_points,
        og_physics_fluid_particles, og_physics_fluid3d_particles, og_physics_garment_points,
        og_physics_ragdoll_pose, og_physics_ragdoll_rig_from_skeleton, og_physics_resize_character,
        og_physics_rope_points, og_physics_set_ragdoll_rig, og_physics_soft_body_points,
        og_physics_stats, og_physics_vehicle_wheels, og_physics_water, og_physics_water_height,
    };
    pub use super::play::og_play_candidates;
    pub use super::procgen::{
        og_procgen_bsp, og_procgen_curl_noise, og_procgen_fracture, og_procgen_heightmap,
        og_procgen_noise, og_procgen_noise_grad, og_procgen_noise_grid, og_procgen_plant,
        og_procgen_poisson_disk, og_procgen_wfc, og_procgen_worley,
    };
    pub use super::season::og_atmosphere_season;
    pub use super::shared::{
        og_shared_deliver, og_shared_events, og_shared_join, og_shared_leave, og_shared_object,
        og_shared_put, og_shared_remote, og_shared_remove, og_shared_send, og_shared_shape,
        og_shared_status, og_shared_take, og_shared_teleport,
    };
    pub use super::sight::{og_sight_explored, og_sight_forget, og_sight_reveal, og_sight_visible};
    pub use super::skeleton2d::{og_anim_rig2d_info, og_anim_skeleton2d_boxes, og_load_rig2d};
    pub use super::story::{
        og_story_advance, og_story_ambient, og_story_bark, og_story_barks, og_story_choose,
        og_story_complete_quest, og_story_conversations, og_story_fail_quest, og_story_get,
        og_story_graph, og_story_history, og_story_load, og_story_mark, og_story_mark_seen,
        og_story_parse, og_story_progress, og_story_quest, og_story_quests, og_story_reset,
        og_story_set, og_story_set_saliency, og_story_set_stage, og_story_start,
        og_story_start_quest, og_story_stop, og_story_track, og_story_view,
    };
    pub use super::table::{
        og_table_get, og_table_load, og_table_missing, og_table_name, og_table_parse, og_table_row,
        og_table_rows,
    };
    pub use super::terrain::{
        og_terrain_bounds, og_terrain_create, og_terrain_edit, og_terrain_layer, og_terrain_load,
        og_terrain_sample, og_terrain_set_layer, og_terrain_stamp, og_terrain_unstamp,
    };
    pub use super::timeline::{
        og_load_timeline, og_path_create, og_path_find, og_path_length, og_path_nearest,
        og_path_sample, og_timeline_captions, og_timeline_info, og_timeline_pause,
        og_timeline_play, og_timeline_seek, og_timeline_stop,
    };
    pub use super::vfx::{og_load_vfx, og_vfx_info, og_vfx_spawn};
    pub use super::video::{
        og_load_video, og_load_video_info, og_video_buffering, og_video_capabilities,
        og_video_set_quality,
    };
}
