import { file } from '@origingame/engine/author';
// Effekseer node 2 "glow"
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
    1.0,
    1.0
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
      0.0938,
      0.2675
    ],
    [
      0.2031,
      0.4315
    ],
    [
      0.3125,
      0.5656
    ],
    [
      0.4219,
      0.6744
    ],
    [
      0.6562,
      0.8416
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
        0.0478,
        0.0478,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.0478,
        0.0478,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
