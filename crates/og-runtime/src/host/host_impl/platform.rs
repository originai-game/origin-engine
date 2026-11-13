//! The platform's bindings (PlatformApi, NetApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_platform {
    ($host:ty) => {
        impl $crate::host_impl::api::PlatformApi for $host {
            fn og_platform_store_get(&self, world: u32, key: String) -> String {
                $crate::host_impl::answer($crate::og_platform_store_get(world, key))
            }
            fn og_platform_store_set(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_store_set(world, desc))
            }
            fn og_platform_store_list(&self, world: u32) -> u32 {
                $crate::og_platform_store_list(world)
            }
            fn og_platform_leaderboard_submit(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_leaderboard_submit(world, desc))
            }
            fn og_platform_leaderboard_top(&self, world: u32, desc: Option<String>) -> u32 {
                $crate::og_platform_leaderboard_top(world, desc)
            }
            fn og_platform_leaderboard_around(&self, world: u32, desc: Option<String>) -> u32 {
                $crate::og_platform_leaderboard_around(world, desc)
            }
            fn og_platform_telemetry(
                &self,
                world: u32,
                name: String,
                attrs: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_platform_telemetry(world, name, attrs))
            }
            fn og_platform_point(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_point(world, desc))
            }
            fn og_platform_ai_mark(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_ai_mark(world, desc))
            }
            fn og_platform_load_settings(&self, world: u32) -> u32 {
                $crate::og_platform_load_settings(world)
            }
            fn og_platform_save_settings(&self, world: u32, value: String) -> String {
                $crate::host_impl::answer($crate::og_platform_save_settings(world, value))
            }
            fn og_platform_save_slot(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_save_slot(world, desc))
            }
            fn og_platform_load_slot(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_load_slot(world, desc))
            }
            fn og_platform_products(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_products(world, desc))
            }
            fn og_platform_purchase(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_purchase(world, desc))
            }
            fn og_platform_entitlements(&self, world: u32) -> u32 {
                $crate::og_platform_entitlements(world)
            }
            fn og_platform_consume(&self, world: u32, purchase: String) -> String {
                $crate::host_impl::answer($crate::og_platform_consume(world, purchase))
            }
            fn og_platform_grant(&self, world: u32, purchase: String) -> String {
                $crate::host_impl::answer($crate::og_platform_grant(world, purchase))
            }
            fn og_platform_granted(&self, world: u32, purchase: String) -> bool {
                $crate::og_platform_granted(world, purchase)
            }
            fn og_platform_ad_ready(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_ad_ready(world, desc))
            }
            fn og_platform_show_ad(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_show_ad(world, desc))
            }
            fn og_platform_achievement_unlock(&self, world: u32, id: String) -> String {
                $crate::host_impl::answer($crate::og_platform_achievement_unlock(world, id))
            }
            fn og_platform_achievement_progress(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_achievement_progress(world, desc))
            }
            fn og_platform_achievements(&self, world: u32) -> u32 {
                $crate::og_platform_achievements(world)
            }
            fn og_platform_player(&self, world: u32) -> u32 {
                $crate::og_platform_player(world)
            }
            fn og_platform_friends(&self, world: u32, desc: Option<String>) -> u32 {
                $crate::og_platform_friends(world, desc)
            }
            fn og_platform_invite_send(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_invite_send(world, desc))
            }
            fn og_platform_invites(&self, world: u32) -> u32 {
                $crate::og_platform_invites(world)
            }
            fn og_platform_share(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_share(world, desc))
            }
            fn og_platform_supports(&self, world: u32, method: String) -> bool {
                $crate::og_platform_supports(world, method)
            }
            fn og_platform_notice(&self, world: u32, notice: u32) -> String {
                $crate::host_impl::json($crate::og_platform_notice(world, notice))
            }
            fn og_platform_result(&self, world: u32, request: u32) -> String {
                $crate::host_impl::json($crate::og_platform_result(world, request))
            }
            fn og_platform_info(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_platform_info(world))
            }
            fn og_platform_now(&self, world: u32) -> f64 {
                $crate::og_platform_now(world)
            }
            fn og_platform_room_create(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_room_create(world, desc))
            }
            fn og_platform_quick_play(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_platform_quick_play(world, desc))
            }
            fn og_platform_room_join(&self, world: u32, room: String) -> String {
                $crate::host_impl::answer($crate::og_platform_room_join(world, room))
            }
            fn og_platform_room_leave(&self, world: u32) -> String {
                $crate::host_impl::answer($crate::og_platform_room_leave(world))
            }
            fn og_platform_room_send(&self, world: u32, data: String, to: Option<u32>) -> String {
                $crate::host_impl::answer($crate::og_platform_room_send(world, data, to))
            }
            fn og_platform_room_peers(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_platform_room_peers(world))
            }
            fn og_platform_room_info(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_platform_room_info(world))
            }
            fn og_platform_room_message(&self, world: u32, message: u32) -> String {
                $crate::host_impl::json($crate::og_platform_room_message(world, message))
            }
        }

        impl $crate::host_impl::api::NetApi for $host {
            fn og_net_submit(&self, world: u32, input: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_net_submit(world, input))
            }
            fn og_net_shape(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_net_shape(world, desc))
            }
            fn og_net_submit_input(&self, world: u32, component: String, value: String) -> String {
                $crate::host_impl::answer($crate::og_net_submit_input(world, component, value))
            }
            fn og_net_command(&self, world: u32, component: String, value: String) -> String {
                $crate::host_impl::answer($crate::og_net_command(world, component, value))
            }
            fn og_net_commands(&self, world: u32, component: String) -> String {
                $crate::host_impl::answer($crate::og_net_commands(world, component))
            }
            fn og_net_status(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_net_status(world))
            }
            fn og_net_hash(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_net_hash(world))
            }
            fn og_net_resimulating(&self, world: u32) -> bool {
                $crate::og_net_resimulating(world)
            }
            fn og_net_final_hash(&self, world: u32, tick: f64) -> String {
                $crate::host_impl::json($crate::og_net_final_hash(world, tick))
            }
        }
    };
}
