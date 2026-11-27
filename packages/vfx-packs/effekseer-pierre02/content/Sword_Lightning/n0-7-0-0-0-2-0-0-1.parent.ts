import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.0.0.2.0 "DistanceAdjust"
export default file.particles({
  "maxParticles": 15,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 12,
      "interval": 0.0167
    }
  ],
  "duration": 0.2,
  "looping": false,
  "lifetime": [
    7.6667,
    7.6667
  ],
  "speed": [
    8.3478,
    12.5217
  ],
  "speedOverLife": [
    [
      0.0,
      1.4256
    ],
    [
      0.4844,
      1.0413
    ],
    [
      1.0,
      0.4713
    ]
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-0-0-2-0-0-1.emitter"
      }
    }
  ]
});
