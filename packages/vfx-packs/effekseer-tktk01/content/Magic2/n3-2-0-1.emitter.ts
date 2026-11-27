import { file } from '@origingame/engine/author';
// Effekseer node 3.2.0.1 "gliow大"
export default file.particles({
  "maxParticles": 625,
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
      "count": 500
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.75,
    0.75
  ],
  "speed": [
    0.1071,
    1.125
  ],
  "speedOverLife": [
    [
      0.0,
      1.829
    ],
    [
      0.3906,
      1.2424
    ],
    [
      0.7656,
      0.9073
    ],
    [
      0.7812,
      0.2504
    ],
    [
      0.7969,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    1.2305,
    1.2305
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.5469,
      1.0
    ],
    [
      0.5625,
      1.0
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
        0.0023,
        0.0104,
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0023,
        0.0104,
        0.1961
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
        0.1961
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0254,
        0.1166,
        0.1961
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
