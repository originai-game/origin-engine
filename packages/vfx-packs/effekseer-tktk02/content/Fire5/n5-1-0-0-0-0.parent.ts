import { file } from '@origingame/engine/author';
// Effekseer node 5 "ｉｎｔｅｒｖａｌ"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        2.0,
        0.0
      ],
      [
        -1.4142,
        1.4142,
        0.0
      ],
      [
        -2.0,
        0.0,
        0.0
      ],
      [
        -1.4142,
        -1.4142,
        0.0
      ],
      [
        0.0,
        -2.0,
        0.0
      ],
      [
        1.4142,
        -1.4142,
        0.0
      ],
      [
        2.0,
        0.0,
        0.0
      ],
      [
        1.4142,
        1.4142,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 3,
      "interval": 0.1
    }
  ],
  "duration": 0.2167,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Fire5/n5-1-0-0-0-0.emitter"
      }
    }
  ]
});
