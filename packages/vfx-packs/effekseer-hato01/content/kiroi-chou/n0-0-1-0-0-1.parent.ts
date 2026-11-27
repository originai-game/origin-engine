import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        -0.3,
        0.0,
        0.0
      ],
      [
        -0.2121,
        0.0,
        -0.2121
      ],
      [
        0.0,
        0.0,
        -0.3
      ],
      [
        0.2121,
        0.0,
        -0.2121
      ],
      [
        0.3,
        0.0,
        0.0
      ],
      [
        0.2121,
        0.0,
        0.2121
      ],
      [
        0.0,
        0.0,
        0.3
      ],
      [
        -0.2121,
        0.0,
        0.2121
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 6,
      "interval": 1.675
    }
  ],
  "duration": 8.3917,
  "looping": false,
  "lifetime": [
    20.0,
    20.0
  ],
  "speed": [
    0.155,
    0.265
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-hato01/kiroi-chou/n0-0-1-0-0-1.emitter"
      }
    }
  ]
});
