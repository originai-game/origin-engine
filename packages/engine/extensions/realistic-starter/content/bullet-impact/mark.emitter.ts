import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "align": {
    "kind": "facing",
    "normal": [
      0,
      1,
      0
    ]
  },
  "blend": "alpha",
  "bursts": [
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.05000000074505806,
    0.04500000178813934,
    0.03999999910593033,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.8500000238418579
      ]
    ],
    [
      0.699999988079071,
      [
        1,
        1,
        1,
        0.8500000238418579
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
    0,
    0
  ],
  "lifetime": [
    1.7999999523162842,
    1.7999999523162842
  ],
  "looping": false,
  "maxParticles": 1,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    0.1599999964237213,
    0.1599999964237213
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/realistic-starter/textures/scorch")
});
