import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0.0.1.0.1 "glow"
export default file.particles({
  "maxParticles": 188,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.4,
      0.1333,
      0.1333
    ]
  },
  "spread": 0.0,
  "rate": 1200.0,
  "duration": 0.125,
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
    3.3333,
    3.3333
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0
    ],
    [
      0.0938,
      0.1861
    ],
    [
      0.2031,
      0.3683
    ],
    [
      0.3125,
      0.5174
    ],
    [
      0.4219,
      0.6382
    ],
    [
      0.5312,
      0.7355
    ],
    [
      0.6562,
      0.824
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
        0.0,
        0.0,
        0.0,
        0.502
      ]
    ],
    [
      1.0,
      [
        0.0,
        0.0,
        0.0,
        0.502
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
        0.502
      ]
    ],
    [
      1.0,
      [
        0.2158,
        0.2158,
        0.2158,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
