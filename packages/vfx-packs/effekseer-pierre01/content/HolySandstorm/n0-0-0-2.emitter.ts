import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.2 "Wind_Sprite"
export default file.particles({
  "maxParticles": 215,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.2,
      1.2,
      1.2
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2,
      "cycles": 240,
      "interval": 0.0167
    }
  ],
  "duration": 4.0,
  "looping": false,
  "lifetime": [
    1.3333,
    1.5
  ],
  "speed": [
    0.6,
    1.8
  ],
  "size": [
    27.5,
    27.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0727
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
        0.5356,
        0.4423,
        0.2195,
        0.0
      ]
    ],
    [
      0.0588,
      [
        0.5356,
        0.4423,
        0.2195,
        1.0
      ]
    ],
    [
      0.1765,
      [
        0.5356,
        0.4423,
        0.2195,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.5356,
        0.4423,
        0.2195,
        0.0
      ]
    ]
  ],
  "gravity": [
    13.9762,
    0.0,
    52.16
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Wind"
  },
  "blend": "alpha"
});
