import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "レンズフレア"
export default file.particles({
  "maxParticles": 57,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    15.0,
    15.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0625,
      0.789
    ],
    [
      0.1406,
      0.5829
    ],
    [
      0.2188,
      0.4325
    ],
    [
      0.3125,
      0.3127
    ],
    [
      0.4688,
      0.2198
    ],
    [
      0.8281,
      0.1555
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.1176
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
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/burst02_red"
  },
  "blend": "additive"
});
