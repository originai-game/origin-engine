import { file } from '@origingame/engine/author';
// Effekseer node 0 "FeatherBomb"
export default file.particles({
  "maxParticles": 17,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      1.0,
      1.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    12.0,
    12.0
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
        "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-1-0.emitter"
      }
    }
  ]
});
