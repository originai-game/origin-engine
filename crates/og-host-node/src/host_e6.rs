//! The E6 modules' trait implementations (NavApi, AiApi, PlatformApi, I18nApi): every
//! member is the host-neutral runtime's.

use crate::NodeHost;
use og_api::host::*;
use og_runtime as rt;

impl NavApi for NodeHost {
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
    fn og_nav_load(&self, world: u32, path: String) -> String {
        rt::og_nav_load(world, path)
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

impl AiApi for NodeHost {
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
    fn og_ai_describe(&self, world: u32, entity: f64) -> String {
        rt::og_ai_describe(world, entity)
    }
}

impl PlatformApi for NodeHost {
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
    fn og_platform_result(&self, world: u32, request: u32) -> String {
        rt::og_platform_result(world, request)
    }
    fn og_platform_info(&self, world: u32) -> String {
        rt::og_platform_info(world)
    }
    fn og_platform_now(&self, world: u32) -> f64 {
        rt::og_platform_now(world)
    }
}

impl I18nApi for NodeHost {
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
}
