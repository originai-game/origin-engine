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
