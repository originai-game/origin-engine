# Sprite loading

One unchanged slime sheet preserves sprite-array and streamed-asset release coverage. The engine already owns rig2d tests; no dungeon gameplay is copied.

Readers: packages/engine/tests/web/device.ts and export.ts.

Run through the named tests, not as a gameplay template. Asset-only fixtures
have no standalone playtest; their owning tests control the cook and assertions.
