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
    0.03999999910593033,
    0.03500000014901161,
    0.029999999329447746,
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
      0.029999999329447746,
      [
        1,
        1,
        1,
        0.800000011920929
      ]
    ],
    [
      0.75,
      [
        1,
        1,
        1,
        0.800000011920929
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
    3.799999952316284,
    3.799999952316284
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
    3.200000047683716,
    3.200000047683716
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/realistic-starter/textures/scorch")
});
