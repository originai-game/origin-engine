import { file } from '@origingame/engine/author';
// Effekseer node 0.2.4 "HeatHaze"
export default file.particles({
  "maxParticles": 12,
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
      "count": 9
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    1.4667,
    2.2667
  ],
  "speed": [
    1.8,
    1.8
  ],
  "size": [
    27.0,
    27.0
  ],
  "sizeOverLife": [
    [
      0.0,
      1.0
    ],
    [
      1.0,
      0.6667
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
      0.2679,
      [
        1.0,
        1.0,
        1.0,
        0.3765
      ]
    ],
    [
      0.4643,
      [
        1.0,
        1.0,
        1.0,
        0.3765
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
    "$ogRef": "base/vfx/effekseer-pierre02/textures/ExperimentalNoise"
  },
  "blend": "alpha",
  "distortion": 0.02
});
