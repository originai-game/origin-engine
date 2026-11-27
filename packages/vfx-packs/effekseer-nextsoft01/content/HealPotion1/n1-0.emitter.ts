import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "キラキラ"
export default file.particles({
  "maxParticles": 38,
  "shape": {
    "kind": "point"
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 30,
      "interval": 0.0167
    }
  ],
  "duration": 0.5,
  "looping": false,
  "lifetime": [
    0.5833,
    0.5833
  ],
  "speed": [
    0.6364,
    2.5456
  ],
  "size": [
    0.3,
    0.3
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.1734,
        0.9858,
        0.3345,
        1.0
      ]
    ],
    [
      0.7143,
      [
        0.1734,
        0.9858,
        0.3345,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.1734,
        0.9858,
        0.3345,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.9899,
        1.0,
        1.0
      ]
    ],
    [
      0.7143,
      [
        1.0,
        0.9899,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.9899,
        1.0,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    -120.0,
    120.0
  ],
  "gravity": [
    0.0,
    -18.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-nextsoft01/textures/kirakira"
  },
  "sheet": {
    "columns": 2,
    "rows": 2,
    "frames": 2,
    "mode": {
      "kind": "fps",
      "fps": 20.0
    }
  },
  "blend": "additive"
});
