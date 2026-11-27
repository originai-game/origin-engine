import { file } from '@origingame/engine/author';
// Effekseer node 0.2.0 "FireEmitter"
export default file.particles({
  "maxParticles": 33,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.2744,
      3.5672,
      0.001
    ]
  },
  "spread": 98.0495,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 13,
      "cycles": 2,
      "interval": 0.0333
    }
  ],
  "duration": 0.05,
  "looping": false,
  "lifetime": [
    0.6667,
    1.1667
  ],
  "speed": [
    2.1377,
    10.6887
  ],
  "speedOverLife": [
    [
      0.0,
      0.4746
    ],
    [
      0.25,
      0.8999
    ],
    [
      0.5,
      1.1334
    ],
    [
      0.75,
      1.1667
    ],
    [
      1.0,
      1.0041
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-2-0-5-0-1.emitter"
      }
    }
  ]
});
