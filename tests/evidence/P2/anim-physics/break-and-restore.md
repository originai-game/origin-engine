# Break and restore: the animation and physics round (phase 2)

Every new test was broken once by changing the code it covers, seen to
fail, and the code restored (the test then passed again). Nothing in a test
was changed to make it pass.

| Test | Broken how | What failed |
|---|---|---|
| `og-anim` `tests/retarget.rs` `a_walk_on_longer_legs_keeps_its_planted_feet_on_the_ground` | the foot IK goal without the leg ratio (`(foot - rest)` instead of `(foot - rest) * k`) | "planted feet within 2 mm of the ground: 0.0085565" (slide 0.0325 m) |
| `og-anim` `tests/retarget.rs` `a_wave_turns_the_a_posed_arm_as_the_walker_turns_its_t_posed_one` | no rest correction (`Quat::IDENTITY` for every bone) | "the arms point alike (worst 0.76536655)" |
| `og-anim` `tests/retarget.rs` the same walk, `foot_ik: false` | (the test itself asserts that forward kinematics alone misses: 0.0115 m) | — |
| `og-cook` `tests/retarget.rs` `library_clips_are_retargeted_into_each_characters_anim_with_planted_feet` | the cook's retargeting with `foot_ik: false` whatever the settings say | "models/strider.glb: planted feet on the ground (0.028791294 m)" |
| `og-cook` `tests/retarget.rs` `an_edit_of_the_rig_or_the_library_recooks_and_a_bad_rig_names_its_line` | the rig and library hashes left out of the `#anim` cook stamp | the assertion that an edited rig file re-cooks `models/strider.glb#anim` (line 200) |
| `og-physics` `tests/ragdoll.rs` `a_2d_ragdoll_follows_goes_limp_on_hinges_in_its_plane_and_recovers` | the 2D bone hinges built without their limit | "the knee bent 3.03 rad past its 0.9 limit" (with the limit: 1.12 at most, landing under the body's weight) |
| `og-physics` `tests/self_collision.rs` `a_sheet_that_falls_and_folds_never_passes_through_itself` | the spatial hash searching only a point's own cell (not the 26 around it) | "two layers came within 0.0445386 × the thickness: one passed through the other" |
| `og-physics` `tests/fluid_groups.rs` `a_heavy_fluid_poured_on_a_light_one_sinks_under_it_and_they_layer` | every particle of a group weighing 1 whatever its fluid's density | "the heavy fluid sank under the light one (heavy 0.79051924, light 0.54000825)" |
| `og-physics` `tests/fluid_groups.rs` `with_mixing_the_two_fluids_blend_into_one` | the mixing rate 0 | "the mix evened out (±0.5)" |
