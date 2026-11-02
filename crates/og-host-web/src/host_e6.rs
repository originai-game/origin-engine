//! The E6 modules' trait implementations (NavApi, AiApi, PlatformApi, I18nApi, NetApi): every
//! member is the host-neutral runtime's.

use crate::WebHost;
use og_api::host::*;
use og_runtime as rt;

impl NavApi for WebHost {
    fn og_nav_grid(&self, world: u32, desc: Option<String>) -> String {
        rt::og_nav_grid(world, desc)
    }
    fn og_nav_grid_from_tilemap(&self, world: u32, desc: Option<String>) -> String {
        rt::og_nav_grid_from_tilemap(world, desc)
    }
    fn og_nav_set_cost(&self, world: u32, grid: f64, x: u32, y: u32, cost: u32) -> String {
        rt::og_nav_set_cost(world, grid, x, y, cost)
    }
    fn og_nav_cell_at(&self, world: u32, grid: f64, point: String) -> String {
        rt::og_nav_cell_at(world, grid, point)
    }
    fn og_nav_find_path(&self, world: u32, desc: Option<String>) -> String {
        rt::og_nav_find_path(world, desc)
    }
    fn og_nav_flow_field(&self, world: u32, desc: Option<String>) -> String {
        rt::og_nav_flow_field(world, desc)
    }
    fn og_nav_set_flow_targets(&self, world: u32, field: f64, targets: String) -> String {
        rt::og_nav_set_flow_targets(world, field, targets)
    }
    fn og_nav_flow_at(&self, world: u32, field: f64, point: String) -> String {
        rt::og_nav_flow_at(world, field, point)
    }
    fn og_nav_load(&self, world: u32, path: String, profile: Option<String>) -> String {
        rt::og_nav_load(world, path, profile)
    }
    fn og_nav_build(&self, world: u32, desc: Option<String>) -> String {
        rt::og_nav_build(world, desc)
    }
    fn og_nav_set_obstacle(
        &self,
        world: u32,
        mesh: f64,
        id: u32,
        obstacle: Option<String>,
    ) -> String {
        rt::og_nav_set_obstacle(world, mesh, id, obstacle)
    }
    fn og_nav_carve(&self, world: u32, mesh: f64, id: u32, shape: Option<String>) -> String {
        rt::og_nav_carve(world, mesh, id, shape)
    }
    fn og_nav_set_area(&self, world: u32, mesh: f64, id: u32, desc: Option<String>) -> String {
        rt::og_nav_set_area(world, mesh, id, desc)
    }
    fn og_nav_set_link(&self, world: u32, mesh: f64, id: u32, desc: Option<String>) -> String {
        rt::og_nav_set_link(world, mesh, id, desc)
    }
    fn og_nav_set_link_open(&self, world: u32, mesh: f64, id: u32, open: bool) -> String {
        rt::og_nav_set_link_open(world, mesh, id, open)
    }
    fn og_nav_finish_link(&self, world: u32, entity: f64) -> bool {
        rt::og_nav_finish_link(world, entity)
    }
    fn og_nav_nearest(
        &self,
        world: u32,
        mesh: f64,
        point: String,
        extent: Option<String>,
    ) -> String {
        rt::og_nav_nearest(world, mesh, point, extent)
    }
    fn og_nav_path_of(&self, world: u32, entity: f64) -> String {
        rt::og_nav_path_of(world, entity)
    }
}

impl AiApi for WebHost {
    fn og_ai_load(&self, world: u32, path: String) -> String {
        rt::og_ai_load(world, path)
    }
    fn og_ai_parse(&self, world: u32, text: String) -> String {
        rt::og_ai_parse(world, text)
    }
    fn og_ai_get(&self, world: u32, entity: f64, key: String) -> String {
        rt::og_ai_get(world, entity, key)
    }
    fn og_ai_set(&self, world: u32, entity: f64, key: String, value: String) -> String {
        rt::og_ai_set(world, entity, key, value)
    }
    fn og_ai_noise(&self, world: u32, desc: Option<String>) -> String {
        rt::og_ai_noise(world, desc)
    }
    fn og_ai_restart(&self, world: u32, entity: f64) -> String {
        rt::og_ai_restart(world, entity)
    }

    fn og_ai_finish(&self, world: u32, entity: f64, success: bool, action: Option<String>) -> bool {
        rt::og_ai_finish(world, entity, success, action)
    }
    fn og_ai_describe(&self, world: u32, entity: f64) -> String {
        rt::og_ai_describe(world, entity)
    }
}

impl PlatformApi for WebHost {
    fn og_platform_store_get(&self, world: u32, key: String) -> String {
        rt::og_platform_store_get(world, key)
    }
    fn og_platform_store_set(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_store_set(world, desc)
    }
    fn og_platform_store_list(&self, world: u32) -> u32 {
        rt::og_platform_store_list(world)
    }
    fn og_platform_leaderboard_submit(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_leaderboard_submit(world, desc)
    }
    fn og_platform_leaderboard_top(&self, world: u32, desc: Option<String>) -> u32 {
        rt::og_platform_leaderboard_top(world, desc)
    }
    fn og_platform_leaderboard_around(&self, world: u32, desc: Option<String>) -> u32 {
        rt::og_platform_leaderboard_around(world, desc)
    }
    fn og_platform_telemetry(&self, world: u32, name: String, attrs: Option<String>) -> String {
        rt::og_platform_telemetry(world, name, attrs)
    }
    fn og_platform_point(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_point(world, desc)
    }
    fn og_platform_ai_mark(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_ai_mark(world, desc)
    }
    fn og_platform_load_settings(&self, world: u32) -> u32 {
        rt::og_platform_load_settings(world)
    }
    fn og_platform_save_settings(&self, world: u32, value: String) -> String {
        rt::og_platform_save_settings(world, value)
    }
    fn og_platform_save_slot(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_save_slot(world, desc)
    }
    fn og_platform_load_slot(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_load_slot(world, desc)
    }
    fn og_platform_products(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_products(world, desc)
    }
    fn og_platform_purchase(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_purchase(world, desc)
    }
    fn og_platform_entitlements(&self, world: u32) -> u32 {
        rt::og_platform_entitlements(world)
    }
    fn og_platform_consume(&self, world: u32, purchase: String) -> String {
        rt::og_platform_consume(world, purchase)
    }
    fn og_platform_grant(&self, world: u32, purchase: String) -> String {
        rt::og_platform_grant(world, purchase)
    }
    fn og_platform_granted(&self, world: u32, purchase: String) -> bool {
        rt::og_platform_granted(world, purchase)
    }
    fn og_platform_ad_ready(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_ad_ready(world, desc)
    }
    fn og_platform_show_ad(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_show_ad(world, desc)
    }
    fn og_platform_achievement_unlock(&self, world: u32, id: String) -> String {
        rt::og_platform_achievement_unlock(world, id)
    }
    fn og_platform_achievement_progress(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_achievement_progress(world, desc)
    }
    fn og_platform_achievements(&self, world: u32) -> u32 {
        rt::og_platform_achievements(world)
    }
    fn og_platform_player(&self, world: u32) -> u32 {
        rt::og_platform_player(world)
    }
    fn og_platform_friends(&self, world: u32, desc: Option<String>) -> u32 {
        rt::og_platform_friends(world, desc)
    }
    fn og_platform_invite_send(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_invite_send(world, desc)
    }
    fn og_platform_invites(&self, world: u32) -> u32 {
        rt::og_platform_invites(world)
    }
    fn og_platform_share(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_share(world, desc)
    }
    fn og_platform_supports(&self, world: u32, method: String) -> bool {
        rt::og_platform_supports(world, method)
    }
    fn og_platform_notice(&self, world: u32, notice: u32) -> String {
        rt::og_platform_notice(world, notice)
    }
    fn og_platform_result(&self, world: u32, request: u32) -> String {
        rt::og_platform_result(world, request)
    }
    fn og_platform_info(&self, world: u32) -> String {
        rt::og_platform_info(world)
    }
    fn og_platform_now(&self, world: u32) -> f64 {
        rt::og_platform_now(world)
    }
    fn og_platform_room_create(&self, world: u32, desc: Option<String>) -> String {
        rt::og_platform_room_create(world, desc)
    }
    fn og_platform_room_join(&self, world: u32, room: String) -> String {
        rt::og_platform_room_join(world, room)
    }
    fn og_platform_room_leave(&self, world: u32) -> String {
        rt::og_platform_room_leave(world)
    }
    fn og_platform_room_send(&self, world: u32, data: String, to: Option<u32>) -> String {
        rt::og_platform_room_send(world, data, to)
    }
    fn og_platform_room_peers(&self, world: u32) -> String {
        rt::og_platform_room_peers(world)
    }
    fn og_platform_room_info(&self, world: u32) -> String {
        rt::og_platform_room_info(world)
    }
    fn og_platform_room_message(&self, world: u32, message: u32) -> String {
        rt::og_platform_room_message(world, message)
    }
}

impl I18nApi for WebHost {
    fn og_i18n_load(&self, world: u32, path: String) -> String {
        rt::og_i18n_load(world, path)
    }
    fn og_i18n_set_locale(&self, world: u32, locale: String) -> String {
        rt::og_i18n_set_locale(world, locale)
    }
    fn og_i18n_locale(&self, world: u32) -> String {
        rt::og_i18n_locale(world)
    }
    fn og_i18n_t(&self, world: u32, key: String, args: Option<String>) -> String {
        rt::og_i18n_t(world, key, args)
    }
    fn og_i18n_has(&self, world: u32, key: String) -> bool {
        rt::og_i18n_has(world, key)
    }
    fn og_i18n_format_compact(&self, world: u32, value: f64) -> String {
        rt::og_i18n_format_compact(world, value)
    }
    fn og_i18n_format_currency(&self, world: u32, amount: f64, currency: String) -> String {
        rt::og_i18n_format_currency(world, amount, currency)
    }
    fn og_i18n_format_date(&self, world: u32, seconds: f64, format: Option<String>) -> String {
        rt::og_i18n_format_date(world, seconds, format)
    }
    fn og_i18n_format_time(&self, world: u32, seconds: f64, format: Option<String>) -> String {
        rt::og_i18n_format_time(world, seconds, format)
    }
    fn og_i18n_format_duration(&self, world: u32, seconds: f64) -> String {
        rt::og_i18n_format_duration(world, seconds)
    }
}

impl NetApi for WebHost {
    fn og_net_submit(&self, world: u32, input: Option<String>) -> String {
        rt::og_net_submit(world, input)
    }
    fn og_net_status(&self, world: u32) -> String {
        rt::og_net_status(world)
    }
    fn og_net_hash(&self, world: u32) -> String {
        rt::og_net_hash(world)
    }
    fn og_net_resimulating(&self, world: u32) -> bool {
        rt::og_net_resimulating(world)
    }
    fn og_net_final_hash(&self, world: u32, tick: f64) -> String {
        rt::og_net_final_hash(world, tick)
    }
}
