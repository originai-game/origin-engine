import { file } from '@origingame/engine/author';
// Effekseer node 4 "Debris"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 11.4218,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 5,
      "interval": 0.0167
    }
  ],
  "duration": 0.0833,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    21.4243,
    33.2716
  ],
  "size": [
    0.5,
    1.5
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1719,
      0.8102
    ],
    [
      0.5312,
      0.3719
    ],
    [
      0.6719,
      0.2197
    ],
    [
      0.8438,
      0.0748
    ],
    [
      1.0,
      0.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
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
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "gravity": [
    0.0,
    -72.0,
    0.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-andrewfm01/textures/smoke_tex"
  },
  "blend": "alpha"
});
