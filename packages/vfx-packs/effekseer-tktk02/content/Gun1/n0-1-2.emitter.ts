import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2 "Glow"
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
    0.8333,
    0.8333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    20.0,
    20.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.05
    ],
    [
      0.1094,
      0.2556
    ],
    [
      0.2188,
      0.429
    ],
    [
      0.3281,
      0.5731
    ],
    [
      0.4531,
      0.7059
    ],
    [
      0.5781,
      0.8092
    ],
    [
      0.7031,
      0.8874
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
        0.1166,
        0.0254,
        0.0
      ]
    ],
    [
      0.2,
      [
        1.0,
        0.1002,
        0.0218,
        1.0
      ]
    ],
    [
      0.4,
      [
        1.0,
        0.0851,
        0.0185,
        1.0
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
