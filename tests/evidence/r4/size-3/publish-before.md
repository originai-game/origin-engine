# publish, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish.wasm: 19166408 B raw, 11799681 B after wasm-opt, 3330189 B brotli -q 11 (79.4 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 11,701,165 B raw; "brotli alone" is each group compressed by itself (they sum to 3,491,687 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 1,012 | 1,168,776 | 9.99 % | 367,241 | 10.52 % |
| [data] | 1 | 1,397,219 | 11.94 % | 340,539 | 9.75 % |
| og_host_web (exports) | 382 | 1,180,927 | 10.09 % | 327,780 | 9.39 % |
| og_render | 664 | 857,946 | 7.33 % | 276,872 | 7.93 % |
| rapier3d | 351 | 842,845 | 7.20 % | 272,551 | 7.81 % |
| og_physics | 280 | 747,880 | 6.39 % | 216,338 | 6.20 % |
| parry3d | 982 | 756,522 | 6.47 % | 203,483 | 5.83 % |
| naga | 1,251 | 624,913 | 5.34 % | 186,523 | 5.34 % |
| wgpu | 376 | 360,356 | 3.08 % | 128,221 | 3.67 % |
| og_ui | 269 | 314,668 | 2.69 % | 107,494 | 3.08 % |
| std | 1,569 | 422,305 | 3.61 % | 106,791 | 3.06 % |
| og_anim | 326 | 351,216 | 3.00 % | 105,788 | 3.03 % |
| wgpu_core | 945 | 322,333 | 2.75 % | 91,721 | 2.63 % |
| wgpu_hal | 353 | 167,090 | 1.43 % | 61,525 | 1.76 % |
| og_nav | 93 | 149,363 | 1.28 % | 50,122 | 1.44 % |
| og_ecs | 473 | 183,258 | 1.57 % | 49,991 | 1.43 % |
| bincode | 78 | 167,931 | 1.44 % | 45,647 | 1.31 % |
| ttf_parser | 291 | 129,757 | 1.11 % | 42,843 | 1.23 % |
| rustybuzz | 294 | 108,883 | 0.93 % | 40,402 | 1.16 % |
| og_core | 536 | 169,699 | 1.45 % | 36,391 | 1.04 % |
| og_audio | 116 | 87,567 | 0.75 % | 32,985 | 0.94 % |
| og_cook | 289 | 97,812 | 0.84 % | 31,712 | 0.91 % |
| og_ai | 74 | 89,841 | 0.77 % | 30,024 | 0.86 % |
| [unnamed] | 380 | 98,068 | 0.84 % | 27,888 | 0.80 % |
| og_probe | 94 | 73,729 | 0.63 % | 27,111 | 0.78 % |
| serde_json | 254 | 82,126 | 0.70 % | 26,632 | 0.76 % |
| js_sys | 64 | 72,858 | 0.62 % | 26,355 | 0.75 % |
| og_host_web | 77 | 46,458 | 0.40 % | 17,032 | 0.49 % |
| og_geom | 59 | 42,381 | 0.36 % | 16,500 | 0.47 % |
| og_input | 69 | 56,037 | 0.48 % | 12,569 | 0.36 % |
| og_tilemap | 51 | 29,788 | 0.25 % | 12,216 | 0.35 % |
| glam | 83 | 32,515 | 0.28 % | 12,026 | 0.34 % |
| og_tween | 65 | 29,136 | 0.25 % | 10,904 | 0.31 % |
| og_net | 61 | 31,229 | 0.27 % | 10,891 | 0.31 % |
| og_i18n | 79 | 25,714 | 0.22 % | 10,405 | 0.30 % |
| wgpu_types | 285 | 29,725 | 0.25 % | 10,357 | 0.30 % |
| indexmap | 119 | 45,025 | 0.38 % | 9,541 | 0.27 % |
| miniz_oxide | 30 | 20,049 | 0.17 % | 8,874 | 0.25 % |
| codespan_reporting | 56 | 21,174 | 0.18 % | 8,423 | 0.24 % |
| og_lanes | 80 | 34,768 | 0.30 % | 8,196 | 0.23 % |
| spade | 44 | 23,504 | 0.20 % | 8,053 | 0.23 % |
| robust | 7 | 28,597 | 0.24 % | 7,723 | 0.22 % |
| compiler_builtins | 19 | 19,291 | 0.16 % | 7,672 | 0.22 % |
| [other] | 84 | 19,665 | 0.17 % | 7,310 | 0.21 % |
| og_timeline | 42 | 14,013 | 0.12 % | 5,841 | 0.17 % |
| nalgebra | 14 | 9,208 | 0.08 % | 4,027 | 0.12 % |
| dlmalloc | 15 | 12,487 | 0.11 % | 3,704 | 0.11 % |
| libm | 13 | 8,992 | 0.08 % | 3,328 | 0.10 % |
| og_blob | 29 | 13,790 | 0.12 % | 3,254 | 0.09 % |
| naga_types | 57 | 6,684 | 0.06 % | 3,165 | 0.09 % |
| smallvec | 59 | 10,276 | 0.09 % | 2,948 | 0.08 % |
| glow | 79 | 10,433 | 0.09 % | 2,830 | 0.08 % |
| arrayvec | 93 | 6,339 | 0.05 % | 2,633 | 0.08 % |
| unicode_bidi | 13 | 5,844 | 0.05 % | 2,363 | 0.07 % |
| wasm_bindgen | 47 | 5,274 | 0.05 % | 1,929 | 0.06 % |
| glamx | 5 | 3,438 | 0.03 % | 1,652 | 0.05 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,354 | 0.04 % |
| parking_lot | 6 | 3,837 | 0.03 % | 1,228 | 0.04 % |
| serde_core | 23 | 3,048 | 0.03 % | 1,161 | 0.03 % |
| zmij | 1 | 2,148 | 0.02 % | 1,103 | 0.03 % |
| parking_lot_core | 9 | 1,613 | 0.01 % | 830 | 0.02 % |
| half | 21 | 1,831 | 0.02 % | 826 | 0.02 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 815 | 0.02 % |
| foldhash | 4 | 3,050 | 0.03 % | 654 | 0.02 % |
| once_cell | 5 | 1,081 | 0.01 % | 630 | 0.02 % |
| ena | 4 | 1,627 | 0.01 % | 596 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 516 | 0.01 % |
| bitflags | 9 | 1,410 | 0.01 % | 478 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 420 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 408 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 405 | 0.01 % |
| bit_vec | 8 | 628 | 0.01 % | 397 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 390 | 0.01 % |
| bit_set | 5 | 486 | 0.00 % | 349 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 323 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 261 | 0.01 % |
| lock_api | 2 | 310 | 0.00 % | 234 | 0.01 % |
| __rustc | 5 | 325 | 0.00 % | 216 | 0.01 % |
| bytemuck | 5 | 324 | 0.00 % | 201 | 0.01 % |
| log | 3 | 321 | 0.00 % | 190 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 158 | 0.00 % |
| simd_adler32 | 1 | 201 | 0.00 % | 140 | 0.00 % |
| serde_arrays | 1 | 64 | 0.00 % | 68 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 457,794 | 153,706 | og_render | `<og_render::Renderer>::render` |
| 363,209 | 123,584 | og_physics | `og_physics::schedule::{closure#0}` |
| 256,696 | 62,557 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 194,853 | 72,675 | og_runtime | `og_runtime::extract::extract` |
| 125,491 | 35,312 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 99,398 | 33,320 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 97,562 | 36,069 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 87,743 | 29,426 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 74,310 | 28,230 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 67,925 | 24,386 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 54,681 | 21,687 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 50,920 | 17,908 | og_anim | `og_anim::system::install::{closure#0}` |
| 50,134 | 17,429 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 48,476 | 9,014 | og_physics | `og_physics::gather::gather` |
| 41,387 | 16,576 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 38,265 | 11,140 | og_anim | `og_anim::rig2d::parse_body` |
| 38,124 | 7,201 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 36,142 | 12,716 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 35,608 | 13,900 | og_runtime | `og_runtime::timeline::decode` |
| 33,324 | 6,309 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 32,913 | 10,480 | og_host_web (exports) | `og_room_deliver` |
| 31,139 | 11,953 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,945 | 10,181 | og_runtime | `og_runtime::engine::call_rpc` |
| 30,730 | 11,171 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 28,269 | 11,256 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,088 | 8,968 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,910 | 9,706 | naga | `<naga::valid::Validator>::validate_function` |
| 26,680 | 6,790 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 26,212 | 10,836 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 25,935 | 9,988 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,439 | 9,611 | og_runtime | `og_runtime::anim::load_animation` |
| 24,517 | 9,465 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 24,457 | 3,877 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 24,414 | 9,420 | og_audio | `og_audio::system::run` |
| 23,982 | 7,859 | og_runtime | `og_runtime::modules::load_style` |
| 23,736 | 5,222 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,695 | 8,954 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 22,902 | 8,618 | og_render | `og_render::id_pass::render_ids_from` |
| 22,875 | 9,122 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,833 | 8,209 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 22,731 | 8,626 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,976 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,657 | 9,486 | og_nav | `og_nav::system::run` |
| 22,317 | 9,394 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,931 | 9,286 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,209 | 7,586 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 20,523 | 9,050 | og_runtime | `og_runtime::stream::fetch` |
| 20,445 | 7,347 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,120 | 7,892 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,083 | 7,399 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 20,049 | 8,624 | og_runtime | `og_runtime::assets::add_gltf` |
| 19,799 | 8,099 | og_physics | `og_physics::dim3::sim::shape` |
| 19,777 | 8,145 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,583 | 6,155 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,258 | 8,274 | og_host_web (exports) | `og_load_rig2d multivalue shim` |
| 19,069 | 7,859 | og_ui | `og_ui::text::bidi_runs` |
| 18,920 | 6,419 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher>::contact_manifold_convex_co...` |
| 18,844 | 6,489 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher<rapier3d::geometry::contact_...` |
| 18,545 | 5,941 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 18,539 | 6,326 | parry3d | `<core::marker::PhantomData<parry3d::shape::shared_shape::SharedShape> as serde_core::de::DeserializeSeed>::deserialize::<&mut bincode::de::Deserializer<binco...` |

twiggy top, per crate (shallow bytes; names and producers left out): 11,799,404 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3595 | 1,397,217 | 11.84 % |
| og_host_web (exports) | 382 | 1,182,076 | 10.02 % |
| og_runtime | 1012 | 1,171,528 | 9.93 % |
| og_render | 664 | 859,730 | 7.29 % |
| rapier3d | 351 | 843,908 | 7.15 % |
| parry3d | 982 | 759,163 | 6.43 % |
| og_physics | 280 | 748,638 | 6.34 % |
| naga | 1251 | 627,924 | 5.32 % |
| std | 1569 | 426,197 | 3.61 % |
| wgpu | 376 | 361,292 | 3.06 % |
| og_anim | 326 | 352,112 | 2.98 % |
| wgpu_core | 945 | 324,684 | 2.75 % |
| og_ui | 269 | 315,427 | 2.67 % |
| og_ecs | 473 | 184,588 | 1.56 % |
| og_core | 536 | 171,046 | 1.45 % |
| bincode | 78 | 168,156 | 1.43 % |
| wgpu_hal | 353 | 167,923 | 1.42 % |
| og_nav | 93 | 149,624 | 1.27 % |
| ttf_parser | 291 | 130,509 | 1.11 % |
| rustybuzz | 294 | 109,609 | 0.93 % |
| [unnamed] | 380 | 99,060 | 0.84 % |
| og_cook | 289 | 98,483 | 0.83 % |
| og_ai | 74 | 90,046 | 0.76 % |
| og_audio | 116 | 87,870 | 0.74 % |
| serde_json | 254 | 82,785 | 0.70 % |
| og_probe | 94 | 73,980 | 0.63 % |
| js_sys | 64 | 72,997 | 0.62 % |
| [wasm structure] | 1453 | 62,734 | 0.53 % |
| og_input | 69 | 56,191 | 0.48 % |
| og_host_web | 77 | 46,659 | 0.40 % |
| indexmap | 119 | 45,313 | 0.38 % |
| og_geom | 59 | 42,557 | 0.36 % |
| og_lanes | 80 | 35,000 | 0.30 % |
| glam | 83 | 32,737 | 0.28 % |
| og_net | 61 | 31,392 | 0.27 % |
| wgpu_types | 285 | 30,350 | 0.26 % |
| og_tilemap | 51 | 29,920 | 0.25 % |
| og_tween | 65 | 29,307 | 0.25 % |
| robust | 7 | 28,621 | 0.24 % |
| og_i18n | 79 | 25,915 | 0.22 % |
| spade | 44 | 23,615 | 0.20 % |
| codespan_reporting | 56 | 21,316 | 0.18 % |
| miniz_oxide | 30 | 20,121 | 0.17 % |
| [other] | 89 | 19,909 | 0.17 % |
| compiler_builtins | 19 | 19,349 | 0.16 % |
| og_timeline | 42 | 14,125 | 0.12 % |
| og_blob | 29 | 13,865 | 0.12 % |
| dlmalloc | 15 | 12,530 | 0.11 % |
| glow | 79 | 10,625 | 0.09 % |
| smallvec | 59 | 10,419 | 0.09 % |
| nalgebra | 14 | 9,249 | 0.08 % |
| libm | 13 | 9,030 | 0.08 % |
| naga_types | 57 | 6,809 | 0.06 % |
| arrayvec | 93 | 6,535 | 0.06 % |
| unicode_bidi | 13 | 5,875 | 0.05 % |
| wasm_bindgen | 47 | 5,380 | 0.05 % |
| parking_lot | 6 | 3,855 | 0.03 % |
| rustc_hash | 27 | 3,605 | 0.03 % |
| glamx | 5 | 3,453 | 0.03 % |
| serde_core | 23 | 3,101 | 0.03 % |
| foldhash | 4 | 3,062 | 0.03 % |
| zmij | 1 | 2,152 | 0.02 % |
| half | 21 | 1,880 | 0.02 % |
| ena | 4 | 1,638 | 0.01 % |
| parking_lot_core | 9 | 1,634 | 0.01 % |
| unicode_segmentation | 8 | 1,600 | 0.01 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,401 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| once_cell | 5 | 1,094 | 0.01 % |
| web_sys | 27 | 1,034 | 0.01 % |
| fdeflate | 1 | 829 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_vec | 8 | 645 | 0.01 % |
| bit_set | 5 | 498 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| __rustc | 5 | 336 | 0.00 % |
| bytemuck | 5 | 334 | 0.00 % |
| log | 3 | 329 | 0.00 % |
| lock_api | 2 | 315 | 0.00 % |
| simd_adler32 | 1 | 204 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 66 | 0.00 % |
