import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.4 "Wave_Light_TEMP"
export default file.particles({
  "maxParticles": 225,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 2,
      "interval": 0.5
    }
  ],
  "duration": 0.5167,
  "looping": false,
  "lifetime": [
    1.9,
    2.1
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    5.0,
    5.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.48
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
        0.8751,
        0.5605,
        0.0
      ]
    ],
    [
      0.1,
      [
        1.0,
        0.8751,
        0.5605,
        0.3765
      ]
    ],
    [
      0.8333,
      [
        1.0,
        0.8751,
        0.5605,
        0.3765
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.8751,
        0.5605,
        0.0
      ]
    ]
  ],
  "angularVelocity": [
    -6.0,
    6.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre02/textures/Wind"
  },
  "blend": "additive"
});
