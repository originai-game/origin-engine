import { file } from '@origingame/engine/author';
// Effekseer node 0.7.1.1 "FireEmitterL"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.1049,
      0.4347,
      0.001
    ]
  },
  "spread": 25.8947,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 21,
      "interval": 0.0417
    }
  ],
  "duration": 0.85,
  "looping": false,
  "lifetime": [
    0.6667,
    1.1667
  ],
  "speed": [
    0.8017,
    2.1544
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Moonlight/n0-7-1-1-8.emitter"
      }
    }
  ]
});
