# E4 break-and-restore

Each new test was run once against broken code (the break below), seen to fail, and run again after the code was restored. Every break was reverted; no test was weakened.

| Break (file: change) | Test run | Result |
|---|---|---|
| og-physics/src/lib.rs: the step uses `Time::dt` instead of the scaled dt | og-physics --test time_scale | both tests FAIL (the slow box falls as far as the normal one; the hit stop moves the body) |
| og-physics/src/lib.rs: a hit stop returns without carrying the touching pairs over | og-physics --test time_scale | a_hit_stop_freezes_bodies_keeps_contacts_and_resumes_exactly FAIL |
| og-render/src/sprites.rs: the per-layer sort key is dropped from the sort | og-tests --test sprites2d | sort_keys_order_sprites_within_a_layer_across_textures FAIL |
| og-render/src/sprites.rs: tile chunks are never culled | og-tests --test sprites2d | tile_layers_cull_chunks_draw_under_their_layer_repeat_and_animate FAIL (chunk count) |
| og-render/src/sprites.rs: a tile layer is placed after the sprites of its layer | og-tests --test sprites2d | tile_layers_... FAIL (the sprite is covered) |
| og-render/src/lib.rs: nearest textures get the linear sampler | og-tests --test sprites2d | a_nearest_texture_is_not_blended_between_texels, nine_slice_..., tile_layers_... FAIL |
| og-render/src/scene.rs: nine_slice ignores its insets | og-tests --test sprites2d | nine_slice_corners_keep_their_size_and_the_centre_stretches FAIL |
| og-render/shaders/sprite.wgsl: the outline width is ignored | og-tests --test sprites2d | an_outline_surrounds_text_only_when_asked FAIL |
| og-render/src/lib.rs: layered strips are not passed to the sprite batch | og-tests --test sprites2d | a_layered_trail_draws_in_the_sprite_order FAIL |
| og-tween/src/lib.rs: a target's byte offset ignores the fields before it | og-tween --test tweens | 3 FAIL (other fields move) |
| og-tween/src/lib.rs: `relative` ignored | og-tween --test tweens | delay_relative_loops_and_yoyo FAIL |
| og-tween/src/lib.rs: yoyo cycles play forwards | og-tween --test tweens | delay_relative_loops_and_yoyo FAIL |
| og-tween/src/lib.rs: a parallel group ends with its first child | og-tween --test tweens | sequences_hand_on_their_leftover_time_and_parallels_wait_for_the_last FAIL |
| og-tween/src/lib.rs: a sequence drops the time a child leaves over | og-tween --test tweens | sequences_... FAIL (the second child has not moved at tick 7) |
| og-tween/src/lib.rs: scaled tweens advance by Time::dt | og-tween --test tweens | time_scale_and_hit_stop_slow_and_hold_scaled_tweens_only FAIL |
| og-tween/src/codec.rs: the state block saves elapsed as 0 | og-tween --test tweens | a_snapshot_mid_tween_restores_running_tweens FAIL |
| og-tween/src/lib.rs: a tween whose target died keeps running | og-tween --test tweens | cancelled_and_orphaned_tweens_end_without_an_event FAIL |
| og-tilemap/src/tileset.rs: `reduce_blob` keeps every corner bit | og-tilemap --test autotile | 4 FAIL (47 masks, 3x3 block, edge, bitmask16) |
| og-tilemap/src/state.rs: a terrain `set` re-resolves only its own row of the neighbourhood | og-tilemap --test autotile | 3 FAIL (incl. a_fill_then_clear_restores_the_neighbourhood_exactly) |
| og-tilemap/src/state.rs: full-tile runs never merge vertically | og-tilemap --test collision | an_l_shape_is_two_rectangles, a_room_outline_is_four_rectangles FAIL |
| og-tilemap/src/state.rs: `encode` writes every cell, not only the differing ones | og-tilemap --test state | an_untouched_256x256_four_layer_map_snapshots_in_a_few_bytes FAIL |
| og-tilemap/src/state.rs: the x flip is not applied to the quad's region | og-tilemap --test chunks | quads_carry_their_cell_flips_and_animations FAIL |
| og-tilemap/src/map.rs: an object's rect y not negated | og-tilemap --test text | a_hand_written_map_reads_legends_rows_and_objects FAIL |
| og-cook/src/cook/tilemap.rs: Tiled flip bits dropped | og-cook --test tilemap | tilesets_and_maps_from_three_sources_cook_and_load FAIL |
| og-cook/src/cook/tilemap.rs: the compression check removed | og-cook --test tilemap | a_compressed_tiled_layer_and_a_missing_tileset_ogt_are_refused_by_name FAIL |
| og-cook/src/cook/tilemap.rs: LDtk solid values ignore the import settings | og-cook --test tilemap | import_settings_pick_the_ldtk_level_unit_and_solid_values FAIL |
| og-cook/src/runtime.rs: object properties dropped on load | og-cook --test tilemap | tilesets_and_maps_from_three_sources_cook_and_load FAIL |
| og-cook/src/cook/tilemap.rs: the missing sibling tileset .ogt check disabled | og-cook --test tilemap | a_compressed_tiled_layer_and_a_missing_tileset_ogt_are_refused_by_name FAIL |
| og-anim/src/flipbook.rs: grid frames numbered column-major | og-anim --test flipbook | the_sheet_reads_grid_and_named_frames_and_round_trips FAIL |
| og-anim/src/flipbook.rs: ping-pong repeats its end frames | og-anim --test flipbook | ping_pong_turns_at_the_ends_and_once_holds_and_finishes FAIL |
| og-anim/src/flipbook.rs: a once clip wraps | og-anim --test flipbook | ping_pong_..._once_holds_and_finishes FAIL |
| og-anim/src/flipbook.rs: events fire from the step already reached (twice per pass) | og-anim --test flipbook | a_looping_clip_..._fires_once_per_pass, ping_pong_... FAIL |
| og-anim/src/flipbook.rs: a seek keeps the old step (fires the frames it skipped) | og-anim --test flipbook | a_clip_change_restarts_a_seek_is_silent_and_speed_scales FAIL |
| og-anim/src/flipbook.rs: playback advances by Time::dt | og-anim --test flipbook | slow_motion_and_hit_stop_slow_and_hold_the_frame FAIL |
| og-anim/src/flipbook.rs: the state block restores no playback | og-anim --test flipbook | a_snapshot_keeps_the_playback_and_the_transcript_is_the_same FAIL |
| og-cook/src/cook/flipbook.rs: the texture's size is not filled in | og-cook --test flipbook | aseprite_and_texturepacker_atlases_cook_into_sheets FAIL |
| og-cook/src/cook/flipbook.rs: Aseprite hash frames in string order | og-cook --test flipbook | aseprite_and_... FAIL ("hero 10" lands at index 2) |
| og-cook/src/cook/flipbook.rs: a reverse tag plays forwards | og-cook --test flipbook | aseprite_and_... FAIL |
| og-cook/src/cook/flipbook.rs: TexturePacker pivots keep their top-left origin | og-cook --test flipbook | aseprite_and_... FAIL |
| og-cook/src/cook/flipbook.rs: a tag replaces the sheet's own clip of that name | og-cook --test flipbook | aseprite_and_... FAIL |
| og-cook/src/cook/mod.rs: the flipbook stamp leaves out its texture and atlas | og-cook --test flipbook | aseprite_and_... FAIL (the edited atlas is not recooked) |
| og-cook/src/cook/texture.rs: `kind pixel` goes through Basis Universal | og-cook --test flipbook | aseprite_and_... FAIL (not RGBA8, mips) |
