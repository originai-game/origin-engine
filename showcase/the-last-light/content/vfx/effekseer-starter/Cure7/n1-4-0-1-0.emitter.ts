import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 2,
      "cycles": 1,
      "interval": 0,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        1,
        0.31119999289512634,
        0
      ]
    ],
    [
      0.25,
      [
        1,
        1,
        0.4368000030517578,
        0.19609999656677246
      ]
    ],
    [
      0.5,
      [
        1,
        1,
        0.5860000252723694,
        0
      ]
    ],
    [
      1,
      [
        1,
        1,
        0.5860000252723694,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    0.666700005531311,
    0.666700005531311
  ],
  "looping": false,
  "maxParticles": 63,
  "rate": 0,
  "shape": {
    "kind": "spline",
    "closed": true,
    "points": [
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ],
      [
        0,
        0,
        0
      ]
    ]
  },
  "size": [
    0.7811999917030334,
    0.7811999917030334
  ],
  "sizeOverLife": [
    [
      0,
      0
    ],
    [
      0.25,
      1
    ],
    [
      0.5,
      0
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    0.14059999585151672,
    2.4844000339508057
  ],
  "speedOverLife": [
    [
      0,
      2.1308000087738037
    ],
    [
      0.25,
      1.3813999891281128
    ],
    [
      0.5,
      0.8472999930381775
    ],
    [
      0.75,
      0.5422999858856201
    ],
    [
      1,
      0.4657000005245209
    ]
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
