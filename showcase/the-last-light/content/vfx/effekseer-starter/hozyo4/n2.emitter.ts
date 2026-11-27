import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
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
        0.04780000075697899,
        0.04780000075697899,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.04780000075697899,
        0.04780000075697899,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "lifetime": [
    1,
    1
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "shape": {
    "kind": "point"
  },
  "size": [
    10,
    10
  ],
  "sizeOverLife": [
    [
      0,
      0.10000000149011612
    ],
    [
      0.09380000084638596,
      0.26750001311302185
    ],
    [
      0.20309999585151672,
      0.43149998784065247
    ],
    [
      0.3125,
      0.5655999779701233
    ],
    [
      0.4219000041484833,
      0.6743999719619751
    ],
    [
      0.6561999917030334,
      0.8416000008583069
    ],
    [
      1,
      1
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/Particle02-b223f51c")
});
