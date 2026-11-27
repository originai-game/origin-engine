import { file } from '@origingame/engine/author';
// Effekseer node 0.11.1.1 "FireGenerator"
export default file.particles({
  "maxParticles": 707,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 320.0,
  "duration": 1.0,
  "looping": true,
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-11-1-1-0-4-0-1.emitter"
      }
    }
  ]
});
