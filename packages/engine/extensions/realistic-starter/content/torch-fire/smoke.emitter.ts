import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -25,
    25
  ],
  "blend": "alpha",
  "color": [
    0.4000000059604645,
    0.3799999952316284,
    0.36000001430511475,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.15000000596046448,
      [
        1,
        1,
        1,
        0.3499999940395355
      ]
    ],
    [
      1,
      [
        1.600000023841858,
        1.600000023841858,
        1.600000023841858,
        0
      ]
    ]
  ],
  "drag": 0.699999988079071,
  "gravity": [
    0,
    0.3499999940395355,
    0
  ],
  "lifetime": [
    1.600000023841858,
    2.4000000953674316
  ],
  "lit": true,
  "maxParticles": 16,
  "rate": 5,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.05999999865889549,
    "surface": false
  },
  "sheet": {
    "blend": true,
    "columns": 2,
    "frames": 4,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 2
  },
  "size": [
    0.1599999964237213,
    0.23999999463558197
  ],
  "sizeOverLife": [
    [
      0,
      0.6000000238418579
    ],
    [
      1,
      3
    ]
  ],
  "softDistance": 0.20000000298023224,
  "speed": [
    0.4000000059604645,
    0.699999988079071
  ],
  "spread": 15,
  "texture": ref("base/vfx/realistic-starter/textures/smoke"),
  "wind": 1
});
