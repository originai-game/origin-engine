import { file } from '@origingame/engine/author';
// Effekseer node 3 "星型"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "box",
    "halfExtents": [
      20.0,
      40.0,
      20.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0667
    }
  ],
  "duration": 0.2833,
  "looping": false,
  "lifetime": [
    0.1667,
    0.5
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
        "$ogRef": "base/vfx/effekseer-tktk01/Cure5/n3-0.emitter"
      }
    }
  ]
});
