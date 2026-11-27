import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.0.1 "FireVariation"
export default file.particles({
  "maxParticles": 60,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 24,
      "cycles": 2,
      "interval": 0.4
    }
  ],
  "duration": 0.4167,
  "looping": false,
  "lifetime": [
    0.4167,
    0.5833
  ],
  "speed": [
    3.6,
    3.6
  ],
  "size": [
    4.0,
    4.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.8
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.25
    ],
    [
      1.0,
      1.25
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.1875,
      [
        1.0,
        0.2195,
        0.0478,
        0.1414
      ]
    ],
    [
      0.2656,
      [
        1.0,
        0.2195,
        0.0478,
        0.2329
      ]
    ],
    [
      0.3333,
      [
        1.0,
        0.2195,
        0.0478,
        0.3346
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.2195,
        0.0478,
        0.0
      ]
    ]
  ],
  "rotation": [
    -6.0,
    6.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/FireBall/n0-1-0-0-1.effect"
  }
});
