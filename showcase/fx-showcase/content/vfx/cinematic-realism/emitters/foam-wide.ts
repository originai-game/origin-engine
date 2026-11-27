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
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 16,
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
      0.07999999821186066,
      [
        1,
        1,
        1,
        0.6000000238418579
      ]
    ],
    [
      0.6000000238418579,
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
  "drag": 1.5,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0,
    0
  ],
  "lifetime": [
    2.4000000953674316,
    3.4000000953674316
  ],
  "looping": false,
  "maxParticles": 18,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "cone",
    "angleDegrees": 86,
    "radius": 0.800000011920929
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
    1.399999976158142,
    2.200000047683716
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
    1.5,
    3
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust")
});
