import { file } from '@origingame/engine/author';
// Effekseer node 1.0.0 "湯気発生位置調整"
export default file.particles({
  "maxParticles": 350,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.5,
        0.0
      ],
      [
        -0.3536,
        0.3536,
        0.0
      ],
      [
        -0.5,
        0.0,
        0.0
      ],
      [
        -0.3536,
        -0.3536,
        0.0
      ],
      [
        0.0,
        -0.5,
        0.0
      ],
      [
        0.3536,
        -0.3536,
        0.0
      ],
      [
        0.5,
        0.0,
        0.0
      ],
      [
        0.3536,
        0.3536,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 600.0,
  "duration": 0.4667,
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
        "$ogRef": "base/vfx/effekseer-tktk01/Fire2/n1-0-0-0.emitter"
      }
    }
  ]
});
