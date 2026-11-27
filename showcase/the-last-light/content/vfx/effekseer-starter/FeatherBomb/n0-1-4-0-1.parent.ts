import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 26,
      "cycles": 2,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "duration": 0.0333000011742115,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 65,
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
      "emitter": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-4-0-1.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
