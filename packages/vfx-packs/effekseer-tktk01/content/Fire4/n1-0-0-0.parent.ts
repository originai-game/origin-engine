import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "火柱制御"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        1.627,
        0.0
      ],
      [
        -1.1505,
        1.1505,
        0.0
      ],
      [
        -1.627,
        0.0,
        0.0
      ],
      [
        -1.1505,
        -1.1505,
        0.0
      ],
      [
        0.0,
        -1.627,
        0.0
      ],
      [
        1.1505,
        -1.1505,
        0.0
      ],
      [
        1.627,
        0.0,
        0.0
      ],
      [
        1.1505,
        1.1505,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 3
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
        "$ogRef": "base/vfx/effekseer-tktk01/Fire4/n1-0-0-0.emitter"
      }
    }
  ]
});
