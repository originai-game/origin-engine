import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -1200,
    1200
  ],
  "blend": "additive",
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.03999999910593033,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.04170000180602074,
  "lifetime": [
    0.41670000553131104,
    0.41670000553131104
  ],
  "looping": false,
  "maxParticles": 375,
  "rate": 7200,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.75,
    0.75
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.28119999170303345,
      0.3346000015735626
    ],
    [
      0.5938000082969666,
      0.6601999998092651
    ],
    [
      0.6093999743461609,
      0.6510000228881836
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/star")
});
