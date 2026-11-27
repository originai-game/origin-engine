import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0.1.8 "SparkEmitter"
export default file.particles({
  "maxParticles": 124,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.4,
      1.4,
      1.4
    ]
  },
  "spread": 81.9505,
  "rate": 3960.0,
  "duration": 0.025,
  "looping": false,
  "lifetime": [
    0.1667,
    0.8333
  ],
  "speed": [
    59.988,
    83.5789
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-1-0-0-1-8-0-0.emitter"
      }
    }
  ],
  "gravity": [
    32.0174,
    -48.3191,
    48.5336
  ]
});
