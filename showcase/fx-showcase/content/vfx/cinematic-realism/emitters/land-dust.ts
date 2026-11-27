import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "alpha",
  "bursts": [
    {
      "count": 10,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    1,
    0.9200000166893005,
    0.7799999713897705,
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
      0.6000000238418579,
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
  "drag": 3.5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.07999999821186066,
    0
  ],
  "lifetime": [
    0.800000011920929,
    1.2999999523162842
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 12,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 86,
    "radius": 0.15000000596046448
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
    0.5
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
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
