# Terrain tile streaming

One terrain and a tick-driven height sample replace the showcase. Resident and streamed runs must produce the same simulation hashes while tile residency changes. No gameplay, lake, character, or reference frame is copied.

Readers: packages/engine/tests/terrain-stream.test.ts.

Run through the named tests, not as a gameplay template. Asset-only fixtures
have no standalone playtest; their owning tests control the cook and assertions.
