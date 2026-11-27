import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "ｉｎｔｅｒｖａｌ1"
export default file.particles({
  "maxParticles": 33,
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
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Gun4/n0-0-0-0-0-0.emitter"
      }
    }
  ]
});
