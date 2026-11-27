import { file } from '@origingame/engine/author';
// Effekseer node 0.0.3 "HeatHaze"
export default file.particles({
  "maxParticles": 4,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.25,
      0.25,
      0.25
    ]
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
    2.5,
    2.5
  ],
  "speed": [
    0.6,
    0.6
  ],
  "size": [
    8.0,
    8.0
  ],
  "sizeOverLife": [
    [
      0.0,
      0.375
    ],
    [
      0.1562,
      0.4356
    ],
    [
      0.3281,
      0.5307
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
        1.0,
        1.0,
        0.0
      ]
    ],
    [
      0.0667,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      0.7,
      [
        1.0,
        1.0,
        1.0,
        0.251
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.0
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-pierre01/textures/ExperimentalNoise"
  },
  "blend": "alpha",
  "distortion": 0.01
});
