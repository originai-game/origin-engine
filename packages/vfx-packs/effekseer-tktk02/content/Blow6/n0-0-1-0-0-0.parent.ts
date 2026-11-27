import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "LocationNull"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        3.0,
        0.0
      ],
      [
        -2.1213,
        2.1213,
        0.0
      ],
      [
        -3.0,
        0.0,
        0.0
      ],
      [
        -2.1213,
        -2.1213,
        0.0
      ],
      [
        0.0,
        -3.0,
        0.0
      ],
      [
        2.1213,
        -2.1213,
        0.0
      ],
      [
        3.0,
        0.0,
        0.0
      ],
      [
        2.1213,
        2.1213,
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
      "cycles": 10,
      "interval": 0.0667
    }
  ],
  "duration": 0.6167,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n0-0-1-0-0-0.emitter"
      }
    }
  ]
});
