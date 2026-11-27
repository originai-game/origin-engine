import { file } from '@origingame/engine/author';
// Effekseer node 2.0 "Node"
export default file.particles({
  "maxParticles": 13,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 10,
      "interval": 0.0167
    }
  ],
  "duration": 0.1667,
  "looping": false,
  "lifetime": [
    16.6667,
    16.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    6.0,
    6.0
  ],
  "align": {
    "kind": "facing",
    "normal": [
      -0.342,
      0.0,
      0.9397
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.2195,
        0.5356,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.2195,
        0.5356,
        0.502
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2195,
        0.2195,
        0.7445,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.2195,
        0.7445,
        0.502
      ]
    ]
  ],
  "angularVelocity": [
    900.0,
    1800.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/Wind01"
  },
  "blend": "additive"
});
