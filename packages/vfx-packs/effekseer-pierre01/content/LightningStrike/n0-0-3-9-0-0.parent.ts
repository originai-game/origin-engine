import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3.9 "SparkEmitter"
export default file.particles({
  "maxParticles": 50,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      2.0,
      2.0
    ]
  },
  "spread": 81.9505,
  "rate": 600.0,
  "duration": 0.0667,
  "looping": false,
  "lifetime": [
    0.1667,
    0.8333
  ],
  "speed": [
    85.6971,
    119.3985
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre01/LightningStrike/n0-0-3-9-0-0.emitter"
      }
    }
  ],
  "gravity": [
    0.0,
    -108.0,
    0.0
  ]
});
