import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2 "Wind_Sprite"
export default file.particles({
  "maxParticles": 19,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      2.0,
      2.0
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 15,
      "interval": 0.0167
    }
  ],
  "duration": 0.25,
  "looping": false,
  "lifetime": [
    1.1667,
    1.5
  ],
  "speed": [
    1.8,
    3.0
  ],
  "size": [
    26.0,
    26.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0769
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0625,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.125,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    0.0,
    -54.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Wind"
  },
  "blend": "alpha"
});
