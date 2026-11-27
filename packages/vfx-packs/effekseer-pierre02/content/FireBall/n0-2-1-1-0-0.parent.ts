import { file } from '@origingame/engine/author';
// Effekseer node 0.2.1 "MainFireGenerator"
export default file.particles({
  "maxParticles": 34,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 27
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
        "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-2-1-1-0-0.emitter"
      }
    }
  ]
});
