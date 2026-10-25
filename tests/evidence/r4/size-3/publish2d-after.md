# publish2d, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish2d.wasm: 16811525 B raw, 10092821 B after wasm-opt, 2935753 B brotli -q 11 (70.0 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 9,999,134 B raw; "brotli alone" is each group compressed by itself (they sum to 3,058,770 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| [data] | 1 | 1,347,522 | 13.48 % | 334,673 | 10.94 % |
| og_runtime | 685 | 962,113 | 9.62 % | 316,408 | 10.34 % |
| og_host_web (exports) | 382 | 1,062,353 | 10.62 % | 297,908 | 9.74 % |
| og_render | 572 | 729,227 | 7.29 % | 234,661 | 7.67 % |
| rapier2d | 320 | 576,938 | 5.77 % | 185,809 | 6.07 % |
| naga | 1,180 | 599,778 | 6.00 % | 180,690 | 5.91 % |
| og_physics | 180 | 574,858 | 5.75 % | 180,651 | 5.91 % |
| parry2d | 780 | 445,505 | 4.46 % | 128,013 | 4.19 % |
| wgpu | 376 | 344,598 | 3.45 % | 123,797 | 4.05 % |
| og_ui | 227 | 274,710 | 2.75 % | 96,403 | 3.15 % |
| og_anim | 254 | 294,372 | 2.94 % | 93,578 | 3.06 % |
| wgpu_core | 911 | 312,505 | 3.13 % | 89,247 | 2.92 % |
| std | 1,259 | 296,938 | 2.97 % | 81,462 | 2.66 % |
| wgpu_hal | 351 | 161,553 | 1.62 % | 59,698 | 1.95 % |
| og_ecs | 461 | 175,798 | 1.76 % | 49,041 | 1.60 % |
| bincode | 71 | 175,103 | 1.75 % | 48,927 | 1.60 % |
| og_core | 676 | 194,931 | 1.95 % | 44,033 | 1.44 % |
| ttf_parser | 292 | 129,768 | 1.30 % | 42,805 | 1.40 % |
| rustybuzz | 294 | 108,882 | 1.09 % | 40,499 | 1.32 % |
| og_nav | 72 | 104,561 | 1.05 % | 37,903 | 1.24 % |
| og_audio | 102 | 71,481 | 0.71 % | 27,929 | 0.91 % |
| og_ai | 66 | 78,561 | 0.79 % | 27,049 | 0.88 % |
| serde_json | 251 | 81,861 | 0.82 % | 26,536 | 0.87 % |
| og_probe | 85 | 71,041 | 0.71 % | 26,208 | 0.86 % |
| og_cook | 265 | 77,498 | 0.78 % | 25,959 | 0.85 % |
| js_sys | 63 | 71,576 | 0.72 % | 25,559 | 0.84 % |
| [unnamed] | 343 | 71,138 | 0.71 % | 22,249 | 0.73 % |
| og_geom | 59 | 42,381 | 0.42 % | 16,472 | 0.54 % |
| og_host_web | 63 | 40,070 | 0.40 % | 14,816 | 0.48 % |
| og_i18n | 79 | 25,775 | 0.26 % | 10,415 | 0.34 % |
| og_tilemap | 40 | 25,012 | 0.25 % | 10,318 | 0.34 % |
| glam | 69 | 26,171 | 0.26 % | 10,077 | 0.33 % |
| og_net | 52 | 27,880 | 0.28 % | 9,827 | 0.32 % |
| indexmap | 114 | 42,534 | 0.43 % | 9,767 | 0.32 % |
| wgpu_types | 261 | 26,766 | 0.27 % | 9,513 | 0.31 % |
| miniz_oxide | 30 | 20,049 | 0.20 % | 8,873 | 0.29 % |
| codespan_reporting | 57 | 21,178 | 0.21 % | 8,430 | 0.28 % |
| og_input | 70 | 31,990 | 0.32 % | 8,371 | 0.27 % |
| spade | 42 | 23,633 | 0.24 % | 8,098 | 0.26 % |
| compiler_builtins | 19 | 19,291 | 0.19 % | 7,683 | 0.25 % |
| robust | 7 | 28,597 | 0.29 % | 7,675 | 0.25 % |
| og_lanes | 71 | 30,966 | 0.31 % | 7,405 | 0.24 % |
| og_tween | 40 | 18,466 | 0.18 % | 7,298 | 0.24 % |
| [other] | 84 | 19,381 | 0.19 % | 7,277 | 0.24 % |
| og_timeline | 42 | 14,013 | 0.14 % | 5,837 | 0.19 % |
| dlmalloc | 15 | 12,487 | 0.12 % | 3,705 | 0.12 % |
| libm | 13 | 8,992 | 0.09 % | 3,329 | 0.11 % |
| og_blob | 28 | 14,680 | 0.15 % | 3,266 | 0.11 % |
| naga_types | 55 | 6,765 | 0.07 % | 3,196 | 0.10 % |
| arrayvec | 98 | 7,138 | 0.07 % | 2,967 | 0.10 % |
| smallvec | 59 | 10,276 | 0.10 % | 2,958 | 0.10 % |
| glow | 77 | 10,391 | 0.10 % | 2,831 | 0.09 % |
| nalgebra | 9 | 5,426 | 0.05 % | 2,384 | 0.08 % |
| unicode_bidi | 14 | 5,857 | 0.06 % | 2,371 | 0.08 % |
| wasm_bindgen | 47 | 5,274 | 0.05 % | 1,925 | 0.06 % |
| rustc_hash | 27 | 3,545 | 0.04 % | 1,351 | 0.04 % |
| parking_lot | 6 | 3,837 | 0.04 % | 1,226 | 0.04 % |
| serde_core | 23 | 3,048 | 0.03 % | 1,158 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,104 | 0.04 % |
| parking_lot_core | 9 | 1,613 | 0.02 % | 833 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 826 | 0.03 % |
| unicode_segmentation | 8 | 1,580 | 0.02 % | 809 | 0.03 % |
| foldhash | 4 | 3,050 | 0.03 % | 655 | 0.02 % |
| once_cell | 5 | 1,081 | 0.01 % | 634 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 518 | 0.02 % |
| glamx | 3 | 1,100 | 0.01 % | 513 | 0.02 % |
| bitflags | 9 | 1,410 | 0.01 % | 479 | 0.02 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 410 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 403 | 0.01 % |
| bit_vec | 8 | 628 | 0.01 % | 398 | 0.01 % |
| web_sys | 26 | 956 | 0.01 % | 394 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 387 | 0.01 % |
| bit_set | 5 | 486 | 0.00 % | 350 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 322 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 262 | 0.01 % |
| lock_api | 2 | 310 | 0.00 % | 236 | 0.01 % |
| __rustc | 5 | 325 | 0.00 % | 208 | 0.01 % |
| bytemuck | 5 | 324 | 0.00 % | 197 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 155 | 0.01 % |
| simd_adler32 | 1 | 201 | 0.00 % | 140 | 0.00 % |
| serde_arrays | 1 | 19 | 0.00 % | 23 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 411,600 | 137,119 | og_render | `<og_render::Renderer>::render` |
| 353,586 | 121,555 | og_physics | `og_physics::dim2::sim::step_here` |
| 193,112 | 49,408 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 162,685 | 61,190 | og_runtime | `og_runtime::extract::extract` |
| 120,474 | 34,363 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 91,926 | 34,737 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 78,545 | 27,480 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 66,667 | 23,599 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 65,114 | 24,858 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 54,681 | 21,730 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 50,911 | 18,141 | og_anim | `og_anim::system::install::{closure#0}` |
| 50,134 | 17,370 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 48,476 | 8,957 | og_physics | `og_physics::gather::gather` |
| 39,829 | 16,016 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 38,261 | 11,133 | og_anim | `og_anim::rig2d::parse_body` |
| 35,933 | 14,113 | og_runtime | `og_runtime::timeline::decode` |
| 30,889 | 10,159 | og_runtime | `og_runtime::engine::call_rpc` |
| 30,594 | 11,122 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,564 | 9,056 | og_host_web (exports) | `og_room_deliver` |
| 29,369 | 10,309 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 29,280 | 5,982 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 28,627 | 11,646 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,469 | 11,329 | naga | `<naga::valid::Validator>::validate_impl` |
| 27,332 | 8,729 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 26,680 | 6,807 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 26,285 | 10,795 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 25,936 | 9,949 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 24,457 | 3,887 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 23,982 | 7,856 | og_runtime | `og_runtime::modules::load_style` |
| 23,736 | 5,211 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,600 | 8,931 | og_audio | `og_audio::system::run` |
| 22,875 | 9,116 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,830 | 8,712 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,983 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,260 | 9,389 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 22,248 | 8,287 | og_runtime | `og_runtime::anim::load_animation` |
| 22,043 | 8,921 | naga | `<naga::valid::Validator>::validate_function` |
| 21,828 | 9,340 | og_nav | `og_nav::system::install::{closure#0}` |
| 21,333 | 8,994 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,155 | 8,209 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,378 | 7,419 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 20,080 | 7,383 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,630 | 7,731 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 19,069 | 7,841 | og_ui | `og_ui::text::bidi_runs` |
| 18,794 | 6,072 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,586 | 5,918 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 18,584 | 7,815 | rapier2d | `<rapier2d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |
| 18,381 | 7,765 | og_ai | `og_ai::system::install::{closure#0}` |
| 18,180 | 7,716 | og_render | `og_render::id_pass::render_ids_from` |
| 17,979 | 6,382 | og_physics | `og_physics::schedule::{closure#0}` |
| 17,859 | 6,359 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,513 | 5,762 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 17,439 | 7,200 | og_runtime | `og_runtime::save::apply_keeping` |
| 16,666 | 7,584 | og_runtime | `og_runtime::stream::fetch` |
| 16,649 | 6,468 | rapier2d | `<rapier2d::pipeline::physics_world::PhysicsWorld>::remove_body` |
| 16,261 | 5,865 | bincode | `<&mut bincode::de::Deserializer<bincode::de::read::SliceReader, bincode::config::WithOtherLimit<bincode::config::WithOtherTrailing<bincode::config::DefaultOp...` |
| 16,252 | 6,972 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 16,224 | 6,667 | og_host_web (exports) | `og_engine_frame multivalue shim` |
| 16,205 | 6,220 | rapier2d | `<rapier2d::dynamics::soft_body::collision_mesh::collision_mesh::SoftCollisionMesh>::remap_topology` |
| 16,141 | 5,632 | naga | `<naga::proc::constant_evaluator::ConstantEvaluator>::try_eval_and_append` |

twiggy top, per crate (shallow bytes; names and producers left out): 10,092,544 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 5816 | 1,347,520 | 13.35 % |
| og_host_web (exports) | 382 | 1,063,505 | 10.54 % |
| og_runtime | 685 | 963,929 | 9.55 % |
| og_render | 572 | 730,760 | 7.24 % |
| naga | 1180 | 602,631 | 5.97 % |
| rapier2d | 320 | 577,903 | 5.73 % |
| og_physics | 180 | 575,329 | 5.70 % |
| parry2d | 780 | 447,551 | 4.43 % |
| wgpu | 376 | 345,534 | 3.42 % |
| wgpu_core | 911 | 314,773 | 3.12 % |
| std | 1259 | 299,983 | 2.97 % |
| og_anim | 254 | 295,074 | 2.92 % |
| og_ui | 227 | 275,348 | 2.73 % |
| og_core | 676 | 196,622 | 1.95 % |
| og_ecs | 461 | 177,095 | 1.75 % |
| bincode | 71 | 175,309 | 1.74 % |
| wgpu_hal | 351 | 162,384 | 1.61 % |
| ttf_parser | 292 | 130,522 | 1.29 % |
| rustybuzz | 294 | 109,608 | 1.09 % |
| og_nav | 72 | 104,760 | 1.04 % |
| serde_json | 251 | 82,512 | 0.82 % |
| og_ai | 66 | 78,743 | 0.78 % |
| og_cook | 265 | 78,100 | 0.77 % |
| [unnamed] | 343 | 72,004 | 0.71 % |
| og_audio | 102 | 71,743 | 0.71 % |
| js_sys | 63 | 71,713 | 0.71 % |
| og_probe | 85 | 71,270 | 0.71 % |
| [wasm structure] | 1447 | 61,858 | 0.61 % |
| indexmap | 114 | 42,809 | 0.42 % |
| og_geom | 59 | 42,558 | 0.42 % |
| og_host_web | 63 | 40,231 | 0.40 % |
| og_input | 70 | 32,145 | 0.32 % |
| og_lanes | 71 | 31,171 | 0.31 % |
| robust | 7 | 28,622 | 0.28 % |
| og_net | 52 | 28,019 | 0.28 % |
| wgpu_types | 261 | 27,341 | 0.27 % |
| glam | 69 | 26,354 | 0.26 % |
| og_i18n | 79 | 25,976 | 0.26 % |
| og_tilemap | 40 | 25,113 | 0.25 % |
| spade | 42 | 23,741 | 0.24 % |
| codespan_reporting | 57 | 21,323 | 0.21 % |
| miniz_oxide | 30 | 20,121 | 0.20 % |
| [other] | 89 | 19,627 | 0.19 % |
| compiler_builtins | 19 | 19,349 | 0.19 % |
| og_tween | 40 | 18,567 | 0.18 % |
| og_blob | 28 | 14,754 | 0.15 % |
| og_timeline | 42 | 14,125 | 0.14 % |
| dlmalloc | 15 | 12,530 | 0.12 % |
| glow | 77 | 10,579 | 0.10 % |
| smallvec | 59 | 10,419 | 0.10 % |
| libm | 13 | 9,030 | 0.09 % |
| arrayvec | 98 | 7,346 | 0.07 % |
| naga_types | 55 | 6,887 | 0.07 % |
| unicode_bidi | 14 | 5,890 | 0.06 % |
| nalgebra | 9 | 5,451 | 0.05 % |
| wasm_bindgen | 47 | 5,381 | 0.05 % |
| parking_lot | 6 | 3,855 | 0.04 % |
| rustc_hash | 27 | 3,605 | 0.04 % |
| serde_core | 23 | 3,101 | 0.03 % |
| foldhash | 4 | 3,062 | 0.03 % |
| zmij | 1 | 2,152 | 0.02 % |
| half | 21 | 1,880 | 0.02 % |
| parking_lot_core | 9 | 1,634 | 0.02 % |
| unicode_segmentation | 8 | 1,600 | 0.02 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,402 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| glamx | 3 | 1,108 | 0.01 % |
| once_cell | 5 | 1,094 | 0.01 % |
| web_sys | 26 | 1,008 | 0.01 % |
| fdeflate | 1 | 829 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_vec | 8 | 645 | 0.01 % |
| bit_set | 5 | 498 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| __rustc | 5 | 336 | 0.00 % |
| bytemuck | 5 | 334 | 0.00 % |
| lock_api | 2 | 315 | 0.00 % |
| simd_adler32 | 1 | 204 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % |
