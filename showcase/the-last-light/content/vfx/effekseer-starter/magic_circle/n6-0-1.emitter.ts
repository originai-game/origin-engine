import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "colorOverLife": [
    [
      0,
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
        1
      ]
    ]
  ],
  "duration": 0.02669999934732914,
  "lifetime": [
    1.666700005531311,
    1.666700005531311
  ],
  "looping": false,
  "maxParticles": 20,
  "rate": 600,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0,
        0.949999988079071,
        0
      ],
      [
        -0.6718000173568726,
        0.6718000173568726,
        0
      ],
      [
        -0.949999988079071,
        0,
        0
      ],
      [
        -0.6718000173568726,
        -0.6718000173568726,
        0
      ],
      [
        0,
        -0.949999988079071,
        0
      ],
      [
        0.6718000173568726,
        -0.6718000173568726,
        0
      ],
      [
        0.949999988079071,
        0,
        0
      ],
      [
        0.6718000173568726,
        0.6718000173568726,
        0
      ]
    ]
  },
  "size": [
    0.17499999701976776,
    0.17499999701976776
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/starpart_")
});
