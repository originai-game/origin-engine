import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 6,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.8999999761581421,
    0.800000011920929,
    0.6200000047683716,
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
        0.4000000059604645
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
  "drag": 4,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    0.5,
    0.8999999761581421
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 6,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 35,
    "radius": 0.029999999329447746
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
    0.10000000149011612,
    0.18000000715255737
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.4000000953674316
    ]
  ],
  "softDistance": 0.10000000149011612,
  "speed": [
    0.10000000149011612,
    1.2999999523162842
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
