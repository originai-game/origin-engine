//! Simulation's bindings (PhysicsApi, DestructionApi, NavApi, AiApi, TilemapApi).
//! Expanded for a host by [`crate::host_bindings!`].

#[doc(hidden)]
#[macro_export]
macro_rules! __og_host_bindings_simulation {
    ($host:ty) => {
        impl $crate::host_impl::api::PhysicsApi for $host {
            fn og_physics_rope_points(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_rope_points(world, entity))
            }
            fn og_physics_cloth_points(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_cloth_points(world, entity))
            }
            fn og_physics_soft_body_points(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_soft_body_points(world, entity))
            }

            fn og_physics_garment_points(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_garment_points(world, entity))
            }
            fn og_physics_fluid_particles(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_fluid_particles(world, entity))
            }
            fn og_physics_fluid3d_particles(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_fluid3d_particles(world, entity))
            }
            fn og_physics_water(&self, world: u32, point: String) -> String {
                $crate::host_impl::answer($crate::og_physics_water(world, point))
            }
            fn og_physics_water_height(&self, world: u32, entity: f64, x: f64, z: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_water_height(world, entity, x, z))
            }
            fn og_physics_vehicle_wheels(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_vehicle_wheels(world, entity))
            }
            fn og_physics_apply_force(
                &self,
                world: u32,
                entity: f64,
                force: String,
                point: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_physics_apply_force(
                    world, entity, force, point,
                ))
            }
            fn og_physics_apply_impulse(
                &self,
                world: u32,
                entity: f64,
                impulse: String,
                point: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_physics_apply_impulse(
                    world, entity, impulse, point,
                ))
            }
            fn og_physics_apply_torque(&self, world: u32, entity: f64, torque: String) -> String {
                $crate::host_impl::answer($crate::og_physics_apply_torque(world, entity, torque))
            }
            fn og_physics_apply_torque_impulse(
                &self,
                world: u32,
                entity: f64,
                torque: String,
            ) -> String {
                $crate::host_impl::answer($crate::og_physics_apply_torque_impulse(
                    world, entity, torque,
                ))
            }
            fn og_physics_ragdoll_rig_from_skeleton(&self, world: u32, skeleton: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_ragdoll_rig_from_skeleton(
                    world, skeleton,
                ))
            }
            fn og_physics_set_ragdoll_rig(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_physics_set_ragdoll_rig(world, entity, desc))
            }
            fn og_physics_ragdoll_pose(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_physics_ragdoll_pose(world, entity))
            }
            fn og_physics_stats(&self, world: u32) -> String {
                $crate::host_impl::answer($crate::og_physics_stats(world))
            }
            fn og_physics_cast_ray(&self, world: u32, desc: Option<String>) -> String {
                $crate::og_physics_raycast(world, desc)
            }
            fn og_physics_cast_ray_all(&self, world: u32, desc: Option<String>) -> String {
                $crate::og_physics_raycast_all(world, desc)
            }
            fn og_physics_project_point(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_physics_project_point(world, desc))
            }
            fn og_physics_cast_shape(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_physics_shape_cast(world, desc))
            }
            fn og_physics_overlap(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_physics_overlap(world, desc))
            }
            fn og_physics_set_mesh(&self, world: u32, mesh: f64, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_physics_set_mesh(world, mesh, desc))
            }
        }

        impl $crate::host_impl::api::DestructionApi for $host {
            fn og_destruction_damage(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_damage(world, entity, desc))
            }
            fn og_destruction_explode(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_destruction_explode(world, desc))
            }
            fn og_destruction_fracture(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_destruction_fracture(world, entity))
            }
            fn og_destruction_info(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_destruction_info(world, entity))
            }
            fn og_destruction_material(&self, world: u32, kind: u32) -> String {
                $crate::host_impl::answer($crate::og_destruction_material(world, kind))
            }
            fn og_destruction_capabilities(&self, world: u32) -> String {
                $crate::host_impl::json($crate::og_destruction_capabilities(world))
            }
            fn og_destruction_carve2d(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_carve2d(world, entity, desc))
            }
            fn og_destruction_fill2d(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_fill2d(world, entity, desc))
            }
            fn og_destruction_explode2d(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_destruction_explode2d(world, desc))
            }
            fn og_destruction_fracture2d(
                &self,
                world: u32,
                entity: f64,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_destruction_fracture2d(world, entity, desc))
            }
            fn og_destruction_cell2d(&self, world: u32, entity: f64, point: String) -> String {
                $crate::host_impl::answer($crate::og_destruction_cell2d(world, entity, point))
            }
        }

        impl $crate::host_impl::api::NavApi for $host {
            fn og_nav_grid(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_nav_grid(world, desc))
            }
            fn og_nav_grid_from_tilemap(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_nav_grid_from_tilemap(world, desc))
            }
            fn og_nav_set_cost(&self, world: u32, grid: f64, x: u32, y: u32, cost: u32) -> String {
                $crate::host_impl::answer($crate::og_nav_set_cost(world, grid, x, y, cost))
            }
            fn og_nav_cell_at(&self, world: u32, grid: f64, point: String) -> String {
                $crate::host_impl::json($crate::og_nav_cell_at(world, grid, point))
            }
            fn og_nav_find_path(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_nav_find_path(world, desc))
            }
            fn og_nav_flow_field(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_nav_flow_field(world, desc))
            }
            fn og_nav_set_flow_targets(&self, world: u32, field: f64, targets: String) -> String {
                $crate::host_impl::answer($crate::og_nav_set_flow_targets(world, field, targets))
            }
            fn og_nav_flow_at(&self, world: u32, field: f64, point: String) -> String {
                $crate::host_impl::json($crate::og_nav_flow_at(world, field, point))
            }
            fn og_nav_load(&self, world: u32, path: String, profile: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_nav_load(world, path, profile))
            }
            fn og_nav_build(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_nav_build(world, desc))
            }
            fn og_nav_set_obstacle(
                &self,
                world: u32,
                mesh: f64,
                id: u32,
                obstacle: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_nav_set_obstacle(world, mesh, id, obstacle))
            }
            fn og_nav_carve(
                &self,
                world: u32,
                mesh: f64,
                id: u32,
                shape: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_nav_carve(world, mesh, id, shape))
            }
            fn og_nav_set_area(
                &self,
                world: u32,
                mesh: f64,
                id: u32,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_nav_set_area(world, mesh, id, desc))
            }
            fn og_nav_set_link(
                &self,
                world: u32,
                mesh: f64,
                id: u32,
                desc: Option<String>,
            ) -> String {
                $crate::host_impl::answer($crate::og_nav_set_link(world, mesh, id, desc))
            }
            fn og_nav_set_link_open(&self, world: u32, mesh: f64, id: u32, open: bool) -> String {
                $crate::host_impl::answer($crate::og_nav_set_link_open(world, mesh, id, open))
            }
            fn og_nav_finish_link(&self, world: u32, entity: f64) -> bool {
                $crate::og_nav_finish_link(world, entity)
            }
            fn og_nav_nearest(
                &self,
                world: u32,
                mesh: f64,
                point: String,
                extent: Option<String>,
            ) -> String {
                $crate::host_impl::json($crate::og_nav_nearest(world, mesh, point, extent))
            }
            fn og_nav_path_of(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::json($crate::og_nav_path_of(world, entity))
            }
        }

        impl $crate::host_impl::api::AiApi for $host {
            fn og_ai_load(&self, world: u32, path: String) -> String {
                $crate::host_impl::answer($crate::og_ai_load(world, path))
            }
            fn og_ai_parse(&self, world: u32, text: String) -> String {
                $crate::host_impl::answer($crate::og_ai_parse(world, text))
            }
            fn og_ai_get(&self, world: u32, entity: f64, key: String) -> String {
                $crate::host_impl::json($crate::og_ai_get(world, entity, key))
            }
            fn og_ai_set(&self, world: u32, entity: f64, key: String, value: String) -> String {
                $crate::host_impl::answer($crate::og_ai_set(world, entity, key, value))
            }
            fn og_ai_noise(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_ai_noise(world, desc))
            }
            fn og_ai_restart(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::answer($crate::og_ai_restart(world, entity))
            }

            fn og_ai_finish(
                &self,
                world: u32,
                entity: f64,
                success: bool,
                action: Option<String>,
            ) -> bool {
                $crate::og_ai_finish(world, entity, success, action)
            }
            fn og_ai_describe(&self, world: u32, entity: f64) -> String {
                $crate::host_impl::json($crate::og_ai_describe(world, entity))
            }
        }

        impl $crate::host_impl::api::TilemapApi for $host {
            fn og_tilemap_set_tile(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tilemap_set_tile(world, desc))
            }
            fn og_tilemap_fill(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tilemap_fill(world, desc))
            }
            fn og_tilemap_create(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tilemap_create(world, desc))
            }
            fn og_tilemap_resize(&self, world: u32, desc: Option<String>) -> String {
                $crate::host_impl::answer($crate::og_tilemap_resize(world, desc))
            }
            fn og_tilemap_neighbours(&self, world: u32, map: f64, x: u32, y: u32) -> String {
                $crate::host_impl::json($crate::og_tilemap_neighbours(world, map, x, y))
            }
            fn og_tilemap_get_tile(
                &self,
                world: u32,
                entity: f64,
                layer: String,
                x: u32,
                y: u32,
            ) -> i32 {
                $crate::og_tilemap_get_tile(world, entity, layer, x, y)
            }
            fn og_tilemap_cell_at(&self, world: u32, entity: f64, point: String) -> String {
                $crate::host_impl::json($crate::og_tilemap_cell_at(world, entity, point))
            }
            fn og_tilemap_cell_center(&self, world: u32, entity: f64, x: u32, y: u32) -> String {
                $crate::host_impl::json($crate::og_tilemap_cell_center(world, entity, x, y))
            }
        }
    };
}
