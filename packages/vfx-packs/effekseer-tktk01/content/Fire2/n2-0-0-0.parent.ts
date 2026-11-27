import { file } from '@origingame/engine/author';
// Effekseer node 2.0.0 "湯気発生位置調整"
export default file.particles({
  "maxParticles": 225,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.75,
        0.0
      ],
      [
        -0.5303,
        0.5303,
        0.0
      ],
      [
        -0.75,
        0.0,
        0.0
      ],
      [
        -0.5303,
        -0.5303,
        0.0
      ],
      [
        0.0,
        -0.75,
        0.0
      ],
      [
        0.5303,
        -0.5303,
        0.0
      ],
      [
        0.75,
        0.0,
        0.0
      ],
      [
        0.5303,
        0.5303,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 300.0,
  "duration": 0.6,
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
        "$ogRef": "base/vfx/effekseer-tktk01/Fire2/n2-0-0-0.emitter"
      }
    }
  ]
});
