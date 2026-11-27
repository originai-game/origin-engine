import { file } from '@origingame/engine/author';
// Effekseer node 3 "衝撃グロー"
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
      0.0667
    ],
    [
      0.0938,
      0.2404
    ],
    [
      0.2031,
      0.4104
    ],
    [
      0.3125,
      0.5496
    ],
    [
      0.4219,
      0.6623
    ],
    [
      0.6562,
      0.8357
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
        0.586,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        0.586,
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
