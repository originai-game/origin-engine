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
    },
    {
      "count": 1,
      "cycles": 1,
      "interval": 0,
      "time": 0.2199999988079071
    }
  ],
  "color": [
    0.8999999761581421,
    0.949999988079071,
    1,
    1
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        1,
        0.550000011920929
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
  "duration": 0.30000001192092896,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    1.100000023841858,
    1.2999999523162842
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    0.3499999940395355,
    0.3499999940395355
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      1,
      3.4000000953674316
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/realistic-starter/textures/ring")
});
