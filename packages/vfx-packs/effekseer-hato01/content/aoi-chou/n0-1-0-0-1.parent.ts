import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        1.0,
        0.0,
        0.0
      ],
      [
        0.7071,
        0.0,
        -0.7071
      ],
      [
        0.0,
        0.0,
        -1.0
      ],
      [
        -0.7071,
        0.0,
        -0.7071
      ],
      [
        -1.0,
        0.0,
        0.0
      ],
      [
        -0.7071,
        0.0,
        0.7071
      ],
      [
        0.0,
        0.0,
        1.0
      ],
      [
        0.7071,
        0.0,
        0.7071
      ]
    ]
  },
  "spread": 92.0249,
  "rate": 0.24,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    13.3333,
    26.6667
  ],
  "speed": [
    1.4151,
    1.4221
  ],
  "size": [
    0.0,
    0.0
  ],
  "subEmitters": [
    {
      "trigger": "life",
      "emitter": {
        "$ogRef": "base/vfx/effekseer-hato01/aoi-chou/n0-1-0-0-1.emitter"
      }
    }
  ]
});
