# E2 break-and-restore

Each new test was run once against broken code (the break below), seen to fail, and run again after the code was restored. Every break was reverted; no test was weakened.

| Break (file: change) | Test run | Result |
|---|---|---|
| og-render/shaders/overlay.wgsl: the fragment ignores the clip rectangle | og-tests --test gfx overlay tests | the_overlay_is_drawn_after_post_untonemapped_and_clipped and overlay_matches_its_reference FAIL |
| og-render/shaders/overlay.wgsl: the sRGB encode flag inverted | og-tests --test gfx overlay tests | all three overlay tests FAIL |
| og-ui/src/draw.rs: quads and images emitted without their clip index | og-ui --test gfx a_scroll_container_clips_everything_it_draws | FAIL (row colour outside the list) |
| og-api/src/runtime/extract.rs: queued sprites and text drawn whatever tick queued them (the bug) | packages/engine bun test tests/render.test.ts (stale sprite) | FAIL: the sprite of tick 1 still covers the centre at tick 5 |
| og-input/src/resource.rs: Gamepad::end_tick keeps the edges | og-input --test pads | pads_and_touches_over_a_scripted_sequence FAIL (A's press edge outlives its tick) |
| og-input/src/resource.rs + sink.rs: touches 1 to 4 also drive og.Input | og-input --test pads | pads_and_touches_over_a_scripted_sequence FAIL (og.Input's pointer follows the second touch) |
| og-ui/src/input.rs: GamepadFrame::from_gamepad drops the pressed buttons | og test templates/ui-kit (gamepad) | FAIL (the d-pad never moves focus; the stick alone still does) |
| og-ui/src/ui.rs: touch controls ignore pointers 1 to 4 | og test templates/ui-kit (multitouch) | FAIL (Hit never fires while the stick is held) |
| og-api/src/runtime/fx.rs: PostEffects not read; screen sprites drawn in the world; Decal rows skipped (one build) | packages/engine bun test tests/render.test.ts | FAIL: screen-space sprites, PostEffects (flash, vignette), Decal, camera shake; the 2D, strip, material and stale-sprite tests still pass, as they must |
| og-cook/src/fx.rs: the emitter's drag line read as 0 | og-cook --test fx | a_cube_and_an_emitter_cook_and_load FAIL |
| og-render/src/post.rs: the effect passes declare only their own inputs (the E1 graph; the fix declares the bound lindepth and gbuffer) | og-tests --test effects every_effect_pass_together_under_taa | FAIL (wgpu validation: a graph transient used as a resource and a colour target in one pass) |
| og-render/shaders (pbr, shadow, material): the unfixed shaders, an alpha cut-off discard followed by the dissolve's (written first, then fixed) | og-tests --test effects dissolved_pixels_leave_no_trace_in_the_prepass | FAIL on metal: a fully dissolved sphere left 3293 pixels (a black disc striped with background). Probes: an unconditional discard at the top passed; the dissolve discard before the cut-off passed; the cut-off removed passed. A second discard in one entry point does not take effect on Metal |

