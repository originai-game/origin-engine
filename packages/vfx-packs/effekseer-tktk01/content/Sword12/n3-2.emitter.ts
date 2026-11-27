import { file } from '@origingame/engine/author';
// Effekseer node 3.2 "グロー"
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
    25.0,
    25.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.2
    ],
    [
      0.0781,
      0.4076
    ],
    [
      0.1562,
      0.5681
    ],
    [
      0.2344,
      0.6878
    ],
    [
      0.3281,
      0.7862
    ],
    [
      0.4688,
      0.8641
    ],
    [
      0.8281,
      0.916
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
        0.2353
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
