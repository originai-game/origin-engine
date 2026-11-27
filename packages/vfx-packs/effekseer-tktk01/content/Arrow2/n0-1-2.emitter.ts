import { file } from '@origingame/engine/author';
// Effekseer node 0.1.2 "glow"
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
    0.3333,
    0.3333
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    7.5,
    7.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.1333
    ],
    [
      0.0938,
      0.2946
    ],
    [
      0.2031,
      0.4525
    ],
    [
      0.3125,
      0.5817
    ],
    [
      0.4219,
      0.6864
    ],
    [
      0.6562,
      0.8475
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
        1.0,
        0.0,
        0.3922
      ]
    ],
    [
      1.0,
      [
        0.0,
        1.0,
        0.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
