import { file } from '@origingame/engine/author';
// Effekseer node 0.7.5.0.3 "FireVariation"
export default file.particles({
  "maxParticles": 170,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 64,
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
    6.075,
    6.075
  ],
  "size": [
    16.875,
    16.875
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.0625,
      [
        1.0,
        0.2195,
        0.0478,
        0.0739
      ]
    ],
    [
      0.0938,
      [
        1.0,
        0.2195,
        0.0478,
        0.1305
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.2195,
        0.0478,
        0.2123
      ]
    ],
    [
      0.1667,
      [
        1.0,
        0.2195,
        0.0478,
        0.3765
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
    39.0,
    51.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Fire_Blur"
  },
  "blend": "additive",
  "effect": {
    "$ogRef": "base/vfx/effekseer-pierre02/Sword_Ember/n0-7-5-0-3.effect"
  }
});
