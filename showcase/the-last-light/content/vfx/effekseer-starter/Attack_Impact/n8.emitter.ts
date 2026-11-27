import { file, ref } from '@origingame/engine/author';

export default file.particles({
  "blend": "additive",
  "bursts": [
    {
      "count": 1,
      "cycles": 20,
      "interval": 0.016699999570846558,
      "time": 0
    }
  ],
  "colorOverLife": [
    [
      0,
      [
        1,
        0.3587000072002411,
        0.3587000072002411,
        0
      ]
    ],
    [
      0.733299970626831,
      [
        1,
        0.3587000072002411,
        0.3587000072002411,
        0
      ]
    ],
    [
      1,
      [
        1,
        0.3587000072002411,
        0.3587000072002411,
        0
      ]
    ]
  ],
  "duration": 0.33329999446868896,
  "lifetime": [
    0.25,
    0.25
  ],
  "looping": false,
  "maxParticles": 20,
  "randomColorOverLife": [
    [
      0,
      [
        1,
        0.7519000172615051,
        0.7519000172615051,
        1
      ]
    ],
    [
      0.733299970626831,
      [
        1,
        0.7519000172615051,
        0.7519000172615051,
        1
      ]
    ],
    [
      1,
      [
        1,
        0.7519000172615051,
        0.7519000172615051,
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
    "kind": "box",
    "halfExtents": [
      0.5,
      0.5,
      0.5
    ]
  },
  "size": [
    0.125,
    0.375
  ],
  "speed": [
    2.598099946975708,
    10.39229965209961
  ],
  "spread": 180,
  "texture": ref("base/vfx/effekseer-starter/textures/particle2")
});
