import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 5,
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
        1,
        0.11760000139474869
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
  "duration": 0.016699999570846558,
  "lifetime": [
    0.33329999446868896,
    0.33329999446868896
  ],
  "looping": false,
  "maxParticles": 57,
  "rate": 0,
  "rotation": [
    -180,
    180
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    15,
    15
  ],
  "sizeOverLife": [
    [
      0,
      1
    ],
    [
      0.0625,
      0.7889999747276306
    ],
    [
      0.14059999585151672,
      0.5828999876976013
    ],
    [
      0.21879999339580536,
      0.4325000047683716
    ],
    [
      0.3125,
      0.3127000033855438
    ],
    [
      0.46880000829696655,
      0.21979999542236328
    ],
    [
      0.8281000256538391,
      0.15549999475479126
    ],
    [
      1,
      0
    ]
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/burst02_red")
});
