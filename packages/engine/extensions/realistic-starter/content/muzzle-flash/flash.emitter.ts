import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 2,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    4,
    2.4000000953674316,
    1.100000023841858,
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
      1,
      [
        1,
        0.6000000238418579,
        0.3499999940395355,
        0
      ]
    ]
  ],
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.03999999910593033,
    0.05999999865889549
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "point"
  },
  "sheet": {
    "blend": false,
    "columns": 2,
    "frames": 4,
    "mode": {
      "kind": "random"
    },
    "rows": 2
  },
  "size": [
    0.3199999928474426,
    0.46000000834465027
  ],
  "sizeOverLife": [
    [
      0,
      0.699999988079071
    ],
    [
      1,
      1.149999976158142
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/realistic-starter/textures/muzzle")
});
