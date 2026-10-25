# lanes2d, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
lanes2d.wasm: 17387376 B raw, 10385027 B after wasm-opt, 2965468 B brotli -q 11 (70.7 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 10,283,115 B raw; "brotli alone" is each group compressed by itself (they sum to 3,093,478 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| [data] | 1 | 1,475,127 | 14.35 % | 320,861 | 10.37 % |
| og_runtime | 675 | 928,358 | 9.03 % | 313,352 | 10.13 % |
| og_host_web (exports) | 383 | 1,010,218 | 9.82 % | 292,208 | 9.45 % |
| rapier2d | 358 | 824,746 | 8.02 % | 267,276 | 8.64 % |
| og_render | 603 | 728,305 | 7.08 % | 234,611 | 7.58 % |
| naga | 1,183 | 599,303 | 5.83 % | 179,999 | 5.82 % |
| parry2d | 797 | 460,454 | 4.48 % | 130,611 | 4.22 % |
| wgpu | 384 | 336,655 | 3.27 % | 122,770 | 3.97 % |
| og_ui | 228 | 273,609 | 2.66 % | 96,124 | 3.11 % |
| og_anim | 254 | 297,723 | 2.90 % | 94,334 | 3.05 % |
| og_physics | 176 | 316,682 | 3.08 % | 92,831 | 3.00 % |
| wgpu_core | 922 | 319,094 | 3.10 % | 90,541 | 2.93 % |
| std | 1,373 | 285,455 | 2.78 % | 81,125 | 2.62 % |
| wgpu_hal | 353 | 161,162 | 1.57 % | 59,584 | 1.93 % |
| bincode | 72 | 176,500 | 1.72 % | 48,778 | 1.58 % |
| og_ecs | 461 | 174,103 | 1.69 % | 48,098 | 1.55 % |
| og_core | 682 | 196,217 | 1.91 % | 44,233 | 1.43 % |
| ttf_parser | 294 | 129,823 | 1.26 % | 42,855 | 1.39 % |
| rustybuzz | 307 | 109,333 | 1.06 % | 40,422 | 1.31 % |
| og_nav | 71 | 105,134 | 1.02 % | 38,026 | 1.23 % |
| og_audio | 104 | 70,036 | 0.68 % | 27,538 | 0.89 % |
| og_probe | 98 | 72,360 | 0.70 % | 26,668 | 0.86 % |
| og_ai | 65 | 77,936 | 0.76 % | 26,595 | 0.86 % |
| js_sys | 74 | 73,386 | 0.71 % | 26,500 | 0.86 % |
| og_cook | 290 | 78,641 | 0.76 % | 26,199 | 0.85 % |
| serde_json | 250 | 79,159 | 0.77 % | 25,982 | 0.84 % |
| rayon | 36 | 81,457 | 0.79 % | 25,738 | 0.83 % |
| [unnamed] | 370 | 79,054 | 0.77 % | 24,693 | 0.80 % |
| rayon_core | 278 | 136,839 | 1.33 % | 21,244 | 0.69 % |
| og_geom | 60 | 42,945 | 0.42 % | 16,715 | 0.54 % |
| og_host_web | 64 | 39,815 | 0.39 % | 14,837 | 0.48 % |
| og_lanes | 92 | 48,475 | 0.47 % | 14,557 | 0.47 % |
| og_tilemap | 42 | 25,584 | 0.25 % | 10,649 | 0.34 % |
| og_i18n | 87 | 26,690 | 0.26 % | 10,614 | 0.34 % |
| indexmap | 123 | 42,561 | 0.41 % | 9,809 | 0.32 % |
| glam | 68 | 24,659 | 0.24 % | 9,573 | 0.31 % |
| wgpu_types | 260 | 26,406 | 0.26 % | 9,455 | 0.31 % |
| og_net | 56 | 24,511 | 0.24 % | 9,127 | 0.30 % |
| miniz_oxide | 30 | 20,037 | 0.19 % | 8,861 | 0.29 % |
| codespan_reporting | 59 | 21,801 | 0.21 % | 8,763 | 0.28 % |
| [other] | 95 | 22,550 | 0.22 % | 8,389 | 0.27 % |
| og_input | 71 | 32,137 | 0.31 % | 8,357 | 0.27 % |
| spade | 43 | 23,650 | 0.23 % | 8,075 | 0.26 % |
| compiler_builtins | 37 | 17,989 | 0.17 % | 7,759 | 0.25 % |
| robust | 7 | 28,597 | 0.28 % | 7,695 | 0.25 % |
| og_tween | 39 | 18,295 | 0.18 % | 7,275 | 0.24 % |
| og_timeline | 42 | 12,965 | 0.13 % | 5,641 | 0.18 % |
| libm | 13 | 8,992 | 0.09 % | 3,329 | 0.11 % |
| og_blob | 34 | 10,719 | 0.10 % | 3,316 | 0.11 % |
| naga_types | 54 | 6,962 | 0.07 % | 3,234 | 0.10 % |
| smallvec | 64 | 10,957 | 0.11 % | 3,041 | 0.10 % |
| arrayvec | 98 | 7,046 | 0.07 % | 2,953 | 0.10 % |
| glow | 75 | 9,649 | 0.09 % | 2,641 | 0.09 % |
| crossbeam_channel | 28 | 7,933 | 0.08 % | 2,484 | 0.08 % |
| nalgebra | 9 | 5,464 | 0.05 % | 2,399 | 0.08 % |
| dlmalloc | 22 | 5,813 | 0.06 % | 2,381 | 0.08 % |
| unicode_bidi | 12 | 5,820 | 0.06 % | 2,330 | 0.08 % |
| wasm_bindgen | 58 | 5,210 | 0.05 % | 1,858 | 0.06 % |
| crossbeam_epoch | 27 | 4,196 | 0.04 % | 1,509 | 0.05 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,337 | 0.04 % |
| parking_lot | 6 | 3,801 | 0.04 % | 1,288 | 0.04 % |
| serde_core | 24 | 3,326 | 0.03 % | 1,226 | 0.04 % |
| crossbeam_deque | 19 | 3,781 | 0.04 % | 1,225 | 0.04 % |
| zmij | 1 | 2,148 | 0.02 % | 1,120 | 0.04 % |
| parking_lot_core | 11 | 1,729 | 0.02 % | 943 | 0.03 % |
| half | 21 | 1,831 | 0.02 % | 841 | 0.03 % |
| unicode_segmentation | 8 | 1,580 | 0.02 % | 812 | 0.03 % |
| foldhash | 4 | 3,041 | 0.03 % | 654 | 0.02 % |
| once_cell | 5 | 945 | 0.01 % | 530 | 0.02 % |
| glamx | 3 | 1,093 | 0.01 % | 524 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 515 | 0.02 % |
| wasm_bindgen_rayon | 3 | 1,084 | 0.01 % | 487 | 0.02 % |
| bitflags | 9 | 1,410 | 0.01 % | 480 | 0.02 % |
| bit_vec | 9 | 717 | 0.01 % | 431 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 413 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 404 | 0.01 % |
| bit_set | 6 | 590 | 0.01 % | 396 | 0.01 % |
| fdeflate | 1 | 826 | 0.01 % | 394 | 0.01 % |
| web_sys | 26 | 956 | 0.01 % | 393 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 325 | 0.01 % |
| wasm_sync | 4 | 424 | 0.00 % | 247 | 0.01 % |
| lock_api | 2 | 278 | 0.00 % | 233 | 0.01 % |
| __rustc | 5 | 316 | 0.00 % | 232 | 0.01 % |
| simd_adler32 | 1 | 250 | 0.00 % | 174 | 0.01 % |
| bytemuck | 5 | 174 | 0.00 % | 173 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 152 | 0.00 % |
| crossbeam_utils | 1 | 82 | 0.00 % | 84 | 0.00 % |
| serde_arrays | 1 | 19 | 0.00 % | 23 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 409,992 | 136,760 | og_render | `<og_render::Renderer>::render` |
| 194,313 | 50,734 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 164,235 | 62,820 | og_runtime | `og_runtime::extract::extract` |
| 159,762 | 57,776 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::step_inner` |
| 121,297 | 34,159 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim2::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 94,294 | 33,826 | og_physics | `og_physics::dim2::sim::step_here` |
| 89,463 | 34,217 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 79,336 | 27,631 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 66,361 | 23,672 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 61,568 | 19,793 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 54,165 | 21,621 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 51,592 | 18,498 | og_anim | `og_anim::system::install::{closure#0}` |
| 51,310 | 17,599 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 49,826 | 9,248 | og_physics | `og_physics::gather::gather` |
| 40,960 | 15,357 | rapier2d | `<rapier2d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 40,141 | 16,141 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 37,980 | 11,300 | og_anim | `og_anim::rig2d::parse_body` |
| 37,348 | 14,326 | og_runtime | `og_runtime::timeline::decode` |
| 29,885 | 11,191 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 29,662 | 6,103 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 29,442 | 10,159 | rapier2d | `rapier2d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 29,113 | 11,809 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,091 | 11,236 | naga | `<naga::valid::Validator>::validate_impl` |
| 27,361 | 8,718 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,175 | 9,814 | og_runtime | `og_runtime::engine::call_rpc` |
| 26,513 | 11,032 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 26,278 | 6,673 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 25,761 | 9,987 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 24,576 | 8,391 | og_host_web (exports) | `og_room_deliver` |
| 24,464 | 3,870 | rapier2d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier2d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier2d::dynamics::isl...` |
| 23,622 | 8,962 | og_audio | `og_audio::system::run` |
| 23,558 | 7,694 | og_runtime | `og_runtime::modules::load_style` |
| 23,500 | 5,196 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 22,699 | 8,990 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,646 | 8,291 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,632 | 8,462 | og_runtime | `og_runtime::anim::load_animation` |
| 22,226 | 8,957 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,146 | 9,357 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,970 | 9,370 | og_nav | `og_nav::system::install::{closure#0}` |
| 21,794 | 9,086 | rapier2d | `<rapier2d::geometry::narrow_phase::NarrowPhase>::compute_contacts::{closure#3}` |
| 21,570 | 8,796 | naga | `<naga::valid::Validator>::validate_function` |
| 21,360 | 9,033 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,069 | 8,202 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,171 | 7,432 | og_host_web (exports) | `og_anim_graph multivalue shim` |
| 19,647 | 7,042 | og_physics | `og_physics::schedule::{closure#0}` |
| 19,626 | 7,330 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,482 | 8,638 | rayon | `rayon::iter::plumbing::bridge_producer_consumer::helper::<rayon::iter::enumerate::EnumerateProducer<rayon::slice::IterMutProducer<rapier2d::dynamics::solver:...` |
| 18,969 | 7,820 | og_ui | `og_ui::text::bidi_runs` |
| 18,861 | 6,070 | og_cook | `<og_cook::runtime::Bundle>::model` |
| 18,855 | 7,483 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 18,780 | 7,808 | og_ai | `og_ai::system::install::{closure#0}` |
| 18,076 | 7,611 | og_render | `og_render::id_pass::render_ids_from` |
| 17,653 | 6,295 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,602 | 5,777 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 17,506 | 7,335 | rapier2d | `<rapier2d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |
| 17,346 | 7,146 | og_runtime | `og_runtime::save::apply_keeping` |
| 16,816 | 5,747 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 16,789 | 7,148 | wgpu_hal | `<wgpu_hal::gles::web::Instance as wgpu_hal::dynamic::instance::DynInstance>::enumerate_adapters` |
| 16,643 | 7,565 | og_runtime | `og_runtime::stream::fetch` |
| 16,632 | 6,436 | rapier2d | `<rapier2d::pipeline::physics_world::PhysicsWorld>::remove_body` |

twiggy top, per crate (shallow bytes; names and producers left out): 10,384,757 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3 | 1,475,126 | 14.20 % |
| og_host_web (exports) | 383 | 1,011,373 | 9.74 % |
| og_runtime | 675 | 930,142 | 8.96 % |
| rapier2d | 358 | 825,817 | 7.95 % |
| og_render | 603 | 729,896 | 7.03 % |
| naga | 1183 | 602,162 | 5.80 % |
| parry2d | 797 | 462,553 | 4.45 % |
| wgpu | 384 | 337,604 | 3.25 % |
| wgpu_core | 922 | 321,391 | 3.09 % |
| og_physics | 176 | 317,143 | 3.05 % |
| og_anim | 254 | 298,423 | 2.87 % |
| std | 1373 | 288,731 | 2.78 % |
| og_ui | 228 | 274,249 | 2.64 % |
| og_core | 682 | 197,917 | 1.91 % |
| bincode | 72 | 176,709 | 1.70 % |
| og_ecs | 461 | 175,400 | 1.69 % |
| wgpu_hal | 353 | 161,995 | 1.56 % |
| rayon_core | 278 | 137,632 | 1.33 % |
| ttf_parser | 294 | 130,581 | 1.26 % |
| rustybuzz | 307 | 110,082 | 1.06 % |
| og_nav | 71 | 105,330 | 1.01 % |
| rayon | 36 | 81,563 | 0.79 % |
| [unnamed] | 370 | 79,994 | 0.77 % |
| serde_json | 250 | 79,794 | 0.77 % |
| og_cook | 290 | 79,298 | 0.76 % |
| og_ai | 65 | 78,112 | 0.75 % |
| js_sys | 74 | 73,548 | 0.71 % |
| og_probe | 98 | 72,622 | 0.70 % |
| og_audio | 104 | 70,299 | 0.68 % |
| [wasm structure] | 1495 | 68,097 | 0.66 % |
| og_lanes | 92 | 48,739 | 0.47 % |
| og_geom | 60 | 43,124 | 0.42 % |
| indexmap | 123 | 42,854 | 0.41 % |
| og_host_web | 64 | 39,978 | 0.38 % |
| og_input | 71 | 32,294 | 0.31 % |
| robust | 7 | 28,622 | 0.28 % |
| wgpu_types | 260 | 26,973 | 0.26 % |
| og_i18n | 87 | 26,910 | 0.26 % |
| og_tilemap | 42 | 25,689 | 0.25 % |
| glam | 68 | 24,836 | 0.24 % |
| og_net | 56 | 24,659 | 0.24 % |
| spade | 43 | 23,763 | 0.23 % |
| [other] | 100 | 22,814 | 0.22 % |
| codespan_reporting | 59 | 21,949 | 0.21 % |
| miniz_oxide | 30 | 20,109 | 0.19 % |
| og_tween | 39 | 18,396 | 0.18 % |
| compiler_builtins | 37 | 18,101 | 0.17 % |
| og_timeline | 42 | 13,075 | 0.13 % |
| smallvec | 64 | 11,110 | 0.11 % |
| og_blob | 34 | 10,804 | 0.10 % |
| glow | 75 | 9,833 | 0.09 % |
| libm | 13 | 9,030 | 0.09 % |
| crossbeam_channel | 28 | 8,004 | 0.08 % |
| arrayvec | 98 | 7,254 | 0.07 % |
| naga_types | 54 | 7,082 | 0.07 % |
| dlmalloc | 22 | 5,866 | 0.06 % |
| unicode_bidi | 12 | 5,849 | 0.06 % |
| nalgebra | 9 | 5,489 | 0.05 % |
| wasm_bindgen | 58 | 5,341 | 0.05 % |
| crossbeam_epoch | 27 | 4,261 | 0.04 % |
| crossbeam_deque | 19 | 3,827 | 0.04 % |
| parking_lot | 6 | 3,819 | 0.04 % |
| rustc_hash | 27 | 3,605 | 0.03 % |
| serde_core | 24 | 3,382 | 0.03 % |
| foldhash | 4 | 3,053 | 0.03 % |
| zmij | 1 | 2,152 | 0.02 % |
| half | 21 | 1,880 | 0.02 % |
| parking_lot_core | 11 | 1,755 | 0.02 % |
| unicode_segmentation | 8 | 1,600 | 0.02 % |
| bitflags | 9 | 1,435 | 0.01 % |
| wgpu_naga_bridge | 3 | 1,401 | 0.01 % |
| raw_window_handle | 19 | 1,209 | 0.01 % |
| glamx | 3 | 1,101 | 0.01 % |
| wasm_bindgen_rayon | 3 | 1,091 | 0.01 % |
| web_sys | 26 | 1,008 | 0.01 % |
| once_cell | 5 | 958 | 0.01 % |
| fdeflate | 1 | 829 | 0.01 % |
| bit_vec | 9 | 736 | 0.01 % |
| slotmap | 6 | 695 | 0.01 % |
| bit_set | 6 | 604 | 0.01 % |
| itoa | 3 | 465 | 0.00 % |
| wasm_sync | 4 | 433 | 0.00 % |
| __rustc | 5 | 327 | 0.00 % |
| lock_api | 2 | 283 | 0.00 % |
| simd_adler32 | 1 | 253 | 0.00 % |
| bytemuck | 5 | 184 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| crossbeam_utils | 1 | 84 | 0.00 % |
| serde_arrays | 1 | 21 | 0.00 % |
