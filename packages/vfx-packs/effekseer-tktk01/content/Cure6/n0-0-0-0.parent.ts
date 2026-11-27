import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "開幕粒"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        4.0,
        0.0
      ],
      [
        -2.8284,
        2.8284,
        0.0
      ],
      [
        -4.0,
        0.0,
        0.0
      ],
      [
        -2.8284,
        -2.8284,
        0.0
      ],
      [
        0.0,
        -4.0,
        0.0
      ],
      [
        2.8284,
        -2.8284,
        0.0
      ],
      [
        4.0,
        0.0,
        0.0
      ],
      [
        2.8284,
        2.8284,
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
    0.6667,
    0.6667
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
        "$ogRef": "base/vfx/effekseer-tktk01/Cure6/n0-0-0-0.emitter"
      }
    }
  ]
});
