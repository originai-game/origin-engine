import { file } from '@origingame/engine/author';
// Effekseer node 2 "光の親"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "box",
    "halfExtents": [
      7.0,
      7.0,
      7.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0167
    }
  ],
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.0833,
    0.25
  ],
  "speed": [
    18.1865,
    72.7461
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-tktk01/Cure2/n2-0.emitter"
      }
    }
  ]
});
