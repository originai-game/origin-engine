import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        0,
        0,
        0.3921999931335449
      ]
    ],
    [
      1,
      [
        1,
        0,
        0,
        0.3921999931335449
      ]
    ]
  ],
  "duration": 0.41670000553131104,
  "lifetime": [
    0.33329999446868896,
    0.33329999446868896
  ],
  "looping": false,
  "maxParticles": 52,
  "rate": 120,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.0010000000474974513,
      1.2990000247955322,
      0.0010000000474974513
    ]
  },
  "size": [
    1,
    1
  ],
  "speed": [
    1.9486000537872314,
    5.845699787139893
  ],
  "speedOverLife": [
    [
      0,
      0.4657000005245209
    ],
    [
      0.25,
      0.5422999858856201
    ],
    [
      0.5,
      0.8472999930381775
    ],
    [
      0.75,
      1.3813999891281128
    ],
    [
      1,
      2.1308000087738037
    ]
  ],
  "spread": 29.496200561523438,
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
