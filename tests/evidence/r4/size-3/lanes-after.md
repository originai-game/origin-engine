# lanes, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes.wasm: 18448991 B raw, 11259570 B after wasm-opt, 3210350 B brotli -q 11 (76.5 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 11,156,199 B raw; "brotli alone" is each group compressed by itself (they sum to 3,343,537 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| rapier3d | 385 | 1,044,525 | 9.36 % | 339,243 | 10.15 % |
| og_runtime | 753 | 999,319 | 8.96 % | 335,438 | 10.03 % |
| [data] | 1 | 1,544,391 | 13.84 % | 332,602 | 9.95 % |
| og_host_web (exports) | 383 | 1,029,296 | 9.23 % | 298,308 | 8.92 % |
| og_render | 656 | 834,091 | 7.48 % | 270,514 | 8.09 % |
| parry3d | 995 | 764,145 | 6.85 % | 204,949 | 6.13 % |
| naga | 1,183 | 599,303 | 5.37 % | 180,033 | 5.38 % |
| wgpu | 385 | 338,032 | 3.03 % | 122,970 | 3.68 % |
| og_physics | 175 | 363,656 | 3.26 % | 109,376 | 3.27 % |
| og_ui | 228 | 273,607 | 2.45 % | 96,247 | 2.88 % |
| og_anim | 258 | 297,758 | 2.67 % | 94,425 | 2.82 % |
| wgpu_core | 922 | 319,078 | 2.86 % | 90,603 | 2.71 % |
| std | 1,413 | 295,202 | 2.65 % | 83,491 | 2.50 % |
| wgpu_hal | 352 | 161,363 | 1.45 % | 59,639 | 1.78 % |
| og_ecs | 461 | 174,200 | 1.56 % | 48,190 | 1.44 % |
| bincode | 79 | 175,420 | 1.57 % | 47,864 | 1.43 % |
| og_core | 701 | 199,197 | 1.79 % | 44,972 | 1.35 % |
| ttf_parser | 293 | 129,812 | 1.16 % | 42,897 | 1.28 % |
| rustybuzz | 307 | 109,334 | 0.98 % | 40,443 | 1.21 % |
| og_nav | 70 | 105,058 | 0.94 % | 38,030 | 1.14 % |
| rayon | 36 | 92,313 | 0.83 % | 29,309 | 0.88 % |
| og_cook | 306 | 83,520 | 0.75 % | 28,315 | 0.85 % |
| og_audio | 104 | 70,033 | 0.63 % | 27,541 | 0.82 % |
| og_ai | 68 | 78,906 | 0.71 % | 26,939 | 0.81 % |
| js_sys | 74 | 72,329 | 0.65 % | 26,172 | 0.78 % |
| serde_json | 249 | 79,148 | 0.71 % | 25,990 | 0.78 % |
| og_probe | 87 | 68,647 | 0.62 % | 25,849 | 0.77 % |
| [unnamed] | 371 | 78,941 | 0.71 % | 24,557 | 0.73 % |
| rayon_core | 278 | 137,188 | 1.23 % | 21,385 | 0.64 % |
| og_geom | 60 | 42,945 | 0.38 % | 16,724 | 0.50 % |
| og_lanes | 101 | 52,142 | 0.47 % | 15,509 | 0.46 % |
| og_host_web | 64 | 40,448 | 0.36 % | 15,062 | 0.45 % |
| og_i18n | 86 | 26,681 | 0.24 % | 10,654 | 0.32 % |
| og_tilemap | 42 | 25,661 | 0.23 % | 10,652 | 0.32 % |
| indexmap | 125 | 44,888 | 0.40 % | 9,527 | 0.28 % |
| glam | 70 | 24,299 | 0.22 % | 9,514 | 0.28 % |
| wgpu_types | 260 | 26,406 | 0.24 % | 9,494 | 0.28 % |
| og_net | 56 | 24,511 | 0.22 % | 9,137 | 0.27 % |
| miniz_oxide | 30 | 20,037 | 0.18 % | 8,876 | 0.27 % |
| codespan_reporting | 58 | 21,797 | 0.20 % | 8,756 | 0.26 % |
| [other] | 95 | 22,723 | 0.20 % | 8,457 | 0.25 % |
| og_input | 70 | 32,101 | 0.29 % | 8,411 | 0.25 % |
| spade | 43 | 23,502 | 0.21 % | 7,997 | 0.24 % |
| compiler_builtins | 37 | 17,989 | 0.16 % | 7,750 | 0.23 % |
| robust | 7 | 28,597 | 0.26 % | 7,645 | 0.23 % |
| og_tween | 39 | 18,299 | 0.16 % | 7,283 | 0.22 % |
| og_timeline | 42 | 12,967 | 0.12 % | 5,625 | 0.17 % |
| nalgebra | 14 | 9,049 | 0.08 % | 3,990 | 0.12 % |
| libm | 13 | 8,992 | 0.08 % | 3,331 | 0.10 % |
| og_blob | 35 | 10,742 | 0.10 % | 3,329 | 0.10 % |
| naga_types | 53 | 6,795 | 0.06 % | 3,147 | 0.09 % |
| smallvec | 64 | 10,957 | 0.10 % | 3,035 | 0.09 % |
| arrayvec | 93 | 6,249 | 0.06 % | 2,640 | 0.08 % |
| glow | 75 | 9,649 | 0.09 % | 2,636 | 0.08 % |
| crossbeam_channel | 28 | 7,933 | 0.07 % | 2,482 | 0.07 % |
| dlmalloc | 22 | 5,813 | 0.05 % | 2,370 | 0.07 % |
| unicode_bidi | 12 | 5,820 | 0.05 % | 2,331 | 0.07 % |
| wasm_bindgen | 58 | 5,210 | 0.05 % | 1,846 | 0.06 % |
| glamx | 6 | 3,442 | 0.03 % | 1,650 | 0.05 % |
| crossbeam_epoch | 27 | 4,196 | 0.04 % | 1,504 | 0.04 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,342 | 0.04 % |
| parking_lot | 6 | 3,801 | 0.03 % | 1,285 | 0.04 % |
| crossbeam_deque | 19 | 3,781 | 0.03 % | 1,231 | 0.04 % |
| serde_core | 24 | 3,326 | 0.03 % | 1,222 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,109 | 0.03 % |
| parking_lot_core | 11 | 1,729 | 0.02 % | 939 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 825 | 0.02 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 814 | 0.02 % |
| foldhash | 4 | 3,041 | 0.03 % | 651 | 0.02 % |
| once_cell | 5 | 945 | 0.01 % | 531 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 513 | 0.02 % |
| wasm_bindgen_rayon | 3 | 1,084 | 0.01 % | 487 | 0.01 % |
| bitflags | 9 | 1,410 | 0.01 % | 481 | 0.01 % |
| bit_vec | 9 | 717 | 0.01 % | 436 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 417 | 0.01 % |
| bit_set | 6 | 592 | 0.01 % | 404 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 402 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 393 | 0.01 % |
| web_sys | 26 | 956 | 0.01 % | 389 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 326 | 0.01 % |
| ena | 3 | 775 | 0.01 % | 323 | 0.01 % |
| wasm_sync | 4 | 424 | 0.00 % | 248 | 0.01 % |
| __rustc | 5 | 316 | 0.00 % | 234 | 0.01 % |
| lock_api | 2 | 278 | 0.00 % | 231 | 0.01 % |
| simd_adler32 | 1 | 250 | 0.00 % | 174 | 0.01 % |
| bytemuck | 5 | 174 | 0.00 % | 170 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 153 | 0.00 % |
| crossbeam_utils | 1 | 82 | 0.00 % | 84 | 0.00 % |
| serde_arrays | 1 | 64 | 0.00 % | 68 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 455,577 | 153,521 | og_render | `<og_render::Renderer>::render` |
| 194,871 | 50,919 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 191,846 | 72,850 | og_runtime | `og_runtime::extract::extract` |
| 173,844 | 62,513 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 132,305 | 37,126 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 110,858 | 39,717 | og_physics | `og_physics::dim3::sim::step_here` |
| 99,662 | 32,652 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 89,463 | 34,227 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 79,336 | 27,608 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 65,348 | 23,482 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 54,165 | 21,657 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,421 | 18,400 | og_anim | `og_anim::system::install::{closure#0}` |
| 51,310 | 17,578 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,826 | 9,201 | og_physics | `og_physics::gather::gather` |
| 41,202 | 15,422 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 40,141 | 16,144 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 38,133 | 11,348 | og_anim | `og_anim::rig2d::parse_body` |
| 36,931 | 14,113 | og_runtime | `og_runtime::timeline::decode` |
| 36,202 | 12,695 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 30,969 | 5,999 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 29,821 | 11,143 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,113 | 11,859 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,885 | 12,385 | rapier3d | `<rapier3d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 28,091 | 11,221 | naga | `<naga::valid::Validator>::validate_impl` |
| 27,361 | 8,714 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,175 | 9,796 | og_runtime | `og_runtime::engine::call_rpc` |
| 26,513 | 11,120 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 26,278 | 6,655 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 25,761 | 9,979 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 25,549 | 10,678 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier3d::dynamics::solver:...` |
| 24,576 | 8,373 | og_host_web (exports) | `og_room_deliver` |
| 24,464 | 3,869 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 23,619 | 8,958 | og_audio | `og_audio::system::run` |
| 23,558 | 7,705 | og_runtime | `og_runtime::modules::load_style` |
| 23,500 | 5,170 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,699 | 8,992 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,646 | 8,287 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,632 | 8,497 | og_runtime | `og_runtime::anim::load_animation` |
| 22,591 | 9,131 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,146 | 9,371 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,970 | 9,375 | og_nav | `og_nav::system::install::{closure#0}` |
| 21,570 | 8,795 | naga | `<naga::valid::Validator>::validate_function` |
| 21,360 | 9,030 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,209 | 7,583 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 21,069 | 8,181 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,440 | 7,317 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,171 | 7,428 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 20,134 | 7,681 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 20,131 | 8,165 | og_physics | `og_physics::dim3::sim::shape` |
| 19,626 | 7,323 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,386 | 6,883 | og_physics | `og_physics::schedule::{closure#0}` |
| 18,969 | 7,830 | og_ui | `og_ui::text::bidi_runs` |
| 18,855 | 7,477 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 18,780 | 7,800 | og_ai | `og_ai::system::install::{closure#0}` |
| 18,334 | 6,038 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,254 | 6,509 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher>::contact_manifold_convex_co...` |
| 18,076 | 7,566 | og_render | `og_render::id_pass::render_ids_from` |
| 17,974 | 6,320 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher<rapier3d::geometry::contact_...` |
| 17,653 | 6,301 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,602 | 5,784 | naga | `<naga::valid::Validator>::validate_block_impl` |

twiggy top, per crate (shallow bytes; names and producers left out): 11,259,300 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,544,390 | 13.72 % |
| rapier3d | 385 | 1,045,677 | 9.29 % |
| og_host_web (exports) | 383 | 1,030,451 | 9.15 % |
| og_runtime | 753 | 1,001,321 | 8.89 % |
| og_render | 656 | 835,839 | 7.42 % |
| parry3d | 995 | 766,826 | 6.81 % |
| naga | 1183 | 602,162 | 5.35 % |
| og_physics | 175 | 364,111 | 3.23 % |
| wgpu | 385 | 338,985 | 3.01 % |
| wgpu_core | 922 | 321,375 | 2.85 % |
| std | 1413 | 298,574 | 2.65 % |
| og_anim | 258 | 298,466 | 2.65 % |
| og_ui | 228 | 274,248 | 2.44 % |
| og_core | 701 | 200,946 | 1.78 % |
| bincode | 79 | 175,648 | 1.56 % |
| og_ecs | 461 | 175,497 | 1.56 % |
| wgpu_hal | 352 | 162,194 | 1.44 % |
| rayon_core | 278 | 137,981 | 1.23 % |
| ttf_parser | 293 | 130,568 | 1.16 % |
| rustybuzz | 307 | 110,083 | 0.98 % |
| og_nav | 70 | 105,251 | 0.93 % |
| rayon | 36 | 92,419 | 0.82 % |
| og_cook | 306 | 84,218 | 0.75 % |
| [unnamed] | 371 | 79,877 | 0.71 % |
| serde_json | 249 | 79,781 | 0.71 % |
| og_ai | 68 | 79,090 | 0.70 % |
| js_sys | 74 | 72,491 | 0.64 % |
| og_audio | 104 | 70,296 | 0.62 % |
| og_probe | 87 | 68,880 | 0.61 % |
| [wasm structure] | 1486 | 68,296 | 0.61 % |
| og_lanes | 101 | 52,433 | 0.47 % |
| indexmap | 125 | 45,187 | 0.40 % |
| og_geom | 60 | 43,124 | 0.38 % |
| og_host_web | 64 | 40,612 | 0.36 % |
| og_input | 70 | 32,256 | 0.29 % |
| robust | 7 | 28,622 | 0.25 % |
| wgpu_types | 260 | 26,973 | 0.24 % |
| og_i18n | 86 | 26,899 | 0.24 % |
| og_tilemap | 42 | 25,766 | 0.23 % |
| og_net | 56 | 24,659 | 0.22 % |
| glam | 70 | 24,483 | 0.22 % |
| spade | 43 | 23,611 | 0.21 % |
| [other] | 100 | 22,985 | 0.20 % |
| codespan_reporting | 58 | 21,943 | 0.19 % |
| miniz_oxide | 30 | 20,109 | 0.18 % |
| og_tween | 39 | 18,398 | 0.16 % |
| compiler_builtins | 37 | 18,099 | 0.16 % |
| og_timeline | 42 | 13,077 | 0.12 % |
| smallvec | 64 | 11,110 | 0.10 % |
| og_blob | 35 | 10,829 | 0.10 % |
| glow | 75 | 9,833 | 0.09 % |
| nalgebra | 14 | 9,090 | 0.08 % |
| libm | 13 | 9,030 | 0.08 % |
| crossbeam_channel | 28 | 8,004 | 0.07 % |
| naga_types | 53 | 6,912 | 0.06 % |
| arrayvec | 93 | 6,445 | 0.06 % |
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
| parking_lot_core | 11 | 1,755 | 0.02 % |
| unicode_segmentation | 8 | 1,600 | 0.01 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,401 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| wasm_bindgen_rayon | 3 | 1,091 | 0.01 % |
| web_sys | 26 | 1,008 | 0.01 % |
| once_cell | 5 | 958 | 0.01 % |
| fdeflate | 1 | 829 | 0.01 % |
| ena | 3 | 784 | 0.01 % |
| bit_vec | 9 | 736 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_set | 6 | 606 | 0.01 % |
| itoa | 3 | 465 | 0.00 % |
| wasm_sync | 4 | 433 | 0.00 % |
| __rustc | 5 | 327 | 0.00 % |
| lock_api | 2 | 283 | 0.00 % |
| simd_adler32 | 1 | 253 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| crossbeam_utils | 1 | 84 | 0.00 % |
| serde_arrays | 1 | 66 | 0.00 % |
