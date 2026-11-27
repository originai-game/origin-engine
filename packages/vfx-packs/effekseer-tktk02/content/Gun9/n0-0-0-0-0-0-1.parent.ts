import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "AngleNull"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 7,
      "cycles": 3,
      "interval": 0.0167
    }
  ],
  "duration": 0.05,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-0-0-0-1.emitter"
      }
    }
  ]
});
