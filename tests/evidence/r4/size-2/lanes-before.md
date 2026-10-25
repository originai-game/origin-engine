# lanes, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes.wasm: 19968897 B raw, 13489824 B after wasm-opt, 3591631 B brotli -q 11 (85.6 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 13,387,468 B raw; "brotli alone" is each group compressed by itself (they sum to 3,805,761 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 685 | 1,473,580 | 11.01 % | 437,836 | 11.50 % |
| [data] | 1 | 1,596,526 | 11.93 % | 364,563 | 9.58 % |
| og_host_web (exports) | 363 | 1,247,206 | 9.32 % | 341,412 | 8.97 % |
| rapier3d | 374 | 1,028,195 | 7.68 % | 338,035 | 8.88 % |
| og_render | 666 | 1,073,130 | 8.02 % | 315,094 | 8.28 % |
| parry3d | 998 | 788,500 | 5.89 % | 208,846 | 5.49 % |
| naga | 1,325 | 649,585 | 4.85 % | 192,882 | 5.07 % |
| og_physics | 272 | 598,591 | 4.47 % | 159,319 | 4.19 % |
| wgpu | 360 | 413,604 | 3.09 % | 142,870 | 3.75 % |
| og_ui | 238 | 336,883 | 2.52 % | 110,604 | 2.91 % |
| std | 1,146 | 441,657 | 3.30 % | 100,261 | 2.63 % |
| wgpu_core | 759 | 363,654 | 2.72 % | 95,257 | 2.50 % |
| og_anim | 246 | 282,381 | 2.11 % | 85,776 | 2.25 % |
| wgpu_hal | 310 | 188,816 | 1.41 % | 65,439 | 1.72 % |
| og_ecs | 419 | 223,247 | 1.67 % | 59,358 | 1.56 % |
| og_nav | 84 | 150,515 | 1.12 % | 50,374 | 1.32 % |
| bincode | 89 | 169,037 | 1.26 % | 46,172 | 1.21 % |
| ttf_parser | 294 | 129,835 | 0.97 % | 42,766 | 1.12 % |
| [unnamed] | 398 | 158,122 | 1.18 % | 41,956 | 1.10 % |
| og_cook | 277 | 127,807 | 0.95 % | 40,691 | 1.07 % |
| rustybuzz | 295 | 109,073 | 0.81 % | 40,623 | 1.07 % |
| serde_json | 201 | 196,195 | 1.47 % | 39,351 | 1.03 % |
| og_audio | 85 | 96,555 | 0.72 % | 34,677 | 0.91 % |
| og_probe | 50 | 97,074 | 0.73 % | 33,769 | 0.89 % |
| og_core | 355 | 157,575 | 1.18 % | 33,160 | 0.87 % |
| rayon | 36 | 91,804 | 0.69 % | 29,040 | 0.76 % |
| og_ai | 64 | 79,622 | 0.59 % | 27,286 | 0.72 % |
| js_sys | 70 | 79,819 | 0.60 % | 27,069 | 0.71 % |
| og_geom | 67 | 86,217 | 0.64 % | 26,241 | 0.69 % |
| og_host_web | 77 | 100,146 | 0.75 % | 25,654 | 0.67 % |
| rayon_core | 271 | 139,149 | 1.04 % | 21,063 | 0.55 % |
| og_tilemap | 28 | 46,802 | 0.35 % | 16,094 | 0.42 % |
| og_timeline | 54 | 71,930 | 0.54 % | 16,025 | 0.42 % |
| og_lanes | 88 | 51,039 | 0.38 % | 15,552 | 0.41 % |
| og_net | 41 | 39,665 | 0.30 % | 13,233 | 0.35 % |
| og_input | 36 | 48,384 | 0.36 % | 13,196 | 0.35 % |
| og_i18n | 69 | 32,789 | 0.24 % | 12,732 | 0.33 % |
| og_tween | 37 | 34,606 | 0.26 % | 11,642 | 0.31 % |
| miniz_oxide | 31 | 25,948 | 0.19 % | 11,333 | 0.30 % |
| spade | 22 | 39,375 | 0.29 % | 10,971 | 0.29 % |
| indexmap | 112 | 48,143 | 0.36 % | 10,232 | 0.27 % |
| glam | 45 | 29,420 | 0.22 % | 9,926 | 0.26 % |
| [other] | 84 | 24,876 | 0.19 % | 9,389 | 0.25 % |
| wgpu_types | 240 | 28,502 | 0.21 % | 9,230 | 0.24 % |
| codespan_reporting | 57 | 21,321 | 0.16 % | 8,454 | 0.22 % |
| compiler_builtins | 18 | 17,568 | 0.13 % | 6,969 | 0.18 % |
| robust | 4 | 14,548 | 0.11 % | 4,247 | 0.11 % |
| nalgebra | 13 | 8,923 | 0.07 % | 3,908 | 0.10 % |
| libm | 11 | 9,166 | 0.07 % | 3,327 | 0.09 % |
| naga_types | 49 | 7,933 | 0.06 % | 3,282 | 0.09 % |
| og_blob | 27 | 13,382 | 0.10 % | 3,119 | 0.08 % |
| glow | 76 | 11,429 | 0.09 % | 3,020 | 0.08 % |
| smallvec | 47 | 9,850 | 0.07 % | 2,673 | 0.07 % |
| dlmalloc | 7 | 6,557 | 0.05 % | 2,289 | 0.06 % |
| arrayvec | 77 | 4,946 | 0.04 % | 2,239 | 0.06 % |
| serde_core | 25 | 6,450 | 0.05 % | 2,102 | 0.06 % |
| unicode_bidi | 7 | 4,753 | 0.04 % | 2,046 | 0.05 % |
| crossbeam_channel | 17 | 7,487 | 0.06 % | 1,997 | 0.05 % |
| crossbeam_epoch | 17 | 4,257 | 0.03 % | 1,700 | 0.04 % |
| rustc_hash | 24 | 4,202 | 0.03 % | 1,570 | 0.04 % |
| wasm_bindgen | 53 | 4,449 | 0.03 % | 1,400 | 0.04 % |
| parking_lot | 6 | 4,040 | 0.03 % | 1,280 | 0.03 % |
| glamx | 4 | 2,191 | 0.02 % | 1,131 | 0.03 % |
| crossbeam_deque | 8 | 4,809 | 0.04 % | 1,068 | 0.03 % |
| zmij | 1 | 1,753 | 0.01 % | 981 | 0.03 % |
| parking_lot_core | 10 | 1,815 | 0.01 % | 976 | 0.03 % |
| half | 21 | 1,842 | 0.01 % | 863 | 0.02 % |
| unicode_segmentation | 5 | 1,367 | 0.01 % | 700 | 0.02 % |
| bit_set | 6 | 1,203 | 0.01 % | 696 | 0.02 % |
| foldhash | 5 | 3,224 | 0.02 % | 694 | 0.02 % |
| __rustc | 5 | 1,492 | 0.01 % | 660 | 0.02 % |
| web_sys | 99 | 2,137 | 0.02 % | 631 | 0.02 % |
| ena | 4 | 1,635 | 0.01 % | 595 | 0.02 % |
| wasm_bindgen_rayon | 2 | 1,121 | 0.01 % | 493 | 0.01 % |
| once_cell | 4 | 769 | 0.01 % | 469 | 0.01 % |
| wgpu_naga_bridge | 2 | 731 | 0.01 % | 426 | 0.01 % |
| raw_window_handle | 18 | 1,148 | 0.01 % | 411 | 0.01 % |
| bit_vec | 6 | 613 | 0.00 % | 388 | 0.01 % |
| itoa | 1 | 554 | 0.00 % | 278 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 261 | 0.01 % |
| lock_api | 2 | 346 | 0.00 % | 254 | 0.01 % |
| slotmap | 2 | 356 | 0.00 % | 252 | 0.01 % |
| bitflags | 4 | 402 | 0.00 % | 192 | 0.01 % |
| bytemuck | 5 | 197 | 0.00 % | 182 | 0.00 % |
| unicode_properties | 2 | 171 | 0.00 % | 157 | 0.00 % |
| log | 2 | 193 | 0.00 % | 157 | 0.00 % |
| wasm_sync | 1 | 85 | 0.00 % | 86 | 0.00 % |
| crossbeam_utils | 1 | 79 | 0.00 % | 76 | 0.00 % |
| serde_arrays | 1 | 59 | 0.00 % | 63 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 453,432 | 154,925 | og_render | `<og_render::Renderer>::render` |
| 249,648 | 81,337 | og_runtime | `og_runtime::extract::extract` |
| 248,585 | 65,367 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 173,519 | 62,715 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 123,986 | 44,542 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 123,350 | 34,824 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 117,026 | 39,874 | og_physics | `og_physics::dim3::sim::step_here` |
| 100,007 | 33,731 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 89,244 | 30,064 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 72,882 | 14,732 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 71,768 | 24,171 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 55,572 | 21,993 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 55,566 | 15,257 | og_host_web (exports) | `og_room_deliver` |
| 55,493 | 18,092 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,248 | 18,477 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 49,222 | 9,219 | og_physics | `og_physics::gather::gather` |
| 47,328 | 16,360 | og_runtime | `og_runtime::timeline::decode` |
| 41,809 | 6,289 | og_host_web | `core::ptr::drop_glue::<og_host_web::canvas::CanvasBackend>` |
| 40,674 | 15,354 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 37,077 | 11,011 | og_anim | `og_anim::rig2d::parse_body` |
| 36,127 | 12,943 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 31,430 | 12,154 | naga | `<naga::valid::Validator>::validate_impl` |
| 31,121 | 12,542 | og_runtime | `og_runtime::assets::spawn_scene` |
| 30,466 | 11,975 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 30,091 | 7,003 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 30,061 | 10,979 | og_runtime | `og_runtime::anim::load_animation` |
| 29,498 | 7,184 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 29,199 | 12,397 | rapier3d | `<rapier3d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 28,692 | 11,058 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 28,097 | 9,013 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 28,012 | 9,736 | naga | `<naga::valid::Validator>::validate_function` |
| 27,499 | 10,803 | og_runtime | `<og_runtime::describe::RuntimeView as og_probe::hooks::SceneView>::pick` |
| 27,422 | 10,555 | og_runtime | `og_runtime::stream::fetch` |
| 27,235 | 10,292 | og_audio | `og_audio::system::run` |
| 26,234 | 10,143 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,580 | 10,532 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier3d::dynamics::solver:...` |
| 25,465 | 10,036 | og_probe | `<og_probe::probe::Probe>::execute` |
| 25,094 | 8,694 | og_runtime | `og_runtime::engine::answer_call` |
| 25,068 | 10,096 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 24,457 | 3,866 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 24,332 | 7,638 | og_runtime | `og_runtime::modules::load_style` |
| 23,984 | 5,295 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,704 | 8,857 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 23,004 | 8,911 | og_runtime | `og_runtime::assets::add_gltf` |
| 22,735 | 8,615 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,976 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,592 | 8,590 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,538 | 8,402 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 22,514 | 9,651 | og_nav | `og_nav::system::run` |
| 22,059 | 8,545 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 21,209 | 7,508 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 20,445 | 7,353 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,307 | 7,069 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 20,102 | 7,885 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,058 | 7,382 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,894 | 8,065 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 19,877 | 8,064 | og_physics | `og_physics::dim3::sim::shape` |
| 19,700 | 6,220 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,579 | 6,512 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher>::contact_manifold_convex_co...` |
| 19,490 | 6,934 | og_physics | `og_physics::schedule::{closure#0}` |

twiggy top, per crate (shallow bytes; names and producers left out): 13,489,554 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,596,525 | 11.84 % |
| og_runtime | 685 | 1,475,521 | 10.94 % |
| og_host_web (exports) | 363 | 1,248,300 | 9.25 % |
| og_render | 666 | 1,074,995 | 7.97 % |
| rapier3d | 374 | 1,029,326 | 7.63 % |
| parry3d | 998 | 791,193 | 5.87 % |
| naga | 1325 | 652,756 | 4.84 % |
| og_physics | 272 | 599,342 | 4.44 % |
| std | 1146 | 444,463 | 3.29 % |
| wgpu | 360 | 414,512 | 3.07 % |
| wgpu_core | 759 | 365,585 | 2.71 % |
| og_ui | 238 | 337,572 | 2.50 % |
| og_anim | 246 | 283,065 | 2.10 % |
| og_ecs | 419 | 224,458 | 1.66 % |
| serde_json | 201 | 196,751 | 1.46 % |
| wgpu_hal | 310 | 189,570 | 1.41 % |
| bincode | 89 | 169,292 | 1.25 % |
| [unnamed] | 398 | 159,235 | 1.18 % |
| og_core | 355 | 158,471 | 1.17 % |
| og_nav | 84 | 150,754 | 1.12 % |
| rayon_core | 271 | 139,928 | 1.04 % |
| ttf_parser | 294 | 130,593 | 0.97 % |
| og_cook | 277 | 128,476 | 0.95 % |
| rustybuzz | 295 | 109,801 | 0.81 % |
| og_host_web | 77 | 100,364 | 0.74 % |
| og_probe | 50 | 97,224 | 0.72 % |
| og_audio | 85 | 96,783 | 0.72 % |
| rayon | 36 | 91,910 | 0.68 % |
| og_geom | 67 | 86,415 | 0.64 % |
| js_sys | 70 | 79,976 | 0.59 % |
| og_ai | 64 | 79,797 | 0.59 % |
| og_timeline | 54 | 72,089 | 0.53 % |
| [wasm structure] | 1489 | 69,126 | 0.51 % |
| og_lanes | 88 | 51,292 | 0.38 % |
| og_input | 36 | 48,475 | 0.36 % |
| indexmap | 112 | 48,420 | 0.36 % |
| og_tilemap | 28 | 46,880 | 0.35 % |
| og_net | 41 | 39,787 | 0.29 % |
| spade | 22 | 39,444 | 0.29 % |
| og_tween | 37 | 34,708 | 0.26 % |
| og_i18n | 69 | 32,970 | 0.24 % |
| glam | 45 | 29,543 | 0.22 % |
| wgpu_types | 240 | 29,022 | 0.22 % |
| miniz_oxide | 31 | 26,023 | 0.19 % |
| [other] | 89 | 25,123 | 0.19 % |
| codespan_reporting | 57 | 21,465 | 0.16 % |
| compiler_builtins | 18 | 17,624 | 0.13 % |
| robust | 4 | 14,563 | 0.11 % |
| og_blob | 27 | 13,452 | 0.10 % |
| glow | 76 | 11,616 | 0.09 % |
| smallvec | 47 | 9,972 | 0.07 % |
| libm | 11 | 9,197 | 0.07 % |
| nalgebra | 13 | 8,960 | 0.07 % |
| naga_types | 49 | 8,038 | 0.06 % |
| crossbeam_channel | 17 | 7,533 | 0.06 % |
| dlmalloc | 7 | 6,576 | 0.05 % |
| serde_core | 25 | 6,513 | 0.05 % |
| arrayvec | 77 | 5,104 | 0.04 % |
| crossbeam_deque | 8 | 4,833 | 0.04 % |
| unicode_bidi | 7 | 4,771 | 0.04 % |
| wasm_bindgen | 53 | 4,568 | 0.03 % |
| crossbeam_epoch | 17 | 4,300 | 0.03 % |
| rustc_hash | 24 | 4,256 | 0.03 % |
| parking_lot | 6 | 4,058 | 0.03 % |
| foldhash | 5 | 3,240 | 0.02 % |
| web_sys | 99 | 2,335 | 0.02 % |
| glamx | 4 | 2,203 | 0.02 % |
| half | 21 | 1,891 | 0.01 % |
| parking_lot_core | 10 | 1,839 | 0.01 % |
| zmij | 1 | 1,757 | 0.01 % |
| ena | 4 | 1,646 | 0.01 % |
| __rustc | 5 | 1,506 | 0.01 % |
| unicode_segmentation | 5 | 1,380 | 0.01 % |
| bit_set | 6 | 1,219 | 0.01 % |
| raw_window_handle | 18 | 1,185 | 0.01 % |
| wasm_bindgen_rayon | 2 | 1,126 | 0.01 % |
| once_cell | 4 | 778 | 0.01 % |
| wgpu_naga_bridge | 2 | 738 | 0.01 % |
| bit_vec | 6 | 626 | 0.00 % |
| itoa | 1 | 558 | 0.00 % |
| bitflags | 4 | 412 | 0.00 % |
| slotmap | 2 | 361 | 0.00 % |
| lock_api | 2 | 351 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| bytemuck | 5 | 207 | 0.00 % |
| log | 2 | 198 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| wasm_sync | 1 | 87 | 0.00 % |
| crossbeam_utils | 1 | 81 | 0.00 % |
| serde_arrays | 1 | 61 | 0.00 % |
