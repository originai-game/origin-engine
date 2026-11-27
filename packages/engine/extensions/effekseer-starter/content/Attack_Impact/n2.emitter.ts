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
        0.5356000065803528,
        0.5356000065803528,
        0.7529000043869019
      ]
    ],
    [
      0.800000011920929,
      [
        1,
        0.5356000065803528,
        0.5356000065803528,
        0.7529000043869019
      ]
    ],
    [
      1,
      [
        1,
        0.5356000065803528,
        0.5356000065803528,
        0
      ]
    ]
  ],
  "duration": 0.016699999570846558,
  "heightOverLife": [
    [
      0,
      0.16670000553131104
    ],
    [
      1,
      0.18330000340938568
    ]
  ],
  "lifetime": [
    0.16670000553131104,
    0.16670000553131104
  ],
  "looping": false,
  "maxParticles": 4,
  "rate": 0,
  "rotation": [
    15,
    75
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    4.5,
    7.5
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
