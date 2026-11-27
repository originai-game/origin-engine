import { file } from '@origingame/engine/author';
// Effekseer node 2.2 "グロー"
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
    0.5,
    0.5
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
      0.3333
    ],
    [
      0.0781,
      0.5063
    ],
    [
      0.1562,
      0.6401
    ],
    [
      0.2344,
      0.7398
    ],
    [
      0.3281,
      0.8219
    ],
    [
      0.4688,
      0.8868
    ],
    [
      0.8281,
      0.93
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
        0.1275,
        1.0,
        1.0,
        0.1569
      ]
    ],
    [
      1.0,
      [
        0.1275,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
