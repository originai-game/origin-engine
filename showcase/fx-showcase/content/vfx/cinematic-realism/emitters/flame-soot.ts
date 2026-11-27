import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.14000000059604645,
    0.12999999523162842,
    0.12999999523162842,
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
      0.3499999940395355,
      [
        1,
        1,
        1,
        0
      ]
    ],
    [
      0.550000011920929,
      [
        1,
        1,
        1,
        0.5
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
  "drag": 2.200000047683716,
  "duration": 2,
  "gravity": [
    0,
    1.399999976158142,
    0
  ],
  "lifetime": [
    1.399999976158142,
    2
  ],
  "lit": true,
  "maxParticles": 64,
  "rate": 30,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 6,
    "radius": 0.03999999910593033
  },
  "sheet": {
    "blend": true,
    "columns": 16,
    "frames": 64,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 4
  },
  "size": [
    0.30000001192092896,
    0.44999998807907104
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      0.4000000059604645,
      1.600000023841858
    ],
    [
      1,
      3.4000000953674316
    ]
  ],
  "softDistance": 0.30000001192092896,
  "speed": [
    8.5,
    11
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
