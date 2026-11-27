import { file } from '@origingame/engine/author';
// Effekseer node 0.8.0.0 "MainFireGenerator"
export default file.particles({
  "maxParticles": 54,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 120.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    0.05,
    0.65
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Sacredlight/n0-8-0-0-1-0-2-0-1.emitter"
      }
    }
  ]
});
