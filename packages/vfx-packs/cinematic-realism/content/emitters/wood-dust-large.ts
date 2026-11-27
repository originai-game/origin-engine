import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 5,
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
        0.44999998807907104
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
    0.05000000074505806,
    0
  ],
  "lifetime": [
    1,
    1.600000023841858
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
    "kind": "sphere",
    "radius": 0.30000001192092896,
    "surface": false
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
  "softDistance": 0.20000000298023224,
  "speed": [
    0.4000000059604645,
    1.2000000476837158
  ],
  "spread": 180,
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
