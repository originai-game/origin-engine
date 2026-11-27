import { file } from '@origingame/engine/author';
// Effekseer node 0.0.4.0 "Wind_Sprite"
export default file.particles({
  "maxParticles": 8,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 11,
      "interval": 0.0417
    }
  ],
  "duration": 0.4333,
  "looping": false,
  "lifetime": [
    0.1167,
    0.2833
  ],
  "speed": [
    12.0,
    12.0
  ],
  "size": [
    2.2,
    2.2
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
      0.5,
      [
        1.0,
        1.0,
        1.0,
        0.0241
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
    -900.0,
    900.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Wind"
  },
  "blend": "alpha"
});
