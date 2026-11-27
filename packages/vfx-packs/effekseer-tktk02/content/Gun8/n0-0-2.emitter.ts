import { file } from '@origingame/engine/author';
// Effekseer node 0.0.2 "Glow"
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
    40.0,
    40.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.125
    ],
    [
      0.0781,
      0.3557
    ],
    [
      0.1719,
      0.5728
    ],
    [
      0.2656,
      0.7331
    ],
    [
      0.375,
      0.8596
    ],
    [
      0.4688,
      0.9262
    ],
    [
      0.5781,
      0.9672
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
        0.5356,
        0.2195,
        0.0
      ]
    ],
    [
      0.125,
      [
        1.0,
        0.4423,
        0.1768,
        0.2353
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.3587,
        0.139,
        0.4392
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.2195,
        0.0781,
        0.3765
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.1166,
        0.0356,
        0.1569
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
