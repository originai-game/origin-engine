import { file } from '@origingame/engine/author';
// Effekseer node 0.1 "光メイン左下"
export default file.particles({
  "maxParticles": 7,
  "shape": {
    "kind": "point"
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.5,
    0.5
  ],
  "speed": [
    2.8284,
    2.8284
  ],
  "size": [
    0.75,
    0.75
  ],
  "align": {
    "kind": "facing",
    "normal": [
      0.0,
      0.0,
      1.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.8994,
        0.8355,
        0.0738,
        0.7843
      ]
    ],
    [
      0.2656,
      [
        0.9255,
        0.8776,
        0.2084,
        0.6281
      ]
    ],
    [
      0.5156,
      [
        0.9505,
        0.9183,
        0.4059,
        0.481
      ]
    ],
    [
      0.7656,
      [
        0.9759,
        0.96,
        0.6766,
        0.3339
      ]
    ],
    [
      1.0,
      [
        1.0,
        1.0,
        1.0,
        0.1961
      ]
    ]
  ],
  "rotation": [
    -360.0,
    360.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Particle02"
  },
  "blend": "additive"
});
