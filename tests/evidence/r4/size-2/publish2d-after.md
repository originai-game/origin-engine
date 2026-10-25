# publish2d, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish2d.wasm: 17795127 B raw, 10700674 B after wasm-opt, 3034243 B brotli -q 11 (72.3 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 10,604,270 B raw; "brotli alone" is each group compressed by itself (they sum to 3,185,137 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 940 | 1,084,463 | 10.23 % | 341,730 | 10.73 % |
| [data] | 1 | 1,318,820 | 12.44 % | 325,623 | 10.22 % |
| og_host_web (exports) | 381 | 1,107,703 | 10.45 % | 306,171 | 9.61 % |
| og_render | 598 | 729,705 | 6.88 % | 235,845 | 7.40 % |
| og_physics | 280 | 729,529 | 6.88 % | 212,408 | 6.67 % |
| naga | 1,251 | 624,896 | 5.89 % | 186,585 | 5.86 % |
| rapier2d | 320 | 578,338 | 5.45 % | 186,318 | 5.85 % |
| parry2d | 780 | 446,904 | 4.21 % | 128,484 | 4.03 % |
| wgpu | 374 | 360,118 | 3.40 % | 128,195 | 4.02 % |
| og_ui | 255 | 306,852 | 2.89 % | 104,896 | 3.29 % |
| std | 1,514 | 406,706 | 3.84 % | 103,585 | 3.25 % |
| wgpu_core | 944 | 322,318 | 3.04 % | 91,850 | 2.88 % |
| og_anim | 288 | 275,216 | 2.60 % | 82,581 | 2.59 % |
| wgpu_hal | 353 | 167,090 | 1.58 % | 61,369 | 1.93 % |
| og_nav | 91 | 149,272 | 1.41 % | 50,090 | 1.57 % |
| og_ecs | 463 | 181,280 | 1.71 % | 49,571 | 1.56 % |
| bincode | 71 | 173,452 | 1.64 % | 48,194 | 1.51 % |
| ttf_parser | 292 | 129,768 | 1.22 % | 42,770 | 1.34 % |
| rustybuzz | 294 | 108,884 | 1.03 % | 40,496 | 1.27 % |
| og_core | 521 | 156,986 | 1.48 % | 34,182 | 1.07 % |
| og_ai | 76 | 89,918 | 0.85 % | 30,056 | 0.94 % |
| og_cook | 266 | 93,659 | 0.88 % | 29,098 | 0.91 % |
| [unnamed] | 375 | 94,887 | 0.89 % | 28,361 | 0.89 % |
| og_audio | 104 | 74,063 | 0.70 % | 27,783 | 0.87 % |
| og_probe | 92 | 73,077 | 0.69 % | 26,873 | 0.84 % |
| serde_json | 253 | 82,417 | 0.78 % | 26,845 | 0.84 % |
| js_sys | 63 | 73,819 | 0.70 % | 26,587 | 0.83 % |
| og_host_web | 77 | 45,820 | 0.43 % | 16,840 | 0.53 % |
| og_geom | 59 | 42,381 | 0.40 % | 16,464 | 0.52 % |
| og_input | 70 | 56,145 | 0.53 % | 12,621 | 0.40 % |
| og_tilemap | 51 | 29,788 | 0.28 % | 12,236 | 0.38 % |
| glam | 77 | 31,981 | 0.30 % | 11,824 | 0.37 % |
| miniz_oxide | 31 | 26,019 | 0.25 % | 11,418 | 0.36 % |
| og_tween | 65 | 29,136 | 0.27 % | 10,907 | 0.34 % |
| og_net | 61 | 31,229 | 0.29 % | 10,891 | 0.34 % |
| wgpu_types | 284 | 29,659 | 0.28 % | 10,389 | 0.33 % |
| og_i18n | 79 | 25,714 | 0.24 % | 10,373 | 0.33 % |
| indexmap | 115 | 42,671 | 0.40 % | 9,820 | 0.31 % |
| codespan_reporting | 57 | 21,178 | 0.20 % | 8,428 | 0.26 % |
| spade | 42 | 23,633 | 0.22 % | 8,101 | 0.25 % |
| compiler_builtins | 19 | 19,291 | 0.18 % | 7,680 | 0.24 % |
| robust | 7 | 28,597 | 0.27 % | 7,672 | 0.24 % |
| og_lanes | 62 | 29,410 | 0.28 % | 7,391 | 0.23 % |
| [other] | 83 | 19,211 | 0.18 % | 7,174 | 0.23 % |
| og_timeline | 42 | 14,013 | 0.13 % | 5,837 | 0.18 % |
| dlmalloc | 15 | 12,487 | 0.12 % | 3,681 | 0.12 % |
| libm | 13 | 8,992 | 0.08 % | 3,329 | 0.10 % |
| naga_types | 58 | 6,851 | 0.06 % | 3,245 | 0.10 % |
| og_blob | 27 | 13,826 | 0.13 % | 3,171 | 0.10 % |
| arrayvec | 98 | 7,138 | 0.07 % | 2,957 | 0.09 % |
| smallvec | 59 | 10,276 | 0.10 % | 2,957 | 0.09 % |
| glow | 79 | 10,433 | 0.10 % | 2,848 | 0.09 % |
| nalgebra | 9 | 5,426 | 0.05 % | 2,388 | 0.07 % |
| unicode_bidi | 14 | 5,857 | 0.06 % | 2,367 | 0.07 % |
| wasm_bindgen | 47 | 5,274 | 0.05 % | 1,925 | 0.06 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,347 | 0.04 % |
| parking_lot | 6 | 3,837 | 0.04 % | 1,235 | 0.04 % |
| serde_core | 23 | 3,048 | 0.03 % | 1,167 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,103 | 0.03 % |
| parking_lot_core | 9 | 1,613 | 0.02 % | 837 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 821 | 0.03 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 814 | 0.03 % |
| foldhash | 4 | 3,050 | 0.03 % | 657 | 0.02 % |
| once_cell | 5 | 1,081 | 0.01 % | 627 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 517 | 0.02 % |
| glamx | 3 | 1,100 | 0.01 % | 512 | 0.02 % |
| bitflags | 9 | 1,410 | 0.01 % | 480 | 0.02 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 422 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 406 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 401 | 0.01 % |
| bit_vec | 8 | 628 | 0.01 % | 398 | 0.01 % |
| bit_set | 5 | 486 | 0.00 % | 347 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 328 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 259 | 0.01 % |
| lock_api | 2 | 310 | 0.00 % | 235 | 0.01 % |
| __rustc | 5 | 325 | 0.00 % | 209 | 0.01 % |
| log | 3 | 321 | 0.00 % | 188 | 0.01 % |
| bytemuck | 5 | 174 | 0.00 % | 172 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 152 | 0.00 % |
| serde_arrays | 1 | 19 | 0.00 % | 23 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 407,976 | 137,800 | og_render | `<og_render::Renderer>::render` |
| 360,640 | 122,390 | og_physics | `og_physics::dim2::sim::step_here` |
| 237,769 | 58,820 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 163,595 | 61,144 | og_runtime | `og_runtime::extract::extract` |
| 118,745 | 33,721 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 97,563 | 36,134 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 83,595 | 27,781 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 68,910 | 24,658 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 65,114 | 24,803 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 55,400 | 21,913 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 50,134 | 17,406 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 48,476 | 8,986 | og_physics | `og_physics::gather::gather` |
| 41,387 | 16,561 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 38,261 | 11,141 | og_anim | `og_anim::rig2d::parse_body` |
| 38,124 | 7,244 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 37,606 | 12,720 | og_runtime | `og_runtime::engine::call_rpc` |
| 36,026 | 14,173 | og_runtime | `og_runtime::timeline::decode` |
| 32,913 | 10,455 | og_host_web (exports) | `og_room_deliver` |
| 31,305 | 6,485 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 31,139 | 11,960 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,730 | 11,155 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,369 | 10,323 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 28,269 | 11,268 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,088 | 9,015 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,910 | 9,704 | naga | `<naga::valid::Validator>::validate_function` |
| 26,680 | 6,814 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 25,994 | 10,019 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 24,457 | 3,921 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 23,982 | 7,851 | og_runtime | `og_runtime::modules::load_style` |
| 23,736 | 5,226 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,893 | 9,139 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,731 | 8,666 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,976 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,655 | 9,496 | og_nav | `og_nav::system::run` |
| 22,573 | 8,548 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,367 | 8,367 | og_render | `og_render::id_pass::render_ids_from` |
| 22,317 | 9,393 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,931 | 9,276 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,455 | 8,526 | og_audio | `og_audio::system::install::{closure#0}` |
| 21,111 | 7,815 | og_runtime | `og_runtime::anim::load_animation` |
| 20,520 | 9,046 | og_runtime | `og_runtime::stream::fetch` |
| 20,120 | 7,899 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,083 | 7,406 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,794 | 7,867 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 19,771 | 8,150 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,583 | 6,157 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,388 | 6,294 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 19,273 | 7,844 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 19,069 | 7,855 | og_ui | `og_ui::text::bidi_runs` |
| 18,794 | 6,065 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,584 | 7,810 | rapier2d | `<rapier2d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |
| 18,032 | 6,406 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,906 | 7,398 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 17,783 | 6,329 | og_physics | `og_physics::schedule::{closure#0}` |
| 17,517 | 7,216 | og_runtime | `og_runtime::save::apply_keeping` |
| 17,422 | 6,500 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 16,649 | 6,483 | rapier2d | `<rapier2d::pipeline::physics_world::PhysicsWorld>::remove_body` |
| 16,349 | 6,911 | wgpu | `<wgpu::backend::wgpu_core::CoreAdapter as wgpu::dispatch::AdapterInterface>::request_device` |
| 16,284 | 6,802 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_shader_module` |
| 16,261 | 5,879 | bincode | `<&mut bincode::de::Deserializer<bincode::de::read::SliceReader, bincode::config::WithOtherLimit<bincode::config::WithOtherTrailing<bincode::config::DefaultOp...` |

twiggy top, per crate (shallow bytes; names and producers left out): 10,700,397 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3450 | 1,318,818 | 12.32 % |
| og_host_web (exports) | 381 | 1,108,852 | 10.36 % |
| og_runtime | 940 | 1,087,012 | 10.16 % |
| og_render | 598 | 731,295 | 6.83 % |
| og_physics | 280 | 730,289 | 6.82 % |
| naga | 1251 | 627,906 | 5.87 % |
| rapier2d | 320 | 579,302 | 5.41 % |
| parry2d | 780 | 448,950 | 4.20 % |
| std | 1514 | 410,466 | 3.84 % |
| wgpu | 374 | 361,049 | 3.37 % |
| wgpu_core | 944 | 324,667 | 3.03 % |
| og_ui | 255 | 307,572 | 2.87 % |
| og_anim | 288 | 276,006 | 2.58 % |
| og_ecs | 463 | 182,581 | 1.71 % |
| bincode | 71 | 173,658 | 1.62 % |
| wgpu_hal | 353 | 167,926 | 1.57 % |
| og_core | 521 | 158,287 | 1.48 % |
| og_nav | 91 | 149,529 | 1.40 % |
| ttf_parser | 292 | 130,522 | 1.22 % |
| rustybuzz | 294 | 109,610 | 1.02 % |
| [unnamed] | 375 | 95,869 | 0.90 % |
| og_cook | 266 | 94,268 | 0.88 % |
| og_ai | 76 | 90,127 | 0.84 % |
| serde_json | 253 | 83,072 | 0.78 % |
| og_audio | 104 | 74,337 | 0.69 % |
| js_sys | 63 | 73,956 | 0.69 % |
| og_probe | 92 | 73,324 | 0.69 % |
| [wasm structure] | 1459 | 62,296 | 0.58 % |
| og_input | 70 | 56,301 | 0.53 % |
| og_host_web | 77 | 46,021 | 0.43 % |
| indexmap | 115 | 42,949 | 0.40 % |
| og_geom | 59 | 42,558 | 0.40 % |
| glam | 77 | 32,187 | 0.30 % |
| og_net | 61 | 31,392 | 0.29 % |
| wgpu_types | 284 | 30,283 | 0.28 % |
| og_tilemap | 51 | 29,920 | 0.28 % |
| og_lanes | 62 | 29,588 | 0.28 % |
| og_tween | 65 | 29,308 | 0.27 % |
| robust | 7 | 28,622 | 0.27 % |
| miniz_oxide | 31 | 26,094 | 0.24 % |
| og_i18n | 79 | 25,915 | 0.24 % |
| spade | 42 | 23,744 | 0.22 % |
| codespan_reporting | 57 | 21,322 | 0.20 % |
| [other] | 88 | 19,452 | 0.18 % |
| compiler_builtins | 19 | 19,349 | 0.18 % |
| og_timeline | 42 | 14,125 | 0.13 % |
| og_blob | 27 | 13,897 | 0.13 % |
| dlmalloc | 15 | 12,530 | 0.12 % |
| glow | 79 | 10,625 | 0.10 % |
| smallvec | 59 | 10,419 | 0.10 % |
| libm | 13 | 9,030 | 0.08 % |
| arrayvec | 98 | 7,346 | 0.07 % |
| naga_types | 58 | 6,979 | 0.07 % |
| unicode_bidi | 14 | 5,890 | 0.06 % |
| nalgebra | 9 | 5,451 | 0.05 % |
| wasm_bindgen | 47 | 5,381 | 0.05 % |
| parking_lot | 6 | 3,855 | 0.04 % |
| rustc_hash | 27 | 3,605 | 0.03 % |
| serde_core | 23 | 3,101 | 0.03 % |
| foldhash | 4 | 3,062 | 0.03 % |
| zmij | 1 | 2,152 | 0.02 % |
| half | 21 | 1,880 | 0.02 % |
| parking_lot_core | 9 | 1,634 | 0.02 % |
| unicode_segmentation | 8 | 1,600 | 0.01 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,401 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| glamx | 3 | 1,108 | 0.01 % |
| once_cell | 5 | 1,094 | 0.01 % |
| web_sys | 27 | 1,034 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_vec | 8 | 645 | 0.01 % |
| bit_set | 5 | 498 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| __rustc | 5 | 336 | 0.00 % |
| log | 3 | 329 | 0.00 % |
| lock_api | 2 | 315 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % |
