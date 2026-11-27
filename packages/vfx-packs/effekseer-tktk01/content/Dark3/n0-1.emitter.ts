import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "glow"
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
      0.1429
    ],
    [
      0.0781,
      0.3688
    ],
    [
      0.1719,
      0.5816
    ],
    [
      0.2656,
      0.7385
    ],
    [
      0.375,
      0.8625
    ],
    [
      0.4688,
      0.9277
    ],
    [
      0.5781,
      0.9678
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
        0.1166,
        0.0,
        0.0104,
        1.0
      ]
    ],
    [
      0.5156,
      [
        0.1166,
        0.0,
        0.0494,
        0.4844
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.0,
        0.1166,
        0.0
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.3587,
        0.0,
        0.1166,
        1.0
      ]
    ],
    [
      0.5,
      [
        0.3587,
        0.0,
        0.2195,
        0.5
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.0,
        0.3587,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
