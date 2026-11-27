import { file } from '@origingame/engine/author';
// Effekseer node 3.0.0 "Ine6"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        0.865,
        0.0
      ],
      [
        -0.6116,
        0.6116,
        0.0
      ],
      [
        -0.865,
        0.0,
        0.0
      ],
      [
        -0.6116,
        -0.6116,
        0.0
      ],
      [
        0.0,
        -0.865,
        0.0
      ],
      [
        0.6116,
        -0.6116,
        0.0
      ],
      [
        0.865,
        0.0,
        0.0
      ],
      [
        0.6116,
        0.6116,
        0.0
      ]
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 6
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.8333,
    0.8333
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
        "$ogRef": "base/vfx/effekseer-tktk02/Gun2/n3-0-0-1-0-0-0-0.emitter"
      }
    }
  ]
});
