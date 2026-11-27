import { file } from '@origingame/engine/author';
// Effekseer node 3.2.0.0 "gliow"
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
    1.6667,
    1.6667
  ],
  "speed": [
    0.1071,
    1.125
  ],
  "speedOverLife": [
    [
      0.0,
      4.0451
    ],
    [
      0.1719,
      2.7843
    ],
    [
      0.25,
      2.3705
    ],
    [
      0.3281,
      2.0663
    ],
    [
      0.3438,
      1.8188
    ],
    [
      0.3594,
      0.0
    ],
    [
      1.0,
      0.0
    ]
  ],
  "size": [
    0.25,
    0.25
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
