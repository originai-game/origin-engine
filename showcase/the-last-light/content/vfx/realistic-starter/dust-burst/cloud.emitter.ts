import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -20,
    20
  ],
  "blend": "alpha",
  "bursts": [
    {
      "count": 22,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "color": [
    0.800000011920929,
    0.7200000286102295,
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
        0.8999999761581421
      ]
    ],
    [
      0.6000000238418579,
      [
        1.0499999523162842,
        1.0499999523162842,
        1.0499999523162842,
        0.6499999761581421
      ]
    ],
    [
      1,
      [
        1.100000023841858,
        1.100000023841858,
        1.100000023841858,
        0
      ]
    ]
  ],
  "drag": 1.7999999523162842,
  "duration": 0.05000000074505806,
  "forceFields": 1,
  "gravity": [
    0,
    0.05000000074505806,
    0
  ],
  "lifetime": [
    2.200000047683716,
    3.4000000953674316
  ],
  "lit": true,
  "looping": false,
  "maxParticles": 32,
  "rate": 0,
  "rotation": [
    0,
    360
  ],
  "shape": {
    "kind": "box",
    "halfExtents": [
      0.800000011920929,
      0.30000001192092896,
      0.800000011920929
    ]
  },
  "sheet": {
    "blend": true,
    "columns": 2,
    "frames": 4,
    "mode": {
      "kind": "life",
      "cycles": 1
    },
    "rows": 2
  },
  "size": [
    0.8999999761581421,
    1.5
  ],
  "sizeOverLife": [
    [
      0,
      0.5
    ],
    [
      1,
      2.200000047683716
    ]
  ],
  "softDistance": 0.6000000238418579,
  "speed": [
    1,
    3
  ],
  "spread": 80,
  "texture": ref("base/vfx/realistic-starter/textures/smoke"),
  "wind": 1
});
