import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0 "Fire"
export default file.particles({
  "maxParticles": 63,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 10,
      "cycles": 6,
      "interval": 0.0833
    }
  ],
  "duration": 0.4333,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
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
        "$ogRef": "base/vfx/effekseer-tktk02/Fire7/n0-0-0-0-0.emitter"
      }
    }
  ]
});
