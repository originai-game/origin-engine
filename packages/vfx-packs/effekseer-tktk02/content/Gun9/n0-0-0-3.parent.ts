import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "ｉｎｔｅｒｖａｌ"
export default file.particles({
  "maxParticles": 9,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        1.0,
        0.0
      ],
      [
        -0.7071,
        0.7071,
        0.0
      ],
      [
        -1.0,
        0.0,
        0.0
      ],
      [
        -0.7071,
        -0.7071,
        0.0
      ],
      [
        0.0,
        -1.0,
        0.0
      ],
      [
        0.7071,
        -0.7071,
        0.0
      ],
      [
        1.0,
        0.0,
        0.0
      ],
      [
        0.7071,
        0.7071,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Gun9/n0-0-0-3.emitter"
      }
    }
  ]
});
