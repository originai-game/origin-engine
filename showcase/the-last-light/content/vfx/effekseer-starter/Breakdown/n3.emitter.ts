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
        0.7519000172615051,
        0.5356000065803528,
        1,
        0.6274999976158142
      ]
    ],
    [
      0.666700005531311,
      [
        0.7519000172615051,
        0.5356000065803528,
        1,
        0.6274999976158142
      ]
    ],
    [
      1,
      [
        0.7519000172615051,
        0.5356000065803528,
        1,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "heightOverLife": [
    [
      0,
      0.04170000180602074
    ],
    [
      1,
      0.25
    ]
  ],
  "lifetime": [
    0.25,
    0.25
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "rotation": [
    30,
    60
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    12,
    12
  ],
  "sizeOverLife": [
    [
      0,
      0.666700005531311
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
  "texture": ref("base/vfx/effekseer-starter/textures/ShockWave3")
});
