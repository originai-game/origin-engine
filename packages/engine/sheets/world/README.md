# World labels

The scene, topdown, states and marks pages show the world labels over the
approved mockup's sample art (`assets/art`, sheet only): the camera looks
down over an empty ground with the art on an unlit plane far behind, each
label anchored at the point along the ray through its pixel in the mockup's
frame and each ground mark where that ray meets the ground
(`src/fixtures.ts`, `src/main.ts`). The phones take the mockup's phone
layouts on the scene page and the middle of the desktop layout elsewhere.

The scene camera looks down at 12°, the other pages at 30°. The painted
player is not an engine entity: `PLAYER_SILHOUETTE` records its cloak,
body and sword bounds in the source art. The sheet maps those bounds
through the art's cover transform into an explicit upright player-reserve
proxy at the synthetic label depth. Resizing recomputes the proxy; state
boards and topdown do not reserve a player. This conservative projected
box is fixture geometry, not a reconstruction of a real player's size.

Waypoints opt into that reserve and keep their own chevrons without an
engine arrow. Prompts avoid the telegraphs' projected danger bounds. The
components' own fixtures and mounting are covered by `tests/ui-world.test.ts`.
