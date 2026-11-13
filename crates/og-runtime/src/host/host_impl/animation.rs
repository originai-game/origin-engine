//! Animation's bindings (TweenApi, AnimApi, PathApi, TimelineApi, MotionApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_animation {
    ($host:ty) => {
        impl $crate::host_impl::api::TweenApi for $host {
            fn og_tween_to(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tween_to(world, desc))
            }
            fn og_tween_from(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tween_from(world, desc))
            }
            fn og_tween_from_to(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tween_from_to(world, desc))
            }
            fn og_tween_sequence(&self, world: u32, steps: String, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tween_sequence(world, steps, desc))
            }
            fn og_tween_parallel(
                &self,
                world: u32,
                tweens: String,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_tween_parallel(world, tweens, desc))
            }
            fn og_tween_cancel(&self, world: u32, tween: f64) -> bool {
                $crate::og_tween_cancel(world, tween)
            }
            fn og_tween_cancel_entity(&self, world: u32, entity: f64) -> u32 {
                $crate::og_tween_cancel_entity(world, entity)
            }
            fn og_tween_active(&self, world: u32, tween: f64) -> bool {
                $crate::og_tween_active(world, tween)
            }
            fn og_tween_squash(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tween_squash(world, desc))
            }
            fn og_tween_flash(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tween_flash(world, desc))
            }
            fn og_tween_timeline(&self, world: u32, desc: Option<String>) -> u32 {
                $crate::og_tween_timeline(world, desc)
            }
            fn og_tween_timeline_get_tween(&self, tween_timeline: u32) -> f64 {
                $crate::og_tween_timeline_get_tween(tween_timeline)
            }
            fn og_tween_timeline_get_error(&self, tween_timeline: u32) -> String {
                $crate::host_impl::json($crate::og_tween_timeline_get_error(tween_timeline))
            }
            fn og_tween_timeline_to(
                &self,
                tween_timeline: u32,
                targets: String,
                vars: Option<String>,
                position: Option<String>,
            ) -> u32 {
                $crate::og_tween_timeline_to(tween_timeline, targets, vars, position)
            }
            fn og_tween_timeline_from(
                &self,
                tween_timeline: u32,
                targets: String,
                vars: Option<String>,
                position: Option<String>,
            ) -> u32 {
                $crate::og_tween_timeline_from(tween_timeline, targets, vars, position)
            }
            fn og_tween_timeline_from_to(
                &self,
                tween_timeline: u32,
                targets: String,
                vars: Option<String>,
                position: Option<String>,
            ) -> u32 {
                $crate::og_tween_timeline_from_to(tween_timeline, targets, vars, position)
            }
            fn og_tween_timeline_set(
                &self,
                tween_timeline: u32,
                targets: String,
                vars: Option<String>,
                position: Option<String>,
            ) -> u32 {
                $crate::og_tween_timeline_set(tween_timeline, targets, vars, position)
            }
            fn og_tween_timeline_call(
                &self,
                tween_timeline: u32,
                name: String,
                position: Option<String>,
            ) -> u32 {
                $crate::og_tween_timeline_call(tween_timeline, name, position)
            }
            fn og_tween_timeline_add_label(
                &self,
                tween_timeline: u32,
                label: String,
                position: Option<String>,
            ) -> u32 {
                $crate::og_tween_timeline_add_label(tween_timeline, label, position)
            }
            fn og_tween_timeline_add(
                &self,
                tween_timeline: u32,
                child: String,
                position: Option<String>,
            ) -> u32 {
                $crate::og_tween_timeline_add(tween_timeline, child, position)
            }
            fn og_tween_timeline_play(&self, tween_timeline: u32, from: Option<String>) -> u32 {
                $crate::og_tween_timeline_play(tween_timeline, from)
            }
            fn og_tween_timeline_pause(&self, tween_timeline: u32, at: Option<String>) -> u32 {
                $crate::og_tween_timeline_pause(tween_timeline, at)
            }
            fn og_tween_timeline_resume(&self, tween_timeline: u32) -> u32 {
                $crate::og_tween_timeline_resume(tween_timeline)
            }
            fn og_tween_timeline_reverse(&self, tween_timeline: u32, from: Option<String>) -> u32 {
                $crate::og_tween_timeline_reverse(tween_timeline, from)
            }
            fn og_tween_timeline_restart(&self, tween_timeline: u32) -> u32 {
                $crate::og_tween_timeline_restart(tween_timeline)
            }
            fn og_tween_timeline_seek(&self, tween_timeline: u32, position: String) -> u32 {
                $crate::og_tween_timeline_seek(tween_timeline, position)
            }
            fn og_tween_timeline_progress(&self, tween_timeline: u32, value: Option<f64>) -> f64 {
                $crate::og_tween_timeline_progress(tween_timeline, value)
            }
            fn og_tween_timeline_time(&self, tween_timeline: u32) -> f64 {
                $crate::og_tween_timeline_time(tween_timeline)
            }
            fn og_tween_timeline_duration(&self, tween_timeline: u32) -> f64 {
                $crate::og_tween_timeline_duration(tween_timeline)
            }
            fn og_tween_timeline_time_scale(&self, tween_timeline: u32, value: Option<f64>) -> f64 {
                $crate::og_tween_timeline_time_scale(tween_timeline, value)
            }
            fn og_tween_timeline_is_active(&self, tween_timeline: u32) -> bool {
                $crate::og_tween_timeline_is_active(tween_timeline)
            }
            fn og_tween_timeline_kill(&self, tween_timeline: u32) -> bool {
                $crate::og_tween_timeline_kill(tween_timeline)
            }
        }

        impl $crate::host_impl::api::AnimApi for $host {
            fn og_anim_graph(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_anim_graph(world, desc))
            }

            fn og_anim_humanoid(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_anim_humanoid(world, desc))
            }

            fn og_anim_rig2d_info(&self, world: u32, rig: f64) -> String {
                $crate::host_impl::answer($crate::og_anim_rig2d_info(world, rig))
            }

            fn og_anim_skeleton2d_boxes(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_anim_skeleton2d_boxes(world, entity))
            }

            fn og_anim_retarget(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_anim_retarget(world, desc))
            }
            fn og_anim_mirror(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_anim_mirror(world, desc))
            }
        }

        impl $crate::host_impl::api::PathApi for $host {
            fn og_path_create(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_path_create(world, desc))
            }
            fn og_path_find(&self, world: u32, name: String) -> String {
                $crate::host_impl::answer($crate::og_path_find(world, name))
            }
            fn og_path_sample(&self, world: u32, path: f64, t: f64) -> String {
                $crate::host_impl::answer($crate::og_path_sample(world, path, t))
            }
            fn og_path_length(&self, world: u32, path: f64) -> String {
                $crate::host_impl::answer($crate::og_path_length(world, path))
            }
            fn og_path_nearest(&self, world: u32, path: f64, point: String) -> String {
                $crate::host_impl::answer($crate::og_path_nearest(world, path, point))
            }
        }

        impl $crate::host_impl::api::TimelineApi for $host {
            fn og_timeline_info(&self, world: u32, asset: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_info(world, asset))
            }
            fn og_timeline_play(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_play(world, entity))
            }
            fn og_timeline_pause(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_pause(world, entity))
            }
            fn og_timeline_seek(&self, world: u32, entity: f64, time: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_seek(world, entity, time))
            }
            fn og_timeline_stop(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_timeline_stop(world, entity))
            }
        }

        impl $crate::host_impl::api::MotionApi for $host {
            fn og_motion_pose(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_pose(world, entity, desc))
            }
            fn og_motion_place(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_place(world, entity, desc))
            }
            fn og_motion_face(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_face(world, entity, desc))
            }
            fn og_motion_walk_to(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_walk_to(world, entity, desc))
            }
            fn og_motion_jump(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_jump(world, entity, desc))
            }
            fn og_motion_clear(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_motion_clear(world, entity))
            }
            fn og_motion_look_at(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_look_at(world, entity, desc))
            }
            fn og_motion_react(&self, world: u32, entity: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_motion_react(world, entity, desc))
            }
            fn og_motion_status(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_motion_status(world, entity))
            }
            fn og_motion_capabilities(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_motion_capabilities(world))
            }
            fn og_motion_set_backend(&self, world: u32, backend: String) -> String {
                $crate::host_impl::answer($crate::og_motion_set_backend(world, backend))
            }
            fn og_motion_presentation(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_motion_presentation(world, entity))
            }
            fn og_motion_poses(&self) -> String {
                $crate::host_impl::json($crate::og_motion_poses())
            }
        }
    };
}
