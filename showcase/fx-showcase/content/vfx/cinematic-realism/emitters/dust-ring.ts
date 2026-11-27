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
    0.8500000238418579,
    0.800000011920929,
    0.7200000286102295,
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
        0.550000011920929
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
  "drag": 3,
  "duration": 0.05000000074505806,
  "gravity": [
    0,
    0.10000000149011612,
    0
  ],
  "lifetime": [
    1.2000000476837158,
    2
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
    "radius": 0.20000000298023224
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
    0.6000000238418579,
    0.8999999761581421
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
  "softDistance": 0.30000001192092896,
  "speed": [
    3,
    5
  ],
  "texture": ref("base/vfx/cinematic-realism/textures/dust"),
  "wind": 1
});
