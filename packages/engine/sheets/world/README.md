# World labels

The scene, topdown, states and marks pages show the world labels over the
approved mockup's sample art (`assets/art`, sheet only): the camera looks
down over an empty ground with the art on an unlit plane far behind, each
label anchored at the point along the ray through its pixel in the mockup's
frame and each ground mark where that ray meets the ground
(`src/fixtures.ts`, `src/main.ts`). The phones take the mockup's phone
layouts on the scene page and the middle of the desktop layout elsewhere.

The camera looks down at 40°, so a telegraph, ring or ghost lies at that
tilt, not the mockup's flatter one. The components' own fixtures and
mounting are covered by `tests/ui-world.test.ts`.
