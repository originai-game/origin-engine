import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "angularVelocity": [
    -120,
    120
  ],
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 15,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.35370001196861267,
        0.35370001196861267,
        0.21960000693798065
      ]
    ],
    [
      1,
      [
        1,
        0.35370001196861267,
        0.35370001196861267,
        0
      ]
    ]
  ],
  "duration": 0.25,
  "heightOverLife": [
    [
      0,
      0.125
    ],
    [
      1,
      0.125
    ]
  ],
  "lifetime": [
    0.11670000106096268,
    0.11670000106096268
  ],
  "looping": false,
  "maxParticles": 10,
  "randomColorOverLife": [
    [
      0,
      [
        1,
        0.7444999814033508,
        0.7444999814033508,
        0.4392000138759613
      ]
    ],
    [
      1,
      [
        1,
        0.7444999814033508,
        0.7444999814033508,
        0
      ]
    ]
  ],
  "rate": 0,
  "rotation": [
    -180,
    180
  ],
  "shape": {
    "kind": "point"
  },
  "size": [
    2,
    6
  ],
  "speed": [
    0,
    0
  ],
  "texture": ref("base/vfx/effekseer-starter/textures/ShockWave3")
});
