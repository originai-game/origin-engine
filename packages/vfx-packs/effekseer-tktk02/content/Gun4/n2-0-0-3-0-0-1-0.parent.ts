import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "ｉｎｔｅｒｖａｌ1"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        1.5,
        0.0
      ],
      [
        -1.0607,
        1.0607,
        0.0
      ],
      [
        -1.5,
        0.0,
        0.0
      ],
      [
        -1.0607,
        -1.0607,
        0.0
      ],
      [
        0.0,
        -1.5,
        0.0
      ],
      [
        1.0607,
        -1.0607,
        0.0
      ],
      [
        1.5,
        0.0,
        0.0
      ],
      [
        1.0607,
        1.0607,
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
      "cycles": 5,
      "interval": 0.0667
    }
  ],
  "duration": 0.2833,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Gun4/n2-0-0-3-0-0-1-0.emitter"
      }
    }
  ]
});
