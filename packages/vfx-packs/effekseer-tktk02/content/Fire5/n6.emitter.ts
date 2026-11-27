import { file } from '@origingame/engine/author';
// Effekseer node 6 "Glow"
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
    0.6667,
    0.6667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    10.0,
    10.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1
    ],
    [
      0.1094,
      0.2948
    ],
    [
      0.2188,
      0.459
    ],
    [
      0.3281,
      0.5955
    ],
    [
      0.4531,
      0.7214
    ],
    [
      0.5781,
      0.8192
    ],
    [
      0.7031,
      0.8933
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
        0.2195,
        0.0478,
        0.0
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.1636,
        0.0356,
        0.2196
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.1166,
        0.0254,
        0.1882
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.0781,
        0.017,
        0.0784
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0104,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
