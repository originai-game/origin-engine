import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "color": [
    0.949999988079071,
    1.0800000429153442,
    1.2000000476837158,
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
        0.25
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
  "drag": 2,
  "duration": 2,
  "gravity": [
    0,
    0.05000000074505806,
    0
  ],
  "lifetime": [
    1,
    1.7999999523162842
  ],
  "maxParticles": 24,
  "rate": 8,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 60,
    "radius": 0.4000000059604645
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
    0.5,
    0.800000011920929
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2
    ]
  ],
  "softDistance": 0.30000001192092896,
  "speed": [
    0.30000001192092896,
    0.800000011920929
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
