import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3 "衝撃グロー"
export default file.particles({
  "maxParticles": 10,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 8,
      "interval": 0.0667
    }
  ],
  "duration": 0.4833,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    30.0,
    30.0
  ],
  "speedOverLife": [
    [
      0.0,
      5.5026
    ],
    [
      0.1719,
      2.1897
    ],
    [
      0.3438,
      0.415
    ],
    [
      0.5,
      0.2199
    ],
    [
      0.6562,
      1.3755
    ],
    [
      0.6719,
      0.2473
    ],
    [
      0.6875,
      0.0
    ],
    [
      1.0,
      0.0
    ]
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
        0.3922
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
