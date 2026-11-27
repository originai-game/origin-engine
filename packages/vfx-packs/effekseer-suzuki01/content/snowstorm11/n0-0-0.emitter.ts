import { file } from '@origingame/engine/author';
// Effekseer node 0.0.0 "random"
export default file.particles({
  "maxParticles": 4096,
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.275,
      0.275,
      1.1
    ]
  },
  "spread": 180.0,
  "rate": 0.0,
  "bursts": [
    {
      "time": 0.0,
      "count": 5,
      "cycles": 8,
      "interval": 0.0167
    }
  ],
  "duration": 0.1333,
  "looping": false,
  "lifetime": [
    2.0,
    2.0
  ],
  "speed": [
    0.2858,
    1.1432
  ],
  "size": [
    -0.165,
    0.275
  ],
  "colorOverLife": [
    [
      0.0,
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
    "$ogRef": "base/vfx/effekseer-suzuki01/textures/Particle01"
  },
  "blend": "additive"
});
