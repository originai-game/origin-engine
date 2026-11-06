# Baked scene loading

One Cornell model and its existing lightmap bytes preserve baked-scene binding and release coverage. No frame is re-recorded.

Readers: og-runtime/tests/baked_scenes.rs.

Run through the named tests, not as a gameplay template. Asset-only fixtures
have no standalone playtest; their owning tests control the cook and assertions.
