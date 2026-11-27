import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -360,
    360
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 18,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "collision": {
    "against": "depth",
    "bounce": 0.20000000298023224,
    "die": false,
    "friction": 0.6000000238418579
  },
  "color": [
    0.41999998688697815,
    0.3799999952316284,
    0.3400000035762787,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      0.8500000238418579,
      [
        1,
        1,
        1,
        1
      ]
    ],
    [
      1,
      [
        1,
        1,
        1,
        0
      ]
    ]
  ],
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    -9.800000190734863,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    1.7999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.699999988079071,
      0.4000000059604645,
      0.699999988079071
    ]
  },
  "sheet": {
    "blend": false,
    "columns": 3,
    "frames": 3,
    "mode": {
      "kind": "random"
    },
    "rows": 1
  },
  "size": [
    0.05999999865889549,
    0.1599999964237213
  ],
  "speed": [
    1,
    3.5
  ],
  "spread": 40,
  "texture": ref("base/vfx/realistic-starter/textures/dirt")
});
