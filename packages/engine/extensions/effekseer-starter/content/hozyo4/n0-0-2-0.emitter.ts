import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        0.04780000075697899,
        0.04780000075697899,
        0.3921999931335449
      ]
    ],
    [
      0.4844000041484833,
      [
        1,
        0.011099999770522118,
        0.011099999770522118,
        0.2021999955177307
      ]
    ],
    [
      1,
      [
        1,
        0,
        0,
        0
      ]
    ]
  ],
  "lifetime": [
    0.16670000553131104,
    0.33329999446868896
  ],
  "maxParticles": 3813,
  "rate": 12000,
  "shape": {
    "kind": "point"
  },
  "size": [
    1,
    1
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
