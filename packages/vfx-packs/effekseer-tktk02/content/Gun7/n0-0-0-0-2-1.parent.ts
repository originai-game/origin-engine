import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "ｉｎｔｅｒｖａｌ"
export default file.particles({
  "maxParticles": 40,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.1,
      0.1,
      0.1
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 32,
      "interval": 0.0333
    }
  ],
  "duration": 1.05,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-tktk02/Gun7/n0-0-0-0-2-1.emitter"
      }
    }
  ]
});
