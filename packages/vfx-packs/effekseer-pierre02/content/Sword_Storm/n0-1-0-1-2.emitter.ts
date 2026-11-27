import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.1.2 "Wind_Sprite"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.5,
      1.5,
      1.5
    ]
  },
  "spread": 0.0,
  "rate": 15.0,
  "duration": 1.0,
  "looping": true,
  "lifetime": [
    1.1667,
    1.5
  ],
  "speed": [
    1.8,
    3.0
  ],
  "size": [
    5.55,
    5.55
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1351
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
      0.125,
      [
        1.0,
        1.0,
        1.0,
        0.0837
      ]
    ],
    [
      0.25,
      [
        1.0,
        1.0,
        1.0,
        0.1434
      ]
    ],
    [
      0.375,
      [
        1.0,
        1.0,
        1.0,
        0.1793
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
    -1.5843,
    -9.4641,
    -4.9559
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Wind"
  },
  "blend": "alpha"
});
