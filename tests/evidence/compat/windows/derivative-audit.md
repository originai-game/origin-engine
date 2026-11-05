# Explicit shader derivatives: Windows follow-up

Runner: studio-win, NVIDIA RTX 5060. Derivative head: `54b532622`,
rebased onto main `226f11fbf`. No reference or tolerance changes.

Fresh `cargo test -p og-render --test tests shaders::`: 5 passed,
including explicit derivative policy and MSL/SPIR-V/HLSL translation.

Fresh Vulkan `cargo nextest run -p og-tests -p og-tests-cooked`:
526 run, 517 passed, 9 failed, 21 skipped, 553.636 seconds.
All nine failures were then reproduced without the derivative change,
on its exact parent `226f11fbf` (9 run, 9 failed, 28.175 seconds):

- a_visual_garment_is_simulated_on_the_gpu_from_its_skin: hem 169 to 177
- toon_rim_flash_and_dissolve: PBR 26 distinct buckets, toon 13
- the_surf_breaks_along_the_waterline: water-shore RMSE 0.01235
- ocean_scenes_match_their_references_and_read_as_a_sea: water-shore
- far_copies_draw_as_the_impostor: RMSE 0.01214
- the_grass_scene_matches_its_reference: RMSE 0.01017
- the_garment_matches_its_reference: RMSE 0.03238
- the_fluid_matches_its_reference: RMSE 0.01370
- meshlet_sway_matches_its_reference: RMSE 0.01218

The six distinct failing reference images are byte-identical between the
parent and derivative head. No assertion was disabled to get this result.
High-preset, post-adjust and skinned-hooks passed this Vulkan suite;
post-adjust has an existing Vulkan-specific reference.

A separate DX12 baseline on main `17edeb203`, before the derivative change,
also reproduced all four requested checks: toon 26/13, post-adjust
RMSE 0.01008, impostor 0.01007, and visual-garment 0.03373 (1.569% outliers).
This resolves the missing garment/impostor baseline from the original audit.

These are compatibility failures, not evidence of a derivative regression.
The toon failure is in the strict relative bucket-count assertion: Metal's
existing logs report PBR 29 / toon 13. The toon count agrees; why the PBR
row differs still requires investigation. Post-adjust is unchanged by the
derivative switch; its DX12 mismatch to the newer Metal shadow-tent reference
is not resolved by this audit. No backend reference is justified by these
observations alone.

Raw logs remain in `.amp/in/scratch/derivative-{vulkan,vulkan-baseline,main-dx12-control}.log`
on studio-win. The earlier full-frame comparison and DX12 suite are recorded
in the derivative commit message; they are inherited measurements, not rerun
here. Thread: https://ampcode.com/threads/T-01a1ab9d-3662-750e-b906-7429d51e0ebd
