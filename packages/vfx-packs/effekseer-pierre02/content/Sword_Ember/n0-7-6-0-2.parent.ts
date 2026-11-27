import { file } from '@origingame/engine/author';
// Effekseer node 0.7.6 "EmberGenerator"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "box",
    "halfExtents": [
      13.5,
      0.001,
      13.5
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.035,
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-7-6-0-2.emitter"
      }
    }
  ]
});
