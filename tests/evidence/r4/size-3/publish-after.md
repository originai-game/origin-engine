# publish, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish.wasm: 17874587 B raw, 10969279 B after wasm-opt, 3182706 B brotli -q 11 (75.9 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 10,874,104 B raw; "brotli alone" is each group compressed by itself (they sum to 3,312,237 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| [data] | 1 | 1,413,198 | 13.00 % | 346,376 | 10.46 % |
| og_runtime | 739 | 1,027,140 | 9.45 % | 337,798 | 10.20 % |
| og_host_web (exports) | 382 | 1,082,606 | 9.96 % | 304,575 | 9.20 % |
| og_render | 638 | 838,485 | 7.71 % | 273,749 | 8.26 % |
| rapier3d | 349 | 840,215 | 7.73 % | 271,283 | 8.19 % |
| parry3d | 980 | 754,781 | 6.94 % | 202,814 | 6.12 % |
| og_physics | 177 | 585,593 | 5.39 % | 186,867 | 5.64 % |
| naga | 1,176 | 599,307 | 5.51 % | 180,645 | 5.45 % |
| wgpu | 376 | 344,597 | 3.17 % | 123,905 | 3.74 % |
| og_ui | 225 | 274,733 | 2.53 % | 96,479 | 2.91 % |
| og_anim | 254 | 294,154 | 2.71 % | 93,370 | 2.82 % |
| wgpu_core | 911 | 312,487 | 2.87 % | 89,157 | 2.69 % |
| std | 1,297 | 308,824 | 2.84 % | 84,271 | 2.54 % |
| wgpu_hal | 350 | 161,754 | 1.49 % | 59,804 | 1.81 % |
| og_ecs | 462 | 175,876 | 1.62 % | 48,982 | 1.48 % |
| bincode | 78 | 174,140 | 1.60 % | 47,634 | 1.44 % |
| og_core | 695 | 197,622 | 1.82 % | 44,583 | 1.35 % |
| ttf_parser | 291 | 129,757 | 1.19 % | 42,839 | 1.29 % |
| rustybuzz | 294 | 108,883 | 1.00 % | 40,532 | 1.22 % |
| og_nav | 74 | 104,663 | 0.96 % | 37,938 | 1.15 % |
| og_cook | 285 | 82,046 | 0.75 % | 27,888 | 0.84 % |
| og_audio | 102 | 71,492 | 0.66 % | 27,886 | 0.84 % |
| og_probe | 93 | 74,811 | 0.69 % | 27,364 | 0.83 % |
| og_ai | 65 | 78,485 | 0.72 % | 27,035 | 0.82 % |
| serde_json | 252 | 81,963 | 0.75 % | 26,570 | 0.80 % |
| js_sys | 64 | 70,555 | 0.65 % | 25,282 | 0.76 % |
| [unnamed] | 342 | 70,774 | 0.65 % | 22,144 | 0.67 % |
| og_geom | 59 | 42,381 | 0.39 % | 16,481 | 0.50 % |
| og_host_web | 63 | 40,703 | 0.37 % | 15,025 | 0.45 % |
| og_i18n | 80 | 25,930 | 0.24 % | 10,496 | 0.32 % |
| og_tilemap | 39 | 25,000 | 0.23 % | 10,305 | 0.31 % |
| glam | 74 | 26,037 | 0.24 % | 10,075 | 0.30 % |
| og_net | 53 | 27,937 | 0.26 % | 9,852 | 0.30 % |
| indexmap | 117 | 44,879 | 0.41 % | 9,507 | 0.29 % |
| wgpu_types | 261 | 26,766 | 0.25 % | 9,466 | 0.29 % |
| miniz_oxide | 30 | 20,049 | 0.18 % | 8,875 | 0.27 % |
| codespan_reporting | 56 | 21,174 | 0.19 % | 8,425 | 0.25 % |
| og_input | 69 | 31,978 | 0.29 % | 8,387 | 0.25 % |
| spade | 43 | 23,493 | 0.22 % | 8,043 | 0.24 % |
| og_lanes | 80 | 34,106 | 0.31 % | 8,031 | 0.24 % |
| robust | 7 | 28,597 | 0.26 % | 7,701 | 0.23 % |
| compiler_builtins | 19 | 19,291 | 0.18 % | 7,681 | 0.23 % |
| [other] | 83 | 19,506 | 0.18 % | 7,321 | 0.22 % |
| og_tween | 40 | 18,466 | 0.17 % | 7,302 | 0.22 % |
| og_timeline | 46 | 14,058 | 0.13 % | 5,877 | 0.18 % |
| nalgebra | 14 | 9,208 | 0.08 % | 4,022 | 0.12 % |
| dlmalloc | 15 | 12,487 | 0.11 % | 3,701 | 0.11 % |
| libm | 13 | 8,992 | 0.08 % | 3,344 | 0.10 % |
| og_blob | 29 | 13,790 | 0.13 % | 3,245 | 0.10 % |
| naga_types | 54 | 6,598 | 0.06 % | 3,135 | 0.09 % |
| smallvec | 59 | 10,276 | 0.09 % | 2,962 | 0.09 % |
| glow | 77 | 10,391 | 0.10 % | 2,813 | 0.08 % |
| arrayvec | 93 | 6,339 | 0.06 % | 2,625 | 0.08 % |
| unicode_bidi | 14 | 5,857 | 0.05 % | 2,367 | 0.07 % |
| wasm_bindgen | 48 | 5,285 | 0.05 % | 1,935 | 0.06 % |
| glamx | 5 | 3,438 | 0.03 % | 1,665 | 0.05 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,340 | 0.04 % |
| parking_lot | 6 | 3,837 | 0.04 % | 1,227 | 0.04 % |
| serde_core | 23 | 3,048 | 0.03 % | 1,156 | 0.03 % |
| zmij | 1 | 2,148 | 0.02 % | 1,104 | 0.03 % |
| parking_lot_core | 9 | 1,613 | 0.01 % | 832 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 828 | 0.02 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 814 | 0.02 % |
| foldhash | 4 | 3,050 | 0.03 % | 655 | 0.02 % |
| once_cell | 5 | 1,081 | 0.01 % | 629 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 517 | 0.02 % |
| bitflags | 9 | 1,410 | 0.01 % | 479 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 412 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 407 | 0.01 % |
| bit_vec | 8 | 628 | 0.01 % | 396 | 0.01 % |
| web_sys | 26 | 956 | 0.01 % | 391 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 388 | 0.01 % |
| bit_set | 5 | 486 | 0.00 % | 349 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 325 | 0.01 % |
| ena | 3 | 775 | 0.01 % | 324 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 260 | 0.01 % |
| lock_api | 2 | 310 | 0.00 % | 232 | 0.01 % |
| __rustc | 5 | 325 | 0.00 % | 208 | 0.01 % |
| bytemuck | 4 | 229 | 0.00 % | 166 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 156 | 0.00 % |
| simd_adler32 | 1 | 201 | 0.00 % | 140 | 0.00 % |
| serde_arrays | 1 | 64 | 0.00 % | 68 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 458,295 | 156,088 | og_render | `<og_render::Renderer>::render` |
| 351,364 | 122,383 | og_physics | `og_physics::schedule::{closure#0}` |
| 193,652 | 49,544 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 190,874 | 72,232 | og_runtime | `og_runtime::extract::extract` |
| 131,579 | 37,214 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 99,398 | 33,168 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 91,925 | 34,748 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 78,545 | 27,498 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 74,310 | 28,236 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 65,622 | 23,329 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 54,681 | 21,651 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 50,740 | 18,100 | og_anim | `og_anim::system::install::{closure#0}` |
| 50,134 | 17,441 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 48,476 | 9,015 | og_physics | `og_physics::gather::gather` |
| 39,829 | 15,997 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 38,265 | 11,148 | og_anim | `og_anim::rig2d::parse_body` |
| 36,142 | 12,735 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 35,515 | 13,890 | og_runtime | `og_runtime::timeline::decode` |
| 30,972 | 6,123 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 30,889 | 10,183 | og_runtime | `og_runtime::engine::call_rpc` |
| 30,594 | 11,146 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,564 | 9,075 | og_host_web (exports) | `og_room_deliver` |
| 28,627 | 11,639 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,469 | 11,329 | naga | `<naga::valid::Validator>::validate_impl` |
| 27,332 | 8,731 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 26,680 | 6,798 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 26,285 | 10,790 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 25,877 | 9,958 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 24,457 | 3,854 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 23,982 | 7,875 | og_runtime | `og_runtime::modules::load_style` |
| 23,736 | 5,211 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,612 | 8,944 | og_audio | `og_audio::system::run` |
| 22,893 | 9,121 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,731 | 8,639 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,982 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,260 | 9,366 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 22,248 | 8,302 | og_runtime | `og_runtime::anim::load_animation` |
| 22,043 | 8,919 | naga | `<naga::valid::Validator>::validate_function` |
| 21,830 | 9,347 | og_nav | `og_nav::system::install::{closure#0}` |
| 21,333 | 8,982 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,209 | 7,520 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 21,155 | 8,222 | og_ui | `<og_ui::ui::Ui>::edit` |
| 21,041 | 7,858 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 20,445 | 7,329 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,378 | 7,431 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 20,080 | 7,407 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,782 | 8,094 | og_physics | `og_physics::dim3::sim::shape` |
| 19,630 | 7,739 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 19,069 | 7,847 | og_ui | `og_ui::text::bidi_runs` |
| 18,837 | 6,374 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher>::contact_manifold_convex_co...` |
| 18,759 | 6,459 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher<rapier3d::geometry::contact_...` |
| 18,586 | 5,913 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 18,539 | 6,331 | parry3d | `<core::marker::PhantomData<parry3d::shape::shared_shape::SharedShape> as serde_core::de::DeserializeSeed>::deserialize::<&mut bincode::de::Deserializer<binco...` |
| 18,381 | 7,767 | og_ai | `og_ai::system::install::{closure#0}` |
| 18,244 | 7,643 | rapier3d | `<rapier3d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |
| 18,180 | 7,741 | og_render | `og_render::id_pass::render_ids_from` |
| 17,912 | 5,908 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 17,859 | 6,339 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,513 | 5,762 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 17,472 | 6,757 | rapier3d | `<rapier3d::pipeline::physics_world::PhysicsWorld>::remove_body` |

twiggy top, per crate (shallow bytes; names and producers left out): 10,969,002 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 5932 | 1,413,196 | 12.88 % |
| og_host_web (exports) | 382 | 1,083,751 | 9.88 % |
| og_runtime | 739 | 1,029,120 | 9.38 % |
| rapier3d | 349 | 841,273 | 7.67 % |
| og_render | 638 | 840,202 | 7.66 % |
| parry3d | 980 | 757,417 | 6.91 % |
| naga | 1176 | 602,152 | 5.49 % |
| og_physics | 177 | 586,052 | 5.34 % |
| wgpu | 376 | 345,533 | 3.15 % |
| wgpu_core | 911 | 314,755 | 2.87 % |
| std | 1297 | 311,961 | 2.84 % |
| og_anim | 254 | 294,856 | 2.69 % |
| og_ui | 225 | 275,368 | 2.51 % |
| og_core | 695 | 199,362 | 1.82 % |
| og_ecs | 462 | 177,175 | 1.62 % |
| bincode | 78 | 174,365 | 1.59 % |
| wgpu_hal | 350 | 162,581 | 1.48 % |
| ttf_parser | 291 | 130,509 | 1.19 % |
| rustybuzz | 294 | 109,609 | 1.00 % |
| og_nav | 74 | 104,866 | 0.96 % |
| og_cook | 285 | 82,699 | 0.75 % |
| serde_json | 252 | 82,616 | 0.75 % |
| og_ai | 65 | 78,665 | 0.72 % |
| og_probe | 93 | 75,061 | 0.68 % |
| og_audio | 102 | 71,754 | 0.65 % |
| [unnamed] | 342 | 71,628 | 0.65 % |
| js_sys | 64 | 70,694 | 0.64 % |
| [wasm structure] | 1438 | 62,061 | 0.57 % |
| indexmap | 117 | 45,162 | 0.41 % |
| og_geom | 59 | 42,557 | 0.39 % |
| og_host_web | 63 | 40,864 | 0.37 % |
| og_lanes | 80 | 34,338 | 0.31 % |
| og_input | 69 | 32,131 | 0.29 % |
| robust | 7 | 28,621 | 0.26 % |
| og_net | 53 | 28,078 | 0.26 % |
| wgpu_types | 261 | 27,339 | 0.25 % |
| glam | 74 | 26,233 | 0.24 % |
| og_i18n | 80 | 26,133 | 0.24 % |
| og_tilemap | 39 | 25,099 | 0.23 % |
| spade | 43 | 23,602 | 0.22 % |
| codespan_reporting | 56 | 21,317 | 0.19 % |
| miniz_oxide | 30 | 20,121 | 0.18 % |
| [other] | 88 | 19,747 | 0.18 % |
| compiler_builtins | 19 | 19,349 | 0.18 % |
| og_tween | 40 | 18,567 | 0.17 % |
| og_timeline | 46 | 14,178 | 0.13 % |
| og_blob | 29 | 13,865 | 0.13 % |
| dlmalloc | 15 | 12,530 | 0.11 % |
| glow | 77 | 10,579 | 0.10 % |
| smallvec | 59 | 10,419 | 0.09 % |
| nalgebra | 14 | 9,249 | 0.08 % |
| libm | 13 | 9,030 | 0.08 % |
| naga_types | 54 | 6,717 | 0.06 % |
| arrayvec | 93 | 6,535 | 0.06 % |
| unicode_bidi | 14 | 5,890 | 0.05 % |
| wasm_bindgen | 48 | 5,393 | 0.05 % |
| parking_lot | 6 | 3,855 | 0.04 % |
| rustc_hash | 27 | 3,605 | 0.03 % |
| glamx | 5 | 3,453 | 0.03 % |
| serde_core | 23 | 3,101 | 0.03 % |
| foldhash | 4 | 3,062 | 0.03 % |
| zmij | 1 | 2,152 | 0.02 % |
| half | 21 | 1,880 | 0.02 % |
| parking_lot_core | 9 | 1,634 | 0.01 % |
| unicode_segmentation | 8 | 1,600 | 0.01 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,401 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| once_cell | 5 | 1,094 | 0.01 % |
| web_sys | 26 | 1,008 | 0.01 % |
| fdeflate | 1 | 829 | 0.01 % |
| ena | 3 | 784 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_vec | 8 | 645 | 0.01 % |
| bit_set | 5 | 498 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| __rustc | 5 | 336 | 0.00 % |
| lock_api | 2 | 315 | 0.00 % |
| bytemuck | 4 | 237 | 0.00 % |
| simd_adler32 | 1 | 204 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 66 | 0.00 % |
