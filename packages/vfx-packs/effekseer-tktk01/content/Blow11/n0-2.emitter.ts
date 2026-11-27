import { file } from '@origingame/engine/author';
// Effekseer node 0.2 "中心"
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
    3.0,
    3.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      0.0938,
      0.8139
    ],
    [
      0.2031,
      0.6317
    ],
    [
      0.3125,
      0.4826
    ],
    [
      0.4219,
      0.3618
    ],
    [
      0.5312,
      0.2645
    ],
    [
      0.6562,
      0.176
    ],
    [
      1.0,
      0.0
    ]
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0312,
      [
        1.0,
        1.0,
        1.0,
        0.053
      ]
    ],
    [
      0.0469,
      [
        1.0,
        1.0,
        1.0,
        0.2568
      ]
    ],
    [
      0.0625,
      [
        1.0,
        1.0,
        1.0,
        0.7847
      ]
    ],
    [
      0.0667,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        1.0
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle01"
  },
  "blend": "additive"
});
