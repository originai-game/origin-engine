import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0 "FireGenerator"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 15.0,
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Storm/n0-1-0-1-4.emitter"
      }
    }
  ]
});
