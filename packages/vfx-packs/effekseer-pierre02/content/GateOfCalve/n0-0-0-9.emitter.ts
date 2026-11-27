import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.9 "Shockwave"
export default file.particles({
  "maxParticles": 28,
  "shape": {
    "kind": "box",
    "halfExtents": [
      2.0,
      12.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 90,
      "interval": 0.0167
    }
  ],
  "duration": 1.5,
  "looping": false,
  "lifetime": [
    0.35,
    0.35
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.2,
    5.2
  ],
  "sizeOverLife": [
    [
      0.0,
      0.3846
    ],
    [
      0.6719,
      0.8648
    ],
    [
      0.8438,
      0.954
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
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.9075,
        0.7519,
        0.0
      ]
    ],
    [
      0.1429,
      [
        1.0,
        0.9075,
        0.7519,
        0.3765
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.9075,
        0.7519,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Shockwave"
  },
  "blend": "additive"
});
