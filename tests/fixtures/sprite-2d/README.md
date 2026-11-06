# Sprite loading

One unchanged slime sheet preserves sprite-array and streamed-asset release coverage. The engine already owns rig2d tests; no dungeon gameplay is copied.

Readers: og-runtime/tests/sprite_arrays.rs and stream_release.rs.

Run through the named tests, not as a gameplay template. Asset-only fixtures
have no standalone playtest; their owning tests control the cook and assertions.
