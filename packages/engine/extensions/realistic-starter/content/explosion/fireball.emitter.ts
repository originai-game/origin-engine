import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -60,
    60
  ],
  "blend": "additive",
  "bursts": [
    {
      "count": 16,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        9,
        6,
        3.200000047683716,
        1
      ]
    ],
    [
      0.25,
      [
        6,
        2.5999999046325684,
        0.800000011920929,
        1
      ]
    ],
    [
      0.6000000238418579,
      [
        2.200000047683716,
        0.6000000238418579,
        0.11999999731779099,
        0.800000011920929
      ]
    ],
    [
      1,
      [
        0.5,
        0.10000000149011612,
        0.019999999552965164,
        0
      ]
    ]
  ],
  "drag": 4,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    2,
    0
  ],
  "lifetime": [
    0.44999998807907104,
    0.8500000238418579
  ],
  "looping": false,
  "maxParticles": 24,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "sphere",
    "radius": 0.3499999940395355,
    "surface": false
  },
  "sheet": {
    "blend": false,
    "columns": 2,
    "frames": 2,
    "mode": {
      "kind": "random"
    },
    "rows": 1
  },
  "size": [
    0.8999999761581421,
    1.399999976158142
  ],
  "sizeOverLife": [
    [
      0,
      0.44999998807907104
    ],
    [
      0.25,
      1
    ],
    [
      1,
      1.350000023841858
    ]
  ],
  "softDistance": 0.4000000059604645,
  "speed": [
    1.5,
    3.5
  ],
  "spread": 180,
  "texture": ref("base/vfx/realistic-starter/textures/fire")
});
