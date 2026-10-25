# lanes, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes.wasm: 19514798 B raw, 11850400 B after wasm-opt, 3320422 B brotli -q 11 (79.2 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 11,744,342 B raw; "brotli alone" is each group compressed by itself (they sum to 3,483,764 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 1,011 | 1,126,047 | 9.59 % | 359,074 | 10.31 % |
| rapier3d | 390 | 1,046,994 | 8.91 % | 339,794 | 9.75 % |
| [data] | 1 | 1,462,099 | 12.45 % | 328,975 | 9.44 % |
| og_host_web (exports) | 382 | 1,074,549 | 9.15 % | 308,649 | 8.86 % |
| og_render | 688 | 845,414 | 7.20 % | 275,035 | 7.89 % |
| parry3d | 999 | 765,464 | 6.52 % | 205,513 | 5.90 % |
| naga | 1,255 | 625,099 | 5.32 % | 186,106 | 5.34 % |
| og_physics | 275 | 537,398 | 4.58 % | 142,270 | 4.08 % |
| wgpu | 383 | 354,649 | 3.02 % | 127,846 | 3.67 % |
| std | 1,646 | 410,541 | 3.50 % | 107,282 | 3.08 % |
| og_ui | 256 | 309,462 | 2.63 % | 106,052 | 3.04 % |
| wgpu_core | 951 | 329,076 | 2.80 % | 93,239 | 2.68 % |
| og_anim | 285 | 276,786 | 2.36 % | 82,770 | 2.38 % |
| wgpu_hal | 355 | 166,569 | 1.42 % | 61,545 | 1.77 % |
| og_nav | 87 | 148,195 | 1.26 % | 49,852 | 1.43 % |
| og_ecs | 462 | 179,285 | 1.53 % | 48,875 | 1.40 % |
| bincode | 79 | 169,183 | 1.44 % | 46,126 | 1.32 % |
| ttf_parser | 293 | 129,812 | 1.11 % | 42,883 | 1.23 % |
| rustybuzz | 307 | 109,336 | 0.93 % | 40,427 | 1.16 % |
| og_core | 527 | 159,714 | 1.36 % | 34,556 | 0.99 % |
| og_cook | 309 | 102,274 | 0.87 % | 32,753 | 0.94 % |
| [unnamed] | 402 | 107,500 | 0.92 % | 31,429 | 0.90 % |
| og_ai | 75 | 89,738 | 0.76 % | 29,700 | 0.85 % |
| rayon | 36 | 92,313 | 0.79 % | 29,188 | 0.84 % |
| og_audio | 103 | 71,056 | 0.61 % | 27,592 | 0.79 % |
| js_sys | 74 | 74,567 | 0.63 % | 27,125 | 0.78 % |
| serde_json | 253 | 80,486 | 0.69 % | 26,494 | 0.76 % |
| og_probe | 93 | 70,153 | 0.60 % | 26,314 | 0.76 % |
| rayon_core | 279 | 137,127 | 1.17 % | 21,372 | 0.61 % |
| og_host_web | 80 | 47,473 | 0.40 % | 17,518 | 0.50 % |
| og_geom | 60 | 42,945 | 0.37 % | 16,733 | 0.48 % |
| og_lanes | 89 | 49,988 | 0.43 % | 15,404 | 0.44 % |
| og_tilemap | 53 | 29,757 | 0.25 % | 12,241 | 0.35 % |
| og_input | 70 | 48,503 | 0.41 % | 11,977 | 0.34 % |
| glam | 79 | 30,481 | 0.26 % | 11,494 | 0.33 % |
| miniz_oxide | 31 | 26,078 | 0.22 % | 11,390 | 0.33 % |
| og_tween | 62 | 28,893 | 0.25 % | 10,843 | 0.31 % |
| og_i18n | 86 | 26,609 | 0.23 % | 10,599 | 0.30 % |
| og_net | 63 | 28,502 | 0.24 % | 10,394 | 0.30 % |
| wgpu_types | 283 | 29,306 | 0.25 % | 10,243 | 0.29 % |
| indexmap | 126 | 45,025 | 0.38 % | 9,598 | 0.28 % |
| codespan_reporting | 58 | 21,797 | 0.19 % | 8,752 | 0.25 % |
| [other] | 97 | 23,038 | 0.20 % | 8,547 | 0.25 % |
| spade | 44 | 23,506 | 0.20 % | 7,992 | 0.23 % |
| compiler_builtins | 37 | 17,989 | 0.15 % | 7,748 | 0.22 % |
| robust | 7 | 28,597 | 0.24 % | 7,703 | 0.22 % |
| og_timeline | 42 | 12,967 | 0.11 % | 5,630 | 0.16 % |
| nalgebra | 14 | 9,049 | 0.08 % | 3,942 | 0.11 % |
| libm | 13 | 8,992 | 0.08 % | 3,327 | 0.10 % |
| og_blob | 34 | 9,890 | 0.08 % | 3,209 | 0.09 % |
| naga_types | 56 | 6,881 | 0.06 % | 3,186 | 0.09 % |
| smallvec | 64 | 10,957 | 0.09 % | 3,046 | 0.09 % |
| glow | 77 | 9,691 | 0.08 % | 2,671 | 0.08 % |
| arrayvec | 93 | 6,249 | 0.05 % | 2,649 | 0.08 % |
| crossbeam_channel | 28 | 7,933 | 0.07 % | 2,479 | 0.07 % |
| dlmalloc | 22 | 5,813 | 0.05 % | 2,375 | 0.07 % |
| unicode_bidi | 12 | 5,820 | 0.05 % | 2,324 | 0.07 % |
| wasm_bindgen | 58 | 5,210 | 0.04 % | 1,860 | 0.05 % |
| glamx | 6 | 3,442 | 0.03 % | 1,649 | 0.05 % |
| crossbeam_epoch | 27 | 4,196 | 0.04 % | 1,500 | 0.04 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,346 | 0.04 % |
| parking_lot | 6 | 3,801 | 0.03 % | 1,284 | 0.04 % |
| crossbeam_deque | 19 | 3,781 | 0.03 % | 1,233 | 0.04 % |
| serde_core | 24 | 3,326 | 0.03 % | 1,221 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,138 | 0.03 % |
| parking_lot_core | 11 | 1,729 | 0.01 % | 941 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 822 | 0.02 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 809 | 0.02 % |
| foldhash | 4 | 3,041 | 0.03 % | 652 | 0.02 % |
| ena | 4 | 1,680 | 0.01 % | 623 | 0.02 % |
| once_cell | 5 | 945 | 0.01 % | 538 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 518 | 0.01 % |
| wasm_bindgen_rayon | 3 | 1,084 | 0.01 % | 484 | 0.01 % |
| bitflags | 9 | 1,410 | 0.01 % | 481 | 0.01 % |
| bit_vec | 9 | 717 | 0.01 % | 440 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 414 | 0.01 % |
| bit_set | 6 | 590 | 0.01 % | 404 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 402 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 399 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 326 | 0.01 % |
| wasm_sync | 4 | 424 | 0.00 % | 246 | 0.01 % |
| log | 4 | 481 | 0.00 % | 237 | 0.01 % |
| __rustc | 5 | 316 | 0.00 % | 236 | 0.01 % |
| lock_api | 2 | 278 | 0.00 % | 229 | 0.01 % |
| bytemuck | 5 | 174 | 0.00 % | 174 | 0.00 % |
| unicode_properties | 2 | 171 | 0.00 % | 162 | 0.00 % |
| crossbeam_utils | 1 | 82 | 0.00 % | 78 | 0.00 % |
| serde_arrays | 1 | 64 | 0.00 % | 68 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 451,523 | 153,341 | og_render | `<og_render::Renderer>::render` |
| 238,724 | 62,962 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 194,504 | 72,588 | og_runtime | `og_runtime::extract::extract` |
| 174,684 | 62,660 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 126,206 | 35,603 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 119,079 | 39,868 | og_physics | `og_physics::dim3::sim::step_here` |
| 99,662 | 32,580 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 95,623 | 35,719 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 84,870 | 28,100 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 67,586 | 24,417 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 55,984 | 22,404 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,310 | 17,578 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,841 | 9,280 | og_physics | `og_physics::gather::gather` |
| 41,752 | 16,722 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 41,202 | 15,462 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 38,124 | 11,355 | og_anim | `og_anim::rig2d::parse_body` |
| 37,019 | 14,150 | og_runtime | `og_runtime::timeline::decode` |
| 36,202 | 12,687 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 33,191 | 6,400 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 30,888 | 11,851 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,669 | 10,991 | og_runtime | `og_runtime::engine::call_rpc` |
| 30,442 | 6,649 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 29,925 | 11,045 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 28,885 | 12,413 | rapier3d | `<rapier3d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 28,824 | 11,838 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,428 | 10,026 | og_host_web (exports) | `og_room_deliver` |
| 28,134 | 8,965 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,640 | 9,632 | naga | `<naga::valid::Validator>::validate_function` |
| 26,278 | 6,666 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 25,835 | 10,082 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,549 | 10,652 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier3d::dynamics::solver:...` |
| 24,464 | 3,859 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 23,558 | 7,714 | og_runtime | `og_runtime::modules::load_style` |
| 23,500 | 5,172 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,839 | 9,500 | og_nav | `og_nav::system::run` |
| 22,699 | 8,976 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,641 | 8,278 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,624 | 8,536 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 22,595 | 9,154 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,499 | 8,500 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,355 | 9,445 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,948 | 9,336 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,850 | 8,761 | og_audio | `og_audio::system::install::{closure#0}` |
| 21,821 | 9,519 | og_runtime | `og_runtime::stream::fetch` |
| 21,679 | 8,232 | og_render | `og_render::id_pass::render_ids_from` |
| 21,491 | 8,000 | og_runtime | `og_runtime::anim::load_animation` |
| 21,209 | 7,558 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 20,440 | 7,336 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,220 | 7,868 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,136 | 8,166 | og_physics | `og_physics::dim3::sim::shape` |
| 19,932 | 8,141 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,691 | 6,177 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,646 | 7,347 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,243 | 6,871 | og_physics | `og_physics::schedule::{closure#0}` |
| 19,231 | 7,770 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 18,969 | 7,820 | og_ui | `og_ui::text::bidi_runs` |
| 18,804 | 7,804 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 18,334 | 6,031 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,300 | 7,526 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 18,270 | 6,541 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher>::contact_manifold_convex_co...` |

twiggy top, per crate (shallow bytes; names and producers left out): 11,850,130 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,462,098 | 12.34 % |
| og_runtime | 1011 | 1,128,793 | 9.53 % |
| og_host_web (exports) | 382 | 1,075,701 | 9.08 % |
| rapier3d | 390 | 1,048,154 | 8.85 % |
| og_render | 688 | 847,237 | 7.15 % |
| parry3d | 999 | 768,154 | 6.48 % |
| naga | 1255 | 628,118 | 5.30 % |
| og_physics | 275 | 538,142 | 4.54 % |
| std | 1646 | 414,579 | 3.50 % |
| wgpu | 383 | 355,598 | 3.00 % |
| wgpu_core | 951 | 331,445 | 2.80 % |
| og_ui | 256 | 310,189 | 2.62 % |
| og_anim | 285 | 277,565 | 2.34 % |
| og_ecs | 462 | 180,583 | 1.52 % |
| bincode | 79 | 169,411 | 1.43 % |
| wgpu_hal | 355 | 167,406 | 1.41 % |
| og_core | 527 | 161,033 | 1.36 % |
| og_nav | 87 | 148,440 | 1.25 % |
| rayon_core | 279 | 137,922 | 1.16 % |
| ttf_parser | 293 | 130,568 | 1.10 % |
| rustybuzz | 307 | 110,085 | 0.93 % |
| [unnamed] | 402 | 108,552 | 0.92 % |
| og_cook | 309 | 102,989 | 0.87 % |
| rayon | 36 | 92,419 | 0.78 % |
| og_ai | 75 | 89,944 | 0.76 % |
| serde_json | 253 | 81,128 | 0.68 % |
| js_sys | 74 | 74,729 | 0.63 % |
| og_audio | 103 | 71,326 | 0.60 % |
| og_probe | 93 | 70,401 | 0.59 % |
| [wasm structure] | 1497 | 68,766 | 0.58 % |
| og_lanes | 89 | 50,243 | 0.42 % |
| og_input | 70 | 48,659 | 0.41 % |
| og_host_web | 80 | 47,683 | 0.40 % |
| indexmap | 126 | 45,328 | 0.38 % |
| og_geom | 60 | 43,124 | 0.36 % |
| glam | 79 | 30,690 | 0.26 % |
| wgpu_types | 283 | 29,923 | 0.25 % |
| og_tilemap | 53 | 29,891 | 0.25 % |
| og_tween | 62 | 29,055 | 0.25 % |
| og_net | 63 | 28,669 | 0.24 % |
| robust | 7 | 28,622 | 0.24 % |
| og_i18n | 86 | 26,827 | 0.23 % |
| miniz_oxide | 31 | 26,153 | 0.22 % |
| spade | 44 | 23,617 | 0.20 % |
| [other] | 102 | 23,306 | 0.20 % |
| codespan_reporting | 58 | 21,942 | 0.19 % |
| compiler_builtins | 37 | 18,099 | 0.15 % |
| og_timeline | 42 | 13,077 | 0.11 % |
| smallvec | 64 | 11,110 | 0.09 % |
| og_blob | 34 | 9,974 | 0.08 % |
| glow | 77 | 9,879 | 0.08 % |
| nalgebra | 14 | 9,090 | 0.08 % |
| libm | 13 | 9,030 | 0.08 % |
| crossbeam_channel | 28 | 8,004 | 0.07 % |
| naga_types | 56 | 7,004 | 0.06 % |
| arrayvec | 93 | 6,445 | 0.05 % |
| dlmalloc | 22 | 5,866 | 0.05 % |
| unicode_bidi | 12 | 5,849 | 0.05 % |
| wasm_bindgen | 58 | 5,339 | 0.05 % |
| crossbeam_epoch | 27 | 4,261 | 0.04 % |
| crossbeam_deque | 19 | 3,827 | 0.03 % |
| parking_lot | 6 | 3,819 | 0.03 % |
| rustc_hash | 27 | 3,605 | 0.03 % |
| glamx | 6 | 3,459 | 0.03 % |
| serde_core | 24 | 3,382 | 0.03 % |
| foldhash | 4 | 3,053 | 0.03 % |
| zmij | 1 | 2,152 | 0.02 % |
| half | 21 | 1,880 | 0.02 % |
| parking_lot_core | 11 | 1,755 | 0.01 % |
| ena | 4 | 1,691 | 0.01 % |
| unicode_segmentation | 8 | 1,600 | 0.01 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,401 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| wasm_bindgen_rayon | 3 | 1,091 | 0.01 % |
| web_sys | 27 | 1,034 | 0.01 % |
| once_cell | 5 | 958 | 0.01 % |
| bit_vec | 9 | 736 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_set | 6 | 604 | 0.01 % |
| log | 4 | 492 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| wasm_sync | 4 | 433 | 0.00 % |
| __rustc | 5 | 327 | 0.00 % |
| lock_api | 2 | 283 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| crossbeam_utils | 1 | 84 | 0.00 % |
| serde_arrays | 1 | 66 | 0.00 % |
