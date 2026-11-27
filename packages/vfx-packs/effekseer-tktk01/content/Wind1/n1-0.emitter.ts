import { file } from '@origingame/engine/author';
// Effekseer node 1.0 "風小"
export default file.particles({
  "maxParticles": 125,
  "shape": {
    "kind": "box",
    "halfExtents": [
      1.0,
      1.0,
      1.0
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 2
    }
  ],
  "duration": 0.0167,
  "looping": false,
  "lifetime": [
    0.3333,
    0.3333
  ],
  "speed": [
    2.5981,
    10.3923
  ],
  "size": [
    15.5,
    15.5
  ],
  "sizeOverLife": [
    [
      0.0,
      0.0323
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
        0.1961
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
    "$ogRef": "base/vfx/effekseer-tktk01/textures/Shock_wave001"
  },
  "blend": "additive"
});
