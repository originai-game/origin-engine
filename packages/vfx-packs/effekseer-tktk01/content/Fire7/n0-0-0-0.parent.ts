import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "火メイン"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        3.0,
        0.0
      ],
      [
        -2.1213,
        2.1213,
        0.0
      ],
      [
        -3.0,
        0.0,
        0.0
      ],
      [
        -2.1213,
        -2.1213,
        0.0
      ],
      [
        0.0,
        -3.0,
        0.0
      ],
      [
        2.1213,
        -2.1213,
        0.0
      ],
      [
        3.0,
        0.0,
        0.0
      ],
      [
        2.1213,
        2.1213,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
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
        "$ogRef": "base/vfx/effekseer-tktk01/Fire7/n0-0-0-0.emitter"
      }
    }
  ]
});
