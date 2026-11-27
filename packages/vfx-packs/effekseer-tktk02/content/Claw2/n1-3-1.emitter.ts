import { file } from '@origingame/engine/author';
// Effekseer node 1.3.1 "Glow2"
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
    0.4167,
    0.4167
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    12.5,
    12.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.08
    ],
    [
      0.1094,
      0.2792
    ],
    [
      0.2188,
      0.447
    ],
    [
      0.3281,
      0.5866
    ],
    [
      0.4531,
      0.7152
    ],
    [
      0.5781,
      0.8152
    ],
    [
      0.7031,
      0.891
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
        0.2195,
        0.0
      ]
    ],
    [
      0.0938,
      [
        0.9498,
        1.0,
        0.1435,
        0.3529
      ]
    ],
    [
      0.2031,
      [
        0.8929,
        1.0,
        0.0765,
        0.7568
      ]
    ],
    [
      0.4062,
      [
        0.7924,
        1.0,
        0.0102,
        1.0
      ]
    ],
    [
      0.5938,
      [
        0.7054,
        1.0,
        0.0044,
        1.0
      ]
    ],
    [
      0.6094,
      [
        0.6984,
        1.0,
        0.004,
        0.9649
      ]
    ],
    [
      0.7969,
      [
        0.6173,
        1.0,
        0.001,
        0.2627
      ]
    ],
    [
      1.0,
      [
        0.5356,
        1.0,
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
