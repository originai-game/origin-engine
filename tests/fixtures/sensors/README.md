# sensors

A pressure plate (a sensor collider) and three kinematic crates walked
across it on fixed paths (`src/main.ts`'s `path`): one crate spawned before
the plate, so its pairs name the sensor second, and two after. A solid
floor lies just under the plate.

What it checks (`tests/plate.og.ts`, `og test tests/fixtures/sensors`):

- every tick, `og.physics.sensorContents` lists the crates the last step
  put over the plate, in entity order and each once, and equals what
  og.SensorEntered and og.SensorExited added up to; every crate enters and
  leaves once; two crates stand on it at once; two runs end on one hash.

The contents across a restore and a hit stop are og-physics'
`tests/sensors.rs` (a game cannot keep a save's text in world state).

Budget: under 10 s warm. No frames: nothing here is drawn.
