import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "GateOfCalve"
export default file.particles({
  "maxParticles": 113,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      12.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 90,
      "interval": 0.0167
    }
  ],
  "duration": 1.5,
  "looping": false,
  "lifetime": [
    10.0,
    10.0
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
        "$ogRef": "base/vfx/effekseer-pierre02/GateOfCalve/n0-0-0-3.emitter"
      }
    }
  ]
});
