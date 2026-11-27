import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "MainFireGenerator"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 26
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.9833,
    1.0167
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
        "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-3-0-3.emitter"
      }
    }
  ]
});
