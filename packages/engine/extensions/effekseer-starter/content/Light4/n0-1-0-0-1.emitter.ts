import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "upright",
    "axis": [
      0,
      1,
      0
    ]
  },
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 6,
      "interval": 0.11670000106096268,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.7764999866485596
      ]
    ],
    [
      0.6000000238418579,
      [
        1,
        1,
        1,
        0.7764999866485596
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
  "duration": 0.6000000238418579,
  "effect": ref("base/vfx/effekseer-starter/Light4/n0-1-0-0-1.effect"),
  "heightOverLife": [
    [
      0,
      2.6666998863220215
    ],
    [
      1,
      2.6666998863220215
    ]
  ],
  "lifetime": [
    0.41670000553131104,
    0.41670000553131104
  ],
  "looping": false,
  "maxParticles": 6,
  "rate": 0,
  "shape": {
    "kind": "box",
    "halfExtents": [
      5,
      1,
      0.0010000000474974513
    ]
  },
  "size": [
    0.375,
    0.375
  ],
  "speed": [
    22.941200256347656,
    30
  ],
  "speedOverLife": [
    [
      0,
      1.0206999778747559
    ],
    [
      0.17190000414848328,
      0.8629000186920166
    ],
    [
      0.34380000829696655,
      1.1720000505447388
    ],
    [
      0.5156000256538391,
      1.9572999477386475
    ],
    [
      0.6718999743461609,
      3.0845000743865967
    ],
    [
      0.6875,
      0.06310000270605087
    ],
    [
      0.7031000256538391,
      0
    ],
    [
      1,
      0
    ]
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/hisigata1")
});
