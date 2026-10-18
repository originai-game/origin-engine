//! The E6 modules' trait implementations (NavApi, AiApi): every
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
    fn og_ai_describe(&self, world: u32, entity: f64) -> String {
        rt::og_ai_describe(world, entity)
    }
}
