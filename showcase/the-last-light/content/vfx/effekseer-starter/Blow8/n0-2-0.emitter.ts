import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 30,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        0,
        0,
        0,
        0.800000011920929
      ]
    ],
    [
      1,
      [
        0,
        0,
        0,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.25,
    0.41670000553131104
  ],
  "looping": false,
  "maxParticles": 338,
  "randomColorOverLife": [
    [
      0,
      [
        0.21580000221729279,
        0.21580000221729279,
        0.21580000221729279,
        0.800000011920929
      ]
    ],
    [
      1,
      [
        0.21580000221729279,
        0.21580000221729279,
        0.21580000221729279,
        0
      ]
    ]
  ],
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.0010000000474974513,
      0.5,
      0.0010000000474974513
    ]
  },
  "size": [
    1,
    1
  ],
  "speed": [
    4.5,
    19.5
  ],
  "speedOverLife": [
    [
      0,
      3.695199966430664
    ],
    [
      0.25,
      1.7410999536514282
    ],
    [
      0.375,
      1.026900053024292
    ],
    [
      0.5,
      0.5001000165939331
    ],
    [
      0.625,
      0.1607999950647354
    ],
    [
      0.75,
      0.009100000374019146
    ],
    [
      0.875,
      0.04479999840259552
    ],
    [
      1,
      0.2581999897956848
    ]
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle01-b96f716d")
});
