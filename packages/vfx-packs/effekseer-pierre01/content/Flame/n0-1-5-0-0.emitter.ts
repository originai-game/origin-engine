import { file } from '@origingame/engine/author';
// Effekseer node 0.1.5.0.0 "FireVariation"
export default file.particles({
  "maxParticles": 128,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 48,
      "cycles": 3,
      "interval": 0.2667
    }
  ],
  "duration": 0.55,
  "looping": false,
  "lifetime": [
    0.25,
    0.35
  ],
  "speed": [
    2.7,
    2.7
  ],
  "size": [
    7.5,
    7.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.4
    ],
    [
      0.1562,
      0.4737
    ],
    [
      0.3281,
      0.5702
    ],
    [
      1.0,
      1.0
    ]
  ],
  "heightOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.1562,
      0.3229
    ],
    [
      0.3281,
      0.4837
    ],
    [
      1.0,
      1.2
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
        0.3587,
        0.1166,
        0.0
      ]
    ],
    [
      0.0625,
      [
        1.0,
        0.3587,
        0.1166,
        0.0985
      ]
    ],
    [
      0.0938,
      [
        1.0,
        0.3587,
        0.1166,
        0.174
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.3587,
        0.1166,
        0.2831
      ]
    ],
    [
      0.1406,
      [
        1.0,
        0.3587,
        0.1166,
        0.3541
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.3587,
        0.1166,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.3587,
        0.1166,
        0.0
      ]
    ]
  ],
  "rotation": [
    39.0,
    51.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre01/Flame/n0-1-5-0-0.effect"
  }
});
