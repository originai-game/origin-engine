import { file } from '@origingame/engine/author';
// Effekseer node 0 "Node"
export default file.particles({
  "maxParticles": 25,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "spread": 0.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 1,
      "cycles": 20,
      "interval": 0.0167
    }
  ],
  "duration": 0.3333,
  "looping": false,
  "lifetime": [
    0.5,
    0.5667
  ],
  "speed": [
    0.0,
    0.0
  ],
  "size": [
    1.0,
    1.0
  ],
  "colorOverLife": [
    [
      0.0,
      [
        1.0,
        0.5356,
        0.7519,
        0.502
      ]
    ],
    [
      1.0,
      [
        1.0,
        0.5356,
        0.7519,
        0.502
      ]
    ]
  ],
  "texture": {
    "$ogRef": "base/vfx/effekseer-magicalxspiral/textures/smoke_01"
  },
  "sheet": {
    "columns": 4,
    "rows": 4,
    "frames": 16,
    "mode": {
      "kind": "life",
      "cycles": 1.0
    }
  },
  "blend": "additive"
});
