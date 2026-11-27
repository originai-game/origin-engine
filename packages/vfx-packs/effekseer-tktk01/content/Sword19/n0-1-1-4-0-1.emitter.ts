import { file } from '@origingame/engine/author';
// Effekseer node 0.1.1.4.0.1 "glow大"
export default file.particles({
  "maxParticles": 313,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 250
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    2.5,
    2.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.5
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
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.1166,
        0.0,
        0.0104,
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
        0.1961
      ]
    ],
    [
      1.0,
      [
        0.3587,
        0.0,
        0.1166,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
