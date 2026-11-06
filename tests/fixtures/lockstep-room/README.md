# Lockstep inputs

Two entities carry the two seats' `og.PlayerInput`. Each seat submits a
distinct tick-driven input; the simulation accumulates both into `Duel`.
There is no gameplay, rendering, audio or asset dependency.

`tests/inputs.og.ts` runs two peers through the real relay, checks both
inputs reached the world, compares their resource reads and final network
hashes, and rejects any desync. The engine compares hashes every ten ticks.

`packages/engine/tests/rooms.test.ts` additionally checks rollback under
latency, jitter and packet loss, quick play, stalled-seat drop/rejoin and
room playtest `then` steps. `web-rooms.test.ts` checks Worker-hosted pages
joining through Play links. Their command/resource names stay compatible
with the former arena fixture; coin gameplay is not part of these tests.

Run with `og test tests/fixtures/lockstep-room`. Warm budget: under 10 s.
