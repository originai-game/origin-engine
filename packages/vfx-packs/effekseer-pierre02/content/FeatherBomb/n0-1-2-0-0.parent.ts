import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2 "FireEmitter"
export default file.particles({
  "maxParticles": 65,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.1364,
      0.5457
    ]
  },
  "spread": 34.3302,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 26,
      "cycles": 2,
      "interval": 0.025
    }
  ],
  "duration": 0.0417,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    4.535,
    14.418
  ],
  "speedOverLife": [
    [
      0.0,
      2.1308
    ],
    [
      0.25,
      1.3814
    ],
    [
      0.5,
      0.8473
    ],
    [
      0.75,
      0.5423
    ],
    [
      1.0,
      0.4657
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
        "$ogRef": "base/vfx/effekseer-pierre02/FeatherBomb/n0-1-2-0-0.emitter"
      }
    }
  ]
});
