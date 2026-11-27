import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "interval"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "box",
    "halfExtents": [
      3.0,
      0.001,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 7,
      "interval": 0.0667
    }
  ],
  "duration": 0.4167,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Fire1/n0-0-0-0-0.emitter"
      }
    }
  ]
});
