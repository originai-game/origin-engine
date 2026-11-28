import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 4,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.800000011920929,
    0.8799999952316284,
    0.9200000166893005,
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
      0.11999999731779099,
      [
        1,
        1,
        1,
        0.30000001192092896
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
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.05000000074505806,
    0
  ],
  "lifetime": [
    0.5,
    1.100000023841858
  ],
  "looping": false,
  "maxParticles": 5,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 55,
    "radius": 0.07999999821186066
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
    0.3499999940395355,
    0.6000000238418579
  ],
  "sizeOverLife": [
    [
      0,
      0.4000000059604645
    ],
    [
      1,
      2
    ]
  ],
  "softDistance": 0.20000000298023224,
  "speed": [
    0.20000000298023224,
    0.5
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust")
});
