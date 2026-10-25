# publish, after (shipping `wasm` profile; scripts/wasm-size-report.sh -n 60, then twiggy 0.8.0 top per crate)

```
publish.wasm: 18900795 B raw, 11604499 B after wasm-opt, 3283141 B brotli -q 11 (78.3 % of wasm_core_brotli_bytes, 4194304 B)
```

Per crate: code bodies and the data section, 11,506,453 B raw; "brotli alone" is each group compressed by itself (they sum to 3,440,744 B, more than the module's brotli)

| Crate | Functions | Raw bytes | Raw share | Brotli alone | Brotli share |
|---|---:|---:|---:|---:|---:|
| og_runtime | 1,006 | 1,146,888 | 9.97 % | 359,955 | 10.46 % |
| [data] | 1 | 1,383,578 | 12.02 % | 337,542 | 9.81 % |
| og_host_web (exports) | 381 | 1,127,027 | 9.79 % | 311,911 | 9.07 % |
| og_render | 659 | 851,361 | 7.40 % | 278,457 | 8.09 % |
| rapier3d | 348 | 841,637 | 7.31 % | 271,547 | 7.89 % |
| og_physics | 284 | 752,238 | 6.54 % | 220,009 | 6.39 % |
| parry3d | 982 | 756,522 | 6.57 % | 203,246 | 5.91 % |
| naga | 1,251 | 624,913 | 5.43 % | 186,428 | 5.42 % |
| wgpu | 374 | 360,117 | 3.13 % | 128,224 | 3.73 % |
| std | 1,565 | 425,554 | 3.70 % | 107,551 | 3.13 % |
| og_ui | 260 | 309,777 | 2.69 % | 106,021 | 3.08 % |
| wgpu_core | 945 | 322,333 | 2.80 % | 91,866 | 2.67 % |
| og_anim | 285 | 274,641 | 2.39 % | 82,672 | 2.40 % |
| wgpu_hal | 353 | 167,090 | 1.45 % | 61,489 | 1.79 % |
| og_nav | 92 | 149,351 | 1.30 % | 50,027 | 1.45 % |
| og_ecs | 465 | 181,344 | 1.58 % | 49,654 | 1.44 % |
| bincode | 78 | 167,931 | 1.46 % | 45,618 | 1.33 % |
| ttf_parser | 291 | 129,757 | 1.13 % | 42,783 | 1.24 % |
| rustybuzz | 294 | 108,883 | 0.95 % | 40,493 | 1.18 % |
| og_core | 530 | 158,593 | 1.38 % | 34,527 | 1.00 % |
| og_cook | 294 | 99,162 | 0.86 % | 31,831 | 0.93 % |
| og_ai | 74 | 89,835 | 0.78 % | 29,972 | 0.87 % |
| [unnamed] | 378 | 99,195 | 0.86 % | 28,766 | 0.84 % |
| og_audio | 104 | 74,072 | 0.64 % | 27,759 | 0.81 % |
| og_probe | 92 | 73,529 | 0.64 % | 27,010 | 0.79 % |
| serde_json | 253 | 82,417 | 0.72 % | 26,845 | 0.78 % |
| js_sys | 64 | 72,799 | 0.63 % | 26,313 | 0.76 % |
| og_host_web | 77 | 46,283 | 0.40 % | 16,991 | 0.49 % |
| og_geom | 59 | 42,381 | 0.37 % | 16,460 | 0.48 % |
| og_input | 69 | 56,133 | 0.49 % | 12,543 | 0.36 % |
| og_tilemap | 51 | 29,788 | 0.26 % | 12,221 | 0.36 % |
| glam | 83 | 31,981 | 0.28 % | 11,911 | 0.35 % |
| miniz_oxide | 31 | 26,019 | 0.23 % | 11,404 | 0.33 % |
| og_net | 61 | 31,229 | 0.27 % | 10,894 | 0.32 % |
| og_tween | 65 | 29,136 | 0.25 % | 10,892 | 0.32 % |
| og_i18n | 79 | 25,714 | 0.22 % | 10,410 | 0.30 % |
| wgpu_types | 285 | 29,725 | 0.26 % | 10,327 | 0.30 % |
| indexmap | 119 | 45,025 | 0.39 % | 9,590 | 0.28 % |
| codespan_reporting | 56 | 21,174 | 0.18 % | 8,442 | 0.25 % |
| spade | 43 | 23,493 | 0.20 % | 8,046 | 0.23 % |
| og_lanes | 68 | 31,922 | 0.28 % | 7,951 | 0.23 % |
| compiler_builtins | 19 | 19,291 | 0.17 % | 7,751 | 0.23 % |
| robust | 7 | 28,597 | 0.25 % | 7,723 | 0.22 % |
| [other] | 84 | 19,665 | 0.17 % | 7,309 | 0.21 % |
| og_timeline | 42 | 14,013 | 0.12 % | 5,840 | 0.17 % |
| nalgebra | 14 | 9,208 | 0.08 % | 3,999 | 0.12 % |
| dlmalloc | 15 | 12,487 | 0.11 % | 3,713 | 0.11 % |
| libm | 13 | 8,992 | 0.08 % | 3,327 | 0.10 % |
| og_blob | 28 | 12,936 | 0.11 % | 3,145 | 0.09 % |
| naga_types | 57 | 6,684 | 0.06 % | 3,144 | 0.09 % |
| smallvec | 59 | 10,276 | 0.09 % | 2,972 | 0.09 % |
| glow | 79 | 10,433 | 0.09 % | 2,838 | 0.08 % |
| arrayvec | 93 | 6,339 | 0.06 % | 2,631 | 0.08 % |
| unicode_bidi | 14 | 5,857 | 0.05 % | 2,365 | 0.07 % |
| wasm_bindgen | 47 | 5,274 | 0.05 % | 1,921 | 0.06 % |
| glamx | 5 | 3,438 | 0.03 % | 1,661 | 0.05 % |
| rustc_hash | 27 | 3,545 | 0.03 % | 1,354 | 0.04 % |
| parking_lot | 6 | 3,837 | 0.03 % | 1,224 | 0.04 % |
| serde_core | 23 | 3,048 | 0.03 % | 1,153 | 0.03 % |
| zmij | 1 | 2,148 | 0.02 % | 1,105 | 0.03 % |
| parking_lot_core | 9 | 1,613 | 0.01 % | 837 | 0.02 % |
| half | 21 | 1,831 | 0.02 % | 827 | 0.02 % |
| unicode_segmentation | 8 | 1,580 | 0.01 % | 815 | 0.02 % |
| foldhash | 4 | 3,050 | 0.03 % | 654 | 0.02 % |
| once_cell | 5 | 1,081 | 0.01 % | 631 | 0.02 % |
| ena | 4 | 1,627 | 0.01 % | 599 | 0.02 % |
| wgpu_naga_bridge | 3 | 1,393 | 0.01 % | 518 | 0.02 % |
| bitflags | 9 | 1,410 | 0.01 % | 479 | 0.01 % |
| raw_window_handle | 19 | 1,170 | 0.01 % | 415 | 0.01 % |
| web_sys | 27 | 980 | 0.01 % | 403 | 0.01 % |
| bit_vec | 8 | 628 | 0.01 % | 401 | 0.01 % |
| slotmap | 6 | 681 | 0.01 % | 400 | 0.01 % |
| bit_set | 5 | 486 | 0.00 % | 355 | 0.01 % |
| itoa | 3 | 457 | 0.00 % | 323 | 0.01 % |
| unicode_width | 1 | 336 | 0.00 % | 260 | 0.01 % |
| lock_api | 2 | 310 | 0.00 % | 236 | 0.01 % |
| __rustc | 5 | 325 | 0.00 % | 210 | 0.01 % |
| bytemuck | 5 | 324 | 0.00 % | 199 | 0.01 % |
| log | 3 | 321 | 0.00 % | 191 | 0.01 % |
| unicode_properties | 2 | 171 | 0.00 % | 150 | 0.00 % |
| serde_arrays | 1 | 64 | 0.00 % | 68 | 0.00 % |

Top 60 functions by raw bytes

| Raw bytes | Brotli alone | Crate | Function |
|---:|---:|---|---|
| 455,446 | 155,780 | og_render | `<og_render::Renderer>::render` |
| 358,502 | 123,581 | og_physics | `og_physics::schedule::{closure#0}` |
| 237,775 | 58,691 | og_host_web (exports) | `og_engine_create multivalue shim` |
| 192,382 | 71,684 | og_runtime | `og_runtime::extract::extract` |
| 125,491 | 35,407 | bincode | `bincode::internal::deserialize_seed::<core::marker::PhantomData<og_physics::dim3::sim::Sim>, bincode::config::WithOtherTrailing<bincode::config::DefaultOptio...` |
| 99,398 | 33,308 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::worker::run_worker` |
| 97,562 | 36,094 | wgpu | `<wgpu::backend::wgpu_core::CoreCommandEncoder as wgpu::dispatch::CommandEncoderInterface>::finish` |
| 83,595 | 27,843 | og_runtime | `og_runtime::modules::mount_back::{closure#0}` |
| 74,310 | 28,224 | rapier3d | `<rapier3d::pipeline::physics_pipeline::PhysicsPipeline>::detect_collisions` |
| 67,866 | 24,362 | js_sys | `js_sys::futures::future_to_promise::<og_host_web::canvas::_::__wasm_bindgen_generated_og_web_attach_canvas::{closure#0}>::{closure#0}::{closure#0}` |
| 55,400 | 21,976 | og_ui | `<og_ui::text::FontMetrics>::run_one` |
| 50,134 | 17,433 | og_host_web (exports) | `og_render_csg_geometry multivalue shim` |
| 48,476 | 9,003 | og_physics | `og_physics::gather::gather` |
| 41,387 | 16,574 | wgpu_hal | `<wgpu_hal::gles::Device>::create_pipeline` |
| 38,265 | 11,153 | og_anim | `og_anim::rig2d::parse_body` |
| 38,124 | 7,220 | og_input | `<og_input::sink::InputLayer as og_input::sink::InputSink>::attach` |
| 36,142 | 12,701 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass` |
| 35,608 | 13,903 | og_runtime | `og_runtime::timeline::decode` |
| 34,323 | 11,300 | og_runtime | `og_runtime::engine::call_rpc` |
| 33,324 | 6,313 | og_physics | `<og_physics::PhysicsState as og_ecs::state::StateBlock>::save` |
| 32,913 | 10,464 | og_host_web (exports) | `og_room_deliver` |
| 31,139 | 11,963 | naga | `<naga::valid::Validator>::validate_impl` |
| 30,730 | 11,175 | og_runtime | `og_runtime::timeline::install::{closure#5}` |
| 28,269 | 11,274 | og_runtime | `og_runtime::assets::spawn_scene` |
| 28,088 | 9,013 | naga | `<naga::front::wgsl::lower::Lowerer>::call` |
| 27,910 | 9,710 | naga | `<naga::valid::Validator>::validate_function` |
| 26,680 | 6,780 | naga | `<naga::back::glsl::writer::Writer<&mut alloc::string::String>>::write_expr` |
| 25,935 | 9,992 | naga | `<naga::front::wgsl::Frontend>::parse` |
| 24,457 | 3,864 | rapier3d | `core::slice::sort::unstable::quicksort::quicksort::<(&rapier3d::dynamics::island_manager::persistent::JointLinkKey, &(u32, u32)), <[(&rapier3d::dynamics::isl...` |
| 23,982 | 7,846 | og_runtime | `og_runtime::modules::load_style` |
| 23,736 | 5,230 | naga | `<naga::front::wgsl::error::Error>::as_parse_error` |
| 23,695 | 8,960 | og_render | `<og_render::terrain::TerrainRenderer>::sync` |
| 23,187 | 9,244 | og_probe | `<og_probe::probe::Probe>::execute` |
| 22,731 | 8,659 | og_anim | `<og_anim::rig2d::Rig2dAsset>::pose_of` |
| 22,699 | 8,978 | ttf_parser | `<ttf_parser::Face>::parse` |
| 22,657 | 9,497 | og_nav | `og_nav::system::run` |
| 22,573 | 8,505 | og_anim | `og_anim::system::install::{closure#0}` |
| 22,367 | 8,350 | og_render | `og_render::id_pass::render_ids_from` |
| 22,317 | 9,381 | wgpu | `<wgpu::backend::wgpu_core::CoreQueue as wgpu::dispatch::QueueInterface>::submit` |
| 21,931 | 9,279 | wgpu | `<wgpu::backend::wgpu_core::CoreDevice as wgpu::dispatch::DeviceInterface>::create_render_pipeline` |
| 21,465 | 8,526 | og_audio | `og_audio::system::install::{closure#0}` |
| 21,209 | 7,496 | rapier3d | `rapier3d::dynamics::solver::staged_island_solver::solve::solve_pass::{closure#2}` |
| 21,111 | 7,819 | og_runtime | `og_runtime::anim::load_animation` |
| 20,520 | 9,048 | og_runtime | `og_runtime::stream::fetch` |
| 20,445 | 7,316 | parry3d | `parry3d::query::sweep_toi::sweep_toi::sweep_time_of_impact` |
| 20,120 | 7,900 | og_ui | `<og_ui::ui::Ui>::edit` |
| 20,083 | 7,394 | og_nav | `<og_nav::mesh::NavMesh>::build_tile` |
| 19,799 | 8,092 | og_physics | `og_physics::dim3::sim::shape` |
| 19,771 | 8,170 | og_ai | `og_ai::system::install::{closure#0}` |
| 19,720 | 7,839 | og_host_web (exports) | `og_load_flipbook multivalue shim` |
| 19,583 | 6,151 | naga | `<naga::valid::Validator>::validate_block_impl` |
| 19,388 | 6,302 | og_host_web (exports) | `og_load_tilemap multivalue shim` |
| 19,273 | 7,823 | og_host_web (exports) | `og_anim_retarget multivalue shim` |
| 19,069 | 7,861 | og_ui | `og_ui::text::bidi_runs` |
| 18,920 | 6,432 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher>::contact_manifold_convex_co...` |
| 18,844 | 6,484 | parry3d | `<parry3d::query::default_query_dispatcher::DefaultQueryDispatcher as parry3d::query::query_dispatcher::PersistentQueryDispatcher<rapier3d::geometry::contact_...` |
| 18,539 | 6,319 | parry3d | `<core::marker::PhantomData<parry3d::shape::shared_shape::SharedShape> as serde_core::de::DeserializeSeed>::deserialize::<&mut bincode::de::Deserializer<binco...` |
| 18,244 | 7,631 | rapier3d | `<rapier3d::dynamics::soft_body::soft_body_set::soft_body_set::SoftBodySet>::split_cluster` |
| 18,032 | 6,410 | wgpu_hal | `<wgpu_hal::gles::Queue as wgpu_hal::dynamic::queue::DynQueue>::submit` |
| 17,912 | 5,918 | og_cook | `<og_cook::runtime::Bundle>::model` |

twiggy top, per crate (shallow bytes; names and producers left out): 11,604,222 B

| Crate | Items | Shallow bytes | Share |
|---|---:|---:|---:|
| [data] | 3476 | 1,383,576 | 11.92 % |
| og_runtime | 1006 | 1,149,627 | 9.91 % |
| og_host_web (exports) | 381 | 1,128,169 | 9.72 % |
| og_render | 659 | 853,129 | 7.35 % |
| rapier3d | 348 | 842,692 | 7.26 % |
| parry3d | 982 | 759,163 | 6.54 % |
| og_physics | 284 | 753,006 | 6.49 % |
| naga | 1251 | 627,924 | 5.41 % |
| std | 1565 | 429,441 | 3.70 % |
| wgpu | 374 | 361,048 | 3.11 % |
| wgpu_core | 945 | 324,684 | 2.80 % |
| og_ui | 260 | 310,513 | 2.68 % |
| og_anim | 285 | 275,424 | 2.37 % |
| og_ecs | 465 | 182,649 | 1.57 % |
| bincode | 78 | 168,156 | 1.45 % |
| wgpu_hal | 353 | 167,923 | 1.45 % |
| og_core | 530 | 159,919 | 1.38 % |
| og_nav | 92 | 149,610 | 1.29 % |
| ttf_parser | 291 | 130,509 | 1.12 % |
| rustybuzz | 294 | 109,609 | 0.94 % |
| [unnamed] | 378 | 100,182 | 0.86 % |
| og_cook | 294 | 99,844 | 0.86 % |
| og_ai | 74 | 90,040 | 0.78 % |
| serde_json | 253 | 83,072 | 0.72 % |
| og_audio | 104 | 74,346 | 0.64 % |
| og_probe | 92 | 73,775 | 0.64 % |
| js_sys | 64 | 72,938 | 0.63 % |
| [wasm structure] | 1451 | 62,566 | 0.54 % |
| og_input | 69 | 56,287 | 0.49 % |
| og_host_web | 77 | 46,484 | 0.40 % |
| indexmap | 119 | 45,313 | 0.39 % |
| og_geom | 59 | 42,557 | 0.37 % |
| glam | 83 | 32,203 | 0.28 % |
| og_lanes | 68 | 32,118 | 0.28 % |
| og_net | 61 | 31,392 | 0.27 % |
| wgpu_types | 285 | 30,350 | 0.26 % |
| og_tilemap | 51 | 29,920 | 0.26 % |
| og_tween | 65 | 29,307 | 0.25 % |
| robust | 7 | 28,621 | 0.25 % |
| miniz_oxide | 31 | 26,094 | 0.22 % |
| og_i18n | 79 | 25,915 | 0.22 % |
| spade | 43 | 23,602 | 0.20 % |
| codespan_reporting | 56 | 21,316 | 0.18 % |
| [other] | 89 | 19,909 | 0.17 % |
| compiler_builtins | 19 | 19,349 | 0.17 % |
| og_timeline | 42 | 14,125 | 0.12 % |
| og_blob | 28 | 13,008 | 0.11 % |
| dlmalloc | 15 | 12,530 | 0.11 % |
| glow | 79 | 10,625 | 0.09 % |
| smallvec | 59 | 10,419 | 0.09 % |
| nalgebra | 14 | 9,249 | 0.08 % |
| libm | 13 | 9,030 | 0.08 % |
| naga_types | 57 | 6,809 | 0.06 % |
| arrayvec | 93 | 6,535 | 0.06 % |
| unicode_bidi | 14 | 5,890 | 0.05 % |
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
| slotmap | 6 | 695 | 0.01 % |
| bit_vec | 8 | 645 | 0.01 % |
| bit_set | 5 | 498 | 0.00 % |
| itoa | 3 | 465 | 0.00 % |
| unicode_width | 1 | 339 | 0.00 % |
| __rustc | 5 | 336 | 0.00 % |
| bytemuck | 5 | 334 | 0.00 % |
| log | 3 | 329 | 0.00 % |
| lock_api | 2 | 315 | 0.00 % |
| unicode_properties | 2 | 176 | 0.00 % |
| serde_arrays | 1 | 66 | 0.00 % |
