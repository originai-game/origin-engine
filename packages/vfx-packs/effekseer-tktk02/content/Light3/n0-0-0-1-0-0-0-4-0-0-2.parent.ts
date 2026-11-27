import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.1.0.0 "Angle"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "sphere",
    "radius": 5.0
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 7
    }
  ],
  "duration": 0.0167,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Light3/n0-0-0-1-0-0-0-4-0-0-2.emitter"
      }
    }
  ]
});
