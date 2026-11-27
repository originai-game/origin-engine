import { file } from '@origingame/engine/author';
// Effekseer node 0.11.1 "MainFireGenerator"
export default file.particles({
  "maxParticles": 40,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 300.0,
  "duration": 0.1067,
  "looping": false,
  "lifetime": [
    0.0167,
    0.2167
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-1-1-0-2.emitter"
      }
    }
  ]
});
