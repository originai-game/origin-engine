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
      "count": 3,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    1.149999976158142,
    1.2000000476837158,
    1.25,
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
      0.10000000149011612,
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
  "drag": 2.5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.2000000476837158
  ],
  "looping": false,
  "maxParticles": 3,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 85,
    "radius": 0.05000000074505806
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
    0.18000000715255737,
    0.30000001192092896
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      1.7999999523162842
    ]
  ],
  "speed": [
    0.10000000149011612,
    0.30000001192092896
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust")
});
