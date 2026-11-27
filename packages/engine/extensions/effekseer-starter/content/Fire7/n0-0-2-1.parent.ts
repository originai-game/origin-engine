import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "bursts": [
    {
      "count": 1,
      "cycles": 10,
      "interval": 0.0333000011742115,
      "time": 0
    }
  ],
  "duration": 0.3167000114917755,
  "lifetime": [
    0.5,
    0.5
  ],
  "looping": false,
  "maxParticles": 13,
  "rate": 0,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ]
    ]
  },
  "size": [
    0,
    0
  ],
  "speed": [
    0,
    0
  ],
  "subEmitters": [
    {
      "emitter": ref("base/vfx/effekseer-starter/Fire7/n0-0-2-1.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
