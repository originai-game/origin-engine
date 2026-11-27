import { file } from '@origingame/engine/author';
// Effekseer node 0.4.1.0.0.1 "glow大"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      0.001,
      0.001
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 100
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    1.5,
    3.0
  ],
  "speedOverLife": [
    [
      0.0,
      2.1324
    ],
    [
      0.2656,
      1.4047
    ],
    [
      0.5,
      0.8969
    ],
    [
      0.75,
      0.5051
    ],
    [
      1.0,
      0.2705
    ]
  ],
  "size": [
    1.875,
    1.875
  ],
  "colorOverLife": [
    [
      0.0,
      [
        0.0,
        0.0,
        0.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.251
      ]
    ]
  ],
  "randomColorOverLife": [
    [
      0.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.251
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.251
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
