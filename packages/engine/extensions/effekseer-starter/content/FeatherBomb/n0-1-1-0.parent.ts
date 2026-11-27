import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 13,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    12,
    12
  ],
  "looping": false,
  "maxParticles": 17,
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1,
      1,
      1
    ]
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
      "emitter": ref("base/vfx/effekseer-starter/FeatherBomb/n0-1-1-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
