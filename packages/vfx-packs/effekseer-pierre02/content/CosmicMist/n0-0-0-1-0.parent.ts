import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "BeyondGenerator"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      1.591,
      1.591
    ]
  },
  "spread": 26.5651,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 12,
      "interval": 0.0167
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    3.0,
    3.0
  ],
  "speed": [
    0.9546,
    2.8638
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre02/CosmicMist/n0-0-0-1-0.emitter"
      }
    }
  ]
});
