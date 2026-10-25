# lanes, before (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes.wasm: 19784009 B raw, 12055848 B after wasm-opt, 3370010 B brotli -q 11 (80.3 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 11,949,269 B raw; "brotli alone" is each group compressed by itself (they sum to 3,535,789 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 1,034 | 1,150,835 | 9.63 % | 366,252 | 10.36 % |
| rapier3d | 392 | 1,047,353 | 8.76 % | 340,267 | 9.62 % |
| [data] | 1 | 1,477,835 | 12.37 % | 331,683 | 9.38 % |
| og_host_web (exports) | 383 | 1,132,615 | 9.48 % | 323,644 | 9.15 % |
| og_render | 682 | 854,323 | 7.15 % | 276,560 | 7.82 % |
| parry3d | 999 | 765,464 | 6.41 % | 205,336 | 5.81 % |
| naga | 1,255 | 625,099 | 5.23 % | 185,739 | 5.25 % |
| og_physics | 271 | 534,197 | 4.47 % | 139,386 | 3.94 % |
| wgpu | 385 | 354,824 | 2.97 % | 127,758 | 3.61 % |
| og_ui | 269 | 313,695 | 2.63 % | 106,868 | 3.02 % |
| std | 1,659 | 409,336 | 3.43 % | 106,852 | 3.02 % |
| og_anim | 326 | 353,345 | 2.96 % | 106,570 | 3.01 % |
| wgpu_core | 951 | 329,076 | 2.75 % | 93,399 | 2.64 % |
| wgpu_hal | 355 | 166,569 | 1.39 % | 61,485 | 1.74 % |
| og_nav | 87 | 148,196 | 1.24 % | 49,943 | 1.41 % |
| og_ecs | 472 | 181,208 | 1.52 % | 49,147 | 1.39 % |
| bincode | 79 | 169,183 | 1.42 % | 46,179 | 1.31 % |
| ttf_parser | 293 | 129,812 | 1.09 % | 42,885 | 1.21 % |
| rustybuzz | 307 | 109,336 | 0.92 % | 40,365 | 1.14 % |
| og_core | 530 | 171,148 | 1.43 % | 36,381 | 1.03 % |
| og_audio | 116 | 84,241 | 0.70 % | 32,623 | 0.92 % |
| og_cook | 313 | 100,854 | 0.84 % | 32,606 | 0.92 % |
| [unnamed] | 405 | 106,663 | 0.89 % | 30,876 | 0.87 % |
| og_ai | 76 | 89,751 | 0.75 % | 29,696 | 0.84 % |
| rayon | 36 | 92,313 | 0.77 % | 29,218 | 0.83 % |
| js_sys | 74 | 74,619 | 0.62 % | 27,246 | 0.77 % |
| serde_json | 254 | 80,195 | 0.67 % | 26,280 | 0.74 % |
| og_probe | 85 | 67,993 | 0.57 % | 25,549 | 0.72 % |
| rayon_core | 278 | 137,106 | 1.15 % | 21,344 | 0.60 % |
| og_host_web | 80 | 47,477 | 0.40 % | 17,494 | 0.49 % |
| og_geom | 60 | 42,945 | 0.36 % | 16,760 | 0.47 % |
| og_lanes | 101 | 52,822 | 0.44 % | 15,676 | 0.44 % |
| og_tilemap | 53 | 29,757 | 0.25 % | 12,236 | 0.35 % |
| og_input | 70 | 48,407 | 0.41 % | 12,087 | 0.34 % |
| glam | 79 | 30,481 | 0.26 % | 11,548 | 0.33 % |
| og_tween | 62 | 28,893 | 0.24 % | 10,833 | 0.31 % |
| og_i18n | 86 | 26,609 | 0.22 % | 10,616 | 0.30 % |
| og_net | 63 | 28,502 | 0.24 % | 10,419 | 0.29 % |
| wgpu_types | 283 | 29,306 | 0.25 % | 10,325 | 0.29 % |
| indexmap | 126 | 45,025 | 0.38 % | 9,632 | 0.27 % |
| miniz_oxide | 30 | 20,037 | 0.17 % | 8,878 | 0.25 % |
| codespan_reporting | 58 | 21,797 | 0.18 % | 8,754 | 0.25 % |
| [other] | 97 | 23,038 | 0.19 % | 8,536 | 0.24 % |
| spade | 44 | 23,506 | 0.20 % | 8,003 | 0.23 % |
| compiler_builtins | 37 | 17,989 | 0.15 % | 7,757 | 0.22 % |
| robust | 7 | 28,597 | 0.24 % | 7,689 | 0.22 % |
| og_timeline | 42 | 12,967 | 0.11 % | 5,641 | 0.16 % |
| nalgebra | 14 | 9,049 | 0.08 % | 3,988 | 0.11 % |
| libm | 13 | 8,992 | 0.08 % | 3,332 | 0.09 % |
| og_blob | 35 | 10,742 | 0.09 % | 3,326 | 0.09 % |
| naga_types | 56 | 6,881 | 0.06 % | 3,173 | 0.09 % |
| smallvec | 64 | 10,957 | 0.09 % | 3,039 | 0.09 % |
| glow | 77 | 9,691 | 0.08 % | 2,654 | 0.08 % |
| arrayvec | 93 | 6,249 | 0.05 % | 2,641 | 0.07 % |
| crossbeam_channel | 28 | 7,933 | 0.07 % | 2,486 | 0.07 % |
| dlmalloc | 22 | 5,813 | 0.05 % | 2,363 | 0.07 % |
| unicode_bidi | 12 | 5,820 | 0.05 % | 2,326 | 0.07 % |
| wasm_bindgen | 58 | 5,210 | 0.04 % | 1,844 | 0.05 % |
| glamx | 6 | 3,442 | 0.03 % | 1,646 | 0.05 % |
| crossbeam_epoch | 27 | 4,196 | 0.04 % | 1,511 | 0.04 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,342 | 0.04 % |
| parking_lot | 6 | 3,801 | 0.03 % | 1,274 | 0.04 % |
| serde_core | 24 | 3,326 | 0.03 % | 1,222 | 0.03 % |
| crossbeam_deque | 19 | 3,781 | 0.03 % | 1,221 | 0.03 % |
| zmij | 1 | 2,148 | 0.02 % | 1,113 | 0.03 % |
| parking_lot_core | 11 | 1,729 | 0.01 % | 937 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 826 | 0.02 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 814 | 0.02 % |
| foldhash | 4 | 3,041 | 0.03 % | 650 | 0.02 % |
| ena | 4 | 1,680 | 0.01 % | 623 | 0.02 % |
| once_cell | 5 | 945 | 0.01 % | 540 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 527 | 0.01 % |
| wasm_bindgen_rayon | 3 | 1,084 | 0.01 % | 483 | 0.01 % |
| bitflags | 9 | 1,410 | 0.01 % | 479 | 0.01 % |
| bit_vec | 9 | 717 | 0.01 % | 440 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 422 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 404 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 403 | 0.01 % |
| bit_set | 6 | 590 | 0.00 % | 398 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 394 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 326 | 0.01 % |
| wasm_sync | 4 | 424 | 0.00 % | 247 | 0.01 % |
| log | 4 | 481 | 0.00 % | 236 | 0.01 % |
| __rustc | 5 | 316 | 0.00 % | 236 | 0.01 % |
| lock_api | 2 | 278 | 0.00 % | 229 | 0.01 % |
| simd_adler32 | 1 | 250 | 0.00 % | 174 | 0.00 % |
| bytemuck | 5 | 174 | 0.00 % | 173 | 0.00 % |
| unicode_properties | 2 | 171 | 0.00 % | 159 | 0.00 % |
| crossbeam_utils | 1 | 82 | 0.00 % | 79 | 0.00 % |
| serde_arrays | 1 | 64 | 0.00 % | 68 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 455,704 | 153,483 | og_render | `<og_render::Renderer>::render` |
| 259,034 | 64,773 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 197,073 | 73,386 | og_runtime | `og_runtime::extract::extract` |
| 173,844 | 62,460 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 126,206 | 35,668 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 126,108 | 41,409 | og_physics | `og_physics::dim3::sim::step_here` |
| 99,662 | 32,787 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 95,623 | 35,733 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 88,541 | 29,631 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 67,638 | 24,464 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 54,165 | 21,654 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,421 | 18,201 | og_anim | `og_anim::system::install::{closure#0}` |
| 51,310 | 17,574 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,841 | 9,301 | og_physics | `og_physics::gather::gather` |
| 41,752 | 16,706 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 41,202 | 15,446 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 38,133 | 11,343 | og_anim | `og_anim::rig2d::parse_body` |
| 37,019 | 14,159 | og_runtime | `og_runtime::timeline::decode` |
| 36,202 | 12,652 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 33,191 | 6,407 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 30,888 | 11,844 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,442 | 6,592 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 29,925 | 11,053 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 28,885 | 12,384 | rapier3d | `<rapier3d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 28,824 | 11,836 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,428 | 10,020 | og_host_web (exports) | `og_room_deliver` |
| 28,134 | 8,957 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,640 | 9,647 | naga | `<naga::valid::Validator>::validate_function` |
| 27,251 | 9,820 | og_runtime | `og_runtime::engine::call_rpc` |
| 26,443 | 11,059 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 26,278 | 6,683 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 26,151 | 9,941 | og_runtime | `og_runtime::anim::load_animation` |
| 25,835 | 10,083 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,549 | 10,649 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier3d::dynamics::solver:...` |
| 24,791 | 9,625 | og_audio | `og_audio::system::run` |
| 24,652 | 9,712 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 24,464 | 3,901 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 23,558 | 7,711 | og_runtime | `og_runtime::modules::load_style` |
| 23,500 | 5,154 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,998 | 8,412 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 22,839 | 9,532 | og_nav | `og_nav::system::run` |
| 22,699 | 8,986 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,646 | 8,301 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,624 | 8,562 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 22,595 | 9,168 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,355 | 9,431 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 22,222 | 8,518 | og_render | `og_render::id_pass::render_ids_from` |
| 21,948 | 9,328 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,825 | 9,518 | og_runtime | `og_runtime::stream::fetch` |
| 21,209 | 7,513 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 20,440 | 7,314 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,220 | 7,862 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,136 | 8,173 | og_physics | `og_physics::dim3::sim::shape` |
| 19,930 | 8,132 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,889 | 8,480 | og_runtime | `og_runtime::assets::add_gltf` |
| 19,691 | 6,184 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,646 | 7,331 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,292 | 8,356 | og_host_web (exports) | `og_load_rig2d multivalue shim` |
| 19,215 | 6,850 | og_physics | `og_physics::schedule::{closure#0}` |
| 18,969 | 7,816 | og_ui | `og_ui::text::bidi_runs` |

twiggy top, per crate (shallow bytes; names and producers left out): 12,055,578 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,477,834 | 12.26 % |
| og_runtime | 1034 | 1,153,633 | 9.57 % |
| og_host_web (exports) | 383 | 1,133,771 | 9.40 % |
| rapier3d | 392 | 1,048,519 | 8.70 % |
| og_render | 682 | 856,139 | 7.10 % |
| parry3d | 999 | 768,154 | 6.37 % |
| naga | 1255 | 628,118 | 5.21 % |
| og_physics | 271 | 534,932 | 4.44 % |
| std | 1659 | 413,401 | 3.43 % |
| wgpu | 385 | 355,778 | 2.95 % |
| og_anim | 326 | 354,236 | 2.94 % |
| wgpu_core | 951 | 331,445 | 2.75 % |
| og_ui | 269 | 314,454 | 2.61 % |
| og_ecs | 472 | 182,535 | 1.51 % |
| og_core | 530 | 172,482 | 1.43 % |
| bincode | 79 | 169,411 | 1.41 % |
| wgpu_hal | 355 | 167,406 | 1.39 % |
| og_nav | 87 | 148,441 | 1.23 % |
| rayon_core | 278 | 137,899 | 1.14 % |
| ttf_parser | 293 | 130,568 | 1.08 % |
| rustybuzz | 307 | 110,085 | 0.91 % |
| [unnamed] | 405 | 107,722 | 0.89 % |
| og_cook | 313 | 101,576 | 0.84 % |
| rayon | 36 | 92,419 | 0.77 % |
| og_ai | 76 | 89,959 | 0.75 % |
| og_audio | 116 | 84,541 | 0.70 % |
| serde_json | 254 | 80,841 | 0.67 % |
| js_sys | 74 | 74,781 | 0.62 % |
| [wasm structure] | 1500 | 68,951 | 0.57 % |
| og_probe | 85 | 68,221 | 0.57 % |
| og_lanes | 101 | 53,113 | 0.44 % |
| og_input | 70 | 48,563 | 0.40 % |
| og_host_web | 80 | 47,687 | 0.40 % |
| indexmap | 126 | 45,328 | 0.38 % |
| og_geom | 60 | 43,124 | 0.36 % |
| glam | 79 | 30,690 | 0.25 % |
| wgpu_types | 283 | 29,923 | 0.25 % |
| og_tilemap | 53 | 29,891 | 0.25 % |
| og_tween | 62 | 29,055 | 0.24 % |
| og_net | 63 | 28,669 | 0.24 % |
| robust | 7 | 28,622 | 0.24 % |
| og_i18n | 86 | 26,827 | 0.22 % |
| spade | 44 | 23,617 | 0.20 % |
| [other] | 102 | 23,306 | 0.19 % |
| codespan_reporting | 58 | 21,942 | 0.18 % |
| miniz_oxide | 30 | 20,109 | 0.17 % |
| compiler_builtins | 37 | 18,099 | 0.15 % |
| og_timeline | 42 | 13,077 | 0.11 % |
| smallvec | 64 | 11,110 | 0.09 % |
| og_blob | 35 | 10,829 | 0.09 % |
| glow | 77 | 9,879 | 0.08 % |
| nalgebra | 14 | 9,090 | 0.08 % |
| libm | 13 | 9,030 | 0.07 % |
| crossbeam_channel | 28 | 8,004 | 0.07 % |
| naga_types | 56 | 7,004 | 0.06 % |
| arrayvec | 93 | 6,445 | 0.05 % |
| dlmalloc | 22 | 5,866 | 0.05 % |
| unicode_bidi | 12 | 5,849 | 0.05 % |
| wasm_bindgen | 58 | 5,339 | 0.04 % |
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
| fdeflate | 1 | 829 | 0.01 % |
| bit_vec | 9 | 736 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_set | 6 | 604 | 0.01 % |
| log | 4 | 492 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| wasm_sync | 4 | 433 | 0.00 % |
| __rustc | 5 | 327 | 0.00 % |
| lock_api | 2 | 283 | 0.00 % |
| simd_adler32 | 1 | 253 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| crossbeam_utils | 1 | 84 | 0.00 % |
| serde_arrays | 1 | 66 | 0.00 % |
