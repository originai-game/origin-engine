import { file } from '@origingame/engine/author';
// Effekseer node 0.4 "Glow1"
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
    15.0,
    15.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0667
    ],
    [
      0.0781,
      0.3127
    ],
    [
      0.1719,
      0.5444
    ],
    [
      0.2656,
      0.7153
    ],
    [
      0.375,
      0.8502
    ],
    [
      0.4688,
      0.9213
    ],
    [
      0.5781,
      0.965
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
        0.0,
        0.0,
        0.0
      ]
    ],
    [
      0.0781,
      [
        1.0,
        0.0,
        0.0,
        0.0735
      ]
    ],
    [
      0.25,
      [
        1.0,
        0.0,
        0.0,
        0.5
      ]
    ],
    [
      0.3125,
      [
        1.0,
        0.0,
        0.0,
        0.508
      ]
    ],
    [
      0.375,
      [
        1.0,
        0.0,
        0.0,
        0.4691
      ]
    ],
    [
      0.5,
      [
        1.0,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      0.75,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk02/textures/Particle02"
  },
  "blend": "additive"
});
