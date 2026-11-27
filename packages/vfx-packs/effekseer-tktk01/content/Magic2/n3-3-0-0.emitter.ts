import { file } from '@origingame/engine/author';
// Effekseer node 3.3.0.0 "gliow"
export default file.particles({
  "maxParticles": 313,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.001,
      0.375,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    1.6667,
    1.6667
  ],
  "speed": [
    0.3,
    3.15
  ],
  "speedOverLife": [
    [
      0.0,
      8.361
    ],
    [
      0.0625,
      5.5279
    ],
    [
      0.1094,
      3.8397
    ],
    [
      0.1406,
      3.0007
    ],
    [
      0.1719,
      2.3906
    ],
    [
      0.2344,
      1.8577
    ],
    [
      0.2656,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.5,
    0.5
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0023,
        0.0104,
        0.5882
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0023,
        0.0104,
        0.5882
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        1.0,
        0.0254,
        0.1166,
        0.5882
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0254,
        0.1166,
        0.5882
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
