import { file } from '@origingame/engine/author';
// Effekseer node 0.1.6 "EmberGenerator"
export default file.particles({
  "maxParticles": 22,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5.0,
      0.001,
      5.0
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.0283,
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
        "$ogRef": "base/vfx/effekseer-pierre01/Flame/n0-1-6-0-2.emitter"
      }
    }
  ]
});
