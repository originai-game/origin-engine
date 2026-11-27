import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 12,
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
      0.07999999821186066,
      [
        1,
        1,
        1,
        0.75
      ]
    ],
    [
      0.6000000238418579,
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
  "drag": 2.200000047683716,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.11999999731779099,
    0
  ],
  "lifetime": [
    1.7999999523162842,
    2.5999999046325684
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 14,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 84,
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
    0.44999998807907104,
    0.800000011920929
  ],
  "sizeOverLife": [
    [
      0,
      0.44999998807907104
    ],
    [
      0.5,
      1.100000023841858
    ],
    [
      1,
      1.5
    ]
  ],
  "softDistance": 0.25,
  "speed": [
    1.2000000476837158,
    2.200000047683716
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
