import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "duration": 0.16670000553131104,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 125,
  "rate": 600,
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
      "emitter": ref("base/vfx/effekseer-starter/Fire7/n1-0-2-0.emitter"),
      "inheritVelocity": 0,
      "trigger": "life"
    }
  ]
});
