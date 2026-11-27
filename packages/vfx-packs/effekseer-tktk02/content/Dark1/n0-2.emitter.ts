import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "Glow"
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
    7.0,
    7.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.1094,
      0.8454
    ],
    [
      0.2188,
      0.7151
    ],
    [
      0.3281,
      0.6067
    ],
    [
      0.4531,
      0.5068
    ],
    [
      0.7031,
      0.3704
    ],
    [
      1.0,
      0.2857
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.5356,
        0.0478,
        1.0,
        0.0
      ]
    ],
    [
      0.25,
      [
        0.4423,
        0.0356,
        1.0,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.3587,
        0.0254,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.2195,
        0.0104,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
