import { file } from '@origingame/engine/author';
// Effekseer node 0.0 "風大"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.5,
      1.5,
      1.5
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 20
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
    45.75,
    45.75
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0164
    ],
    [
      1.0,
      1.0
    ]
  ],
  "align": {
    "kind": "upright",
    "axis": [
      0.0,
      1.0,
      0.0
    ]
  },
  "colorOverLife": [
    [
      0.0,
      [
        0.4995,
        1.0,
        0.4704,
        0.6863
      ]
    ],
    [
      1.0,
      [
        0.4704,
        0.9828,
        0.68,
        0.0
      ]
    ]
  ],
  "rotation": [
    -180.0,
    180.0
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave004"
  },
  "blend": "additive"
});
