# publish2d, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish2d.wasm: 18064580 B raw, 10892604 B after wasm-opt, 3086957 B brotli -q 11 (73.6 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 10,795,704 B raw; "brotli alone" is each group compressed by itself (they sum to 3,239,960 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 940 | 1,094,809 | 10.14 % | 345,923 | 10.68 % |
| [data] | 1 | 1,332,212 | 12.34 % | 329,120 | 10.16 % |
| og_host_web (exports) | 382 | 1,162,002 | 10.76 % | 321,921 | 9.94 % |
| rapier2d | 323 | 731,011 | 6.77 % | 238,206 | 7.35 % |
| og_render | 596 | 737,042 | 6.83 % | 236,348 | 7.29 % |
| naga | 1,251 | 624,896 | 5.79 % | 186,441 | 5.75 % |
| og_physics | 274 | 573,133 | 5.31 % | 160,707 | 4.96 % |
| parry2d | 780 | 446,904 | 4.14 % | 128,376 | 3.96 % |
| wgpu | 376 | 360,357 | 3.34 % | 128,199 | 3.96 % |
| og_ui | 273 | 314,721 | 2.92 % | 107,461 | 3.32 % |
| og_anim | 329 | 352,062 | 3.26 % | 105,821 | 3.27 % |
| std | 1,533 | 407,977 | 3.78 % | 103,801 | 3.20 % |
| wgpu_core | 944 | 322,318 | 2.99 % | 91,858 | 2.84 % |
| wgpu_hal | 353 | 167,090 | 1.55 % | 61,445 | 1.90 % |
| og_nav | 91 | 149,273 | 1.38 % | 50,070 | 1.55 % |
| og_ecs | 472 | 183,191 | 1.70 % | 50,000 | 1.54 % |
| bincode | 71 | 173,452 | 1.61 % | 48,197 | 1.49 % |
| ttf_parser | 292 | 129,768 | 1.20 % | 42,736 | 1.32 % |
| rustybuzz | 294 | 108,884 | 1.01 % | 40,590 | 1.25 % |
| og_core | 526 | 168,081 | 1.56 % | 35,941 | 1.11 % |
| og_audio | 116 | 87,556 | 0.81 % | 32,989 | 1.02 % |
| og_ai | 76 | 89,924 | 0.83 % | 30,017 | 0.93 % |
| og_cook | 264 | 92,342 | 0.86 % | 29,055 | 0.90 % |
| [unnamed] | 377 | 93,760 | 0.87 % | 27,352 | 0.84 % |
| og_probe | 94 | 74,217 | 0.69 % | 27,268 | 0.84 % |
| serde_json | 254 | 82,126 | 0.76 % | 26,619 | 0.82 % |
| js_sys | 63 | 73,878 | 0.68 % | 26,556 | 0.82 % |
| og_host_web | 77 | 45,655 | 0.42 % | 16,765 | 0.52 % |
| og_geom | 59 | 42,381 | 0.39 % | 16,476 | 0.51 % |
| og_input | 70 | 56,049 | 0.52 % | 12,598 | 0.39 % |
| og_tilemap | 51 | 29,788 | 0.28 % | 12,212 | 0.38 % |
| glam | 79 | 32,783 | 0.30 % | 11,984 | 0.37 % |
| og_tween | 65 | 29,136 | 0.27 % | 10,906 | 0.34 % |
| og_net | 61 | 31,229 | 0.29 % | 10,887 | 0.34 % |
| og_i18n | 79 | 25,714 | 0.24 % | 10,400 | 0.32 % |
| wgpu_types | 284 | 29,659 | 0.27 % | 10,344 | 0.32 % |
| indexmap | 115 | 42,671 | 0.40 % | 9,827 | 0.30 % |
| miniz_oxide | 30 | 20,049 | 0.19 % | 8,882 | 0.27 % |
| codespan_reporting | 57 | 21,178 | 0.20 % | 8,437 | 0.26 % |
| spade | 42 | 23,633 | 0.22 % | 8,101 | 0.25 % |
| compiler_builtins | 19 | 19,291 | 0.18 % | 7,677 | 0.24 % |
| robust | 7 | 28,597 | 0.26 % | 7,671 | 0.24 % |
| og_lanes | 71 | 31,559 | 0.29 % | 7,624 | 0.24 % |
| [other] | 83 | 19,211 | 0.18 % | 7,199 | 0.22 % |
| og_timeline | 42 | 14,013 | 0.13 % | 5,838 | 0.18 % |
| dlmalloc | 15 | 12,487 | 0.12 % | 3,694 | 0.11 % |
| libm | 13 | 8,992 | 0.08 % | 3,326 | 0.10 % |
| og_blob | 28 | 14,680 | 0.14 % | 3,270 | 0.10 % |
| naga_types | 58 | 6,851 | 0.06 % | 3,216 | 0.10 % |
| arrayvec | 98 | 7,138 | 0.07 % | 2,962 | 0.09 % |
| smallvec | 59 | 10,276 | 0.10 % | 2,962 | 0.09 % |
| glow | 79 | 10,433 | 0.10 % | 2,845 | 0.09 % |
| nalgebra | 9 | 5,426 | 0.05 % | 2,380 | 0.07 % |
| unicode_bidi | 13 | 5,844 | 0.05 % | 2,363 | 0.07 % |
| wasm_bindgen | 47 | 5,274 | 0.05 % | 1,915 | 0.06 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,351 | 0.04 % |
| parking_lot | 6 | 3,837 | 0.04 % | 1,226 | 0.04 % |
| serde_core | 23 | 3,048 | 0.03 % | 1,170 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,114 | 0.03 % |
| parking_lot_core | 9 | 1,613 | 0.01 % | 836 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 828 | 0.03 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 809 | 0.02 % |
| foldhash | 4 | 3,050 | 0.03 % | 653 | 0.02 % |
| once_cell | 5 | 1,081 | 0.01 % | 629 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 514 | 0.02 % |
| glamx | 3 | 1,100 | 0.01 % | 512 | 0.02 % |
| bitflags | 9 | 1,410 | 0.01 % | 478 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 411 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 404 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 402 | 0.01 % |
| bit_vec | 8 | 628 | 0.01 % | 395 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 387 | 0.01 % |
| bit_set | 5 | 486 | 0.00 % | 353 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 325 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 263 | 0.01 % |
| lock_api | 2 | 310 | 0.00 % | 234 | 0.01 % |
| __rustc | 5 | 325 | 0.00 % | 208 | 0.01 % |
| log | 3 | 321 | 0.00 % | 195 | 0.01 % |
| bytemuck | 5 | 174 | 0.00 % | 170 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 152 | 0.00 % |
| simd_adler32 | 1 | 201 | 0.00 % | 140 | 0.00 % |
| serde_arrays | 1 | 19 | 0.00 % | 23 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 411,588 | 137,217 | og_render | `<og_render::Renderer>::render` |
| 256,541 | 62,848 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 227,188 | 78,646 | og_physics | `og_physics::schedule::{closure#0}` |
| 165,195 | 61,936 | og_runtime | `og_runtime::extract::extract` |
| 151,443 | 53,330 | rapier2d | `<rapier2d::dynamics::solver::staged_island_solver::StagedIslandSolver>::init_and_solve` |
| 118,745 | 33,803 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 97,563 | 36,110 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 87,743 | 29,451 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 68,969 | 24,628 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 65,114 | 24,837 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 54,681 | 21,679 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,091 | 18,008 | og_anim | `og_anim::system::install::{closure#0}` |
| 50,134 | 17,405 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 48,476 | 9,021 | og_physics | `og_physics::gather::gather` |
| 41,387 | 16,571 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 38,261 | 11,135 | og_anim | `og_anim::rig2d::parse_body` |
| 38,124 | 7,292 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 36,026 | 14,160 | og_runtime | `og_runtime::timeline::decode` |
| 32,913 | 10,461 | og_host_web (exports) | `og_room_deliver` |
| 31,305 | 6,496 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 31,139 | 11,968 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,945 | 10,187 | og_runtime | `og_runtime::engine::call_rpc` |
| 30,730 | 11,149 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,369 | 10,294 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 28,269 | 11,263 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,088 | 8,982 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,910 | 9,709 | naga | `<naga::valid::Validator>::validate_function` |
| 26,680 | 6,821 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 26,212 | 10,760 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 25,994 | 10,008 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,439 | 9,597 | og_runtime | `og_runtime::anim::load_animation` |
| 24,593 | 9,508 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 24,457 | 3,920 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 24,402 | 9,387 | og_audio | `og_audio::system::run` |
| 23,982 | 7,873 | og_runtime | `og_runtime::modules::load_style` |
| 23,736 | 5,229 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,205 | 9,256 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,902 | 8,616 | og_render | `og_render::id_pass::render_ids_from` |
| 22,833 | 8,215 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 22,830 | 8,692 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,971 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,655 | 9,515 | og_nav | `og_nav::system::run` |
| 22,317 | 9,396 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,931 | 9,281 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 20,523 | 9,053 | og_runtime | `og_runtime::stream::fetch` |
| 20,120 | 7,879 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,083 | 7,388 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 20,049 | 8,641 | og_runtime | `og_runtime::assets::add_gltf` |
| 19,777 | 8,151 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,583 | 6,153 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,258 | 8,285 | og_host_web (exports) | `og_load_rig2d multivalue shim` |
| 19,069 | 7,850 | og_ui | `og_ui::text::bidi_runs` |
| 18,794 | 6,067 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,584 | 7,813 | rapier2d | `<rapier2d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |
| 18,545 | 5,921 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 18,032 | 6,401 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,906 | 7,427 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 17,517 | 7,210 | og_runtime | `og_runtime::save::apply_keeping` |
| 16,649 | 6,481 | rapier2d | `<rapier2d::pipeline::physics_world::PhysicsWorld>::remove_body` |
| 16,349 | 6,913 | wgpu | `<wgpu::backend::wgpu_core::CoreAdapter as wgpu::dispatch::AdapterInterface>::request_device` |

twiggy top, per crate (shallow bytes; names and producers left out): 10,892,327 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3572 | 1,332,210 | 12.23 % |
| og_host_web (exports) | 382 | 1,163,155 | 10.68 % |
| og_runtime | 940 | 1,097,355 | 10.07 % |
| og_render | 596 | 738,630 | 6.78 % |
| rapier2d | 323 | 731,987 | 6.72 % |
| naga | 1251 | 627,906 | 5.76 % |
| og_physics | 274 | 573,877 | 5.27 % |
| parry2d | 780 | 448,951 | 4.12 % |
| std | 1533 | 411,778 | 3.78 % |
| wgpu | 376 | 361,293 | 3.32 % |
| og_anim | 329 | 352,966 | 3.24 % |
| wgpu_core | 944 | 324,667 | 2.98 % |
| og_ui | 273 | 315,488 | 2.90 % |
| og_ecs | 472 | 184,519 | 1.69 % |
| bincode | 71 | 173,658 | 1.59 % |
| og_core | 526 | 169,401 | 1.56 % |
| wgpu_hal | 353 | 167,926 | 1.54 % |
| og_nav | 91 | 149,530 | 1.37 % |
| ttf_parser | 292 | 130,522 | 1.20 % |
| rustybuzz | 294 | 109,610 | 1.01 % |
| [unnamed] | 377 | 94,746 | 0.87 % |
| og_cook | 264 | 92,946 | 0.85 % |
| og_ai | 76 | 90,133 | 0.83 % |
| og_audio | 116 | 87,859 | 0.81 % |
| serde_json | 254 | 82,785 | 0.76 % |
| og_probe | 94 | 74,469 | 0.68 % |
| js_sys | 63 | 74,015 | 0.68 % |
| [wasm structure] | 1461 | 62,472 | 0.57 % |
| og_input | 70 | 56,205 | 0.52 % |
| og_host_web | 77 | 45,856 | 0.42 % |
| indexmap | 115 | 42,949 | 0.39 % |
| og_geom | 59 | 42,558 | 0.39 % |
| glam | 79 | 32,995 | 0.30 % |
| og_lanes | 71 | 31,764 | 0.29 % |
| og_net | 61 | 31,392 | 0.29 % |
| wgpu_types | 284 | 30,283 | 0.28 % |
| og_tilemap | 51 | 29,920 | 0.27 % |
| og_tween | 65 | 29,308 | 0.27 % |
| robust | 7 | 28,622 | 0.26 % |
| og_i18n | 79 | 25,915 | 0.24 % |
| spade | 42 | 23,741 | 0.22 % |
| codespan_reporting | 57 | 21,322 | 0.20 % |
| miniz_oxide | 30 | 20,121 | 0.18 % |
| [other] | 88 | 19,452 | 0.18 % |
| compiler_builtins | 19 | 19,349 | 0.18 % |
| og_blob | 28 | 14,754 | 0.14 % |
| og_timeline | 42 | 14,125 | 0.13 % |
| dlmalloc | 15 | 12,530 | 0.12 % |
| glow | 79 | 10,625 | 0.10 % |
| smallvec | 59 | 10,419 | 0.10 % |
| libm | 13 | 9,030 | 0.08 % |
| arrayvec | 98 | 7,346 | 0.07 % |
| naga_types | 58 | 6,979 | 0.06 % |
| unicode_bidi | 13 | 5,875 | 0.05 % |
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
| fdeflate | 1 | 829 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_vec | 8 | 645 | 0.01 % |
| bit_set | 5 | 498 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| __rustc | 5 | 336 | 0.00 % |
| log | 3 | 329 | 0.00 % |
| lock_api | 2 | 315 | 0.00 % |
| simd_adler32 | 1 | 204 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % |
