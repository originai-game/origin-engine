import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.1.0.0 "String"
export default file.particles({
  "maxParticles": 14,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 11,
      "interval": 0.2
    }
  ],
  "duration": 2.0167,
  "looping": false,
  "lifetime": [
    3.0,
    3.0
  ],
  "speed": [
    20.0,
    20.0
  ],
  "speedOverLife": [
    [
      0.0,
      0.6837
    ],
    [
      0.125,
      0.2813
    ],
    [
      0.25,
      0.0902
    ],
    [
      0.375,
      0.1271
    ],
    [
      0.5,
      0.3921
    ],
    [
      0.625,
      0.8851
    ],
    [
      0.75,
      1.6061
    ],
    [
      1.0,
      3.6918
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
        "$ogRef": "base/vfx/effekseer-pierre02/Sword_Lightning/n0-7-0-1-0-0-1-15.emitter"
      }
    }
  ]
});
