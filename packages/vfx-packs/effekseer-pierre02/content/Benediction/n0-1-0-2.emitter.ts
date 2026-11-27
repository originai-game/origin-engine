import { file } from '@origingame/engine/author';
// Effekseer node 0.1.0.2 "Blur"
export default file.particles({
  "maxParticles": 27,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.3375,
      0.3375,
      0.3375
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 7,
      "interval": 0.0167
    }
  ],
  "duration": 0.1167,
  "looping": false,
  "lifetime": [
    1.0,
    1.0
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    40.5,
    40.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1875
    ],
    [
      1.0,
      1.0
    ]
  ],
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
      0.0469,
      [
        1.0,
        1.0,
        1.0,
        0.2243
      ]
    ],
    [
      0.1,
      [
        1.0,
        1.0,
        1.0,
        0.4518
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
    -60.0,
    60.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalBlur"
  },
  "blend": "alpha",
  "distortion": 0.02
});
