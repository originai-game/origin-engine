import { file } from '@origingame/engine/author';
// Effekseer node 0.7.0.4.1 "Wind_Sprite"
export default file.particles({
  "maxParticles": 750,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      3.0,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 25,
      "cycles": 36,
      "interval": 0.0167
    }
  ],
  "duration": 0.6,
  "looping": false,
  "lifetime": [
    0.2333,
    0.5333
  ],
  "speed": [
    11.25,
    18.75
  ],
  "size": [
    7.3958,
    10.3542
  ],
  "sizeOverLife": [
    [
      0.0,
      0.6761
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
      0.1304,
      [
        1.0,
        1.0,
        1.0,
        0.1506
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
  "rotation": [
    -360.0,
    360.0
  ],
  "angularVelocity": [
    -1800.0,
    1800.0
  ],
  "gravity": [
    -0.3961,
    -2.366,
    -1.239
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Wind"
  },
  "blend": "additive"
});
