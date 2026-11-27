import { file } from '@origingame/engine/author';
// Effekseer node 0.0.1.14 "Shockwave"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.2667,
    0.2667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    20.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0625
    ],
    [
      0.0781,
      0.207
    ],
    [
      0.1562,
      0.3283
    ],
    [
      0.2344,
      0.4296
    ],
    [
      0.3281,
      0.529
    ],
    [
      0.4688,
      0.6439
    ],
    [
      0.8281,
      0.8651
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
        0.1004
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
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/Shockwave"
  },
  "blend": "additive"
});
