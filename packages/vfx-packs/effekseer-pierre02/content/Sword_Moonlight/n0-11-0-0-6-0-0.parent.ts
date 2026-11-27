import { file } from '@origingame/engine/author';
// Effekseer node 0.11.0.0.6 "EmberGenerator"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.375,
      2.25,
      3.375
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 12,
      "cycles": 4,
      "interval": 0.3
    }
  ],
  "duration": 0.9167,
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-0-0-6-0-0.emitter"
      }
    }
  ]
});
