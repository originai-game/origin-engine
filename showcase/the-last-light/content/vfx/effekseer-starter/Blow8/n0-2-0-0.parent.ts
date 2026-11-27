import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 1,
      "cycles": 9,
      "interval": 0.11670000106096268,
      "time": 0
    }
  ],
  "duration": 0.949999988079071,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    0,
    0
  ],
  "speed": [
    0,
    0
  ],
  "subEmitters": [
    {
      "emitter": ref("base/vfx/effekseer-starter/Blow8/n0-2-0-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
