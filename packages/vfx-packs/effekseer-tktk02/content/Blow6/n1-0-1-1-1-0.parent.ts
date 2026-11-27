import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "LocationNull"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0.0,
        2.0,
        0.0
      ],
      [
        -1.4142,
        1.4142,
        0.0
      ],
      [
        -2.0,
        0.0,
        0.0
      ],
      [
        -1.4142,
        -1.4142,
        0.0
      ],
      [
        0.0,
        -2.0,
        0.0
      ],
      [
        1.4142,
        -1.4142,
        0.0
      ],
      [
        2.0,
        0.0,
        0.0
      ],
      [
        1.4142,
        1.4142,
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
      "cycles": 20,
      "interval": 0.0333
    }
  ],
  "duration": 0.65,
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
        "$ogRef": "base/vfx/effekseer-tktk02/Blow6/n1-0-1-1-1-0.emitter"
      }
    }
  ]
});
